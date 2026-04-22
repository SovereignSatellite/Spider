//! Conformance tests for the `LuaJIT` target.

extern crate alloc;

use alloc::sync::Arc;
use std::{
	env,
	ffi::OsStr,
	fs::{self, File},
	io::{self, BufWriter, Write},
	path::{Path, PathBuf},
	thread,
};

use datatest_stable::Result;
use wast::{
	QuoteWat, WastArg, WastExecute, WastInvoke, WastRet, WastThread, Wat,
	core::{NanPattern, WastArgCore, WastRetCore},
	token::{F32, F64, Id, Span},
};

use luajit_builder::LuaJITBuilder;
use luajit_printer::{
	LuaJITPrinter,
	library::{NamesFinder, Printer as LibraryPrinter, Sections as LibrarySections},
};
use luau_builder as _;
use luau_printer as _;

use common::{compiler::Compiler, process, visitor::Visitor};

mod common;

const HARNESS_START_SOURCE: &str = include_str!("harness/luajit.start.lua");
const HARNESS_END_SOURCE: &str = include_str!("harness/luajit.end.lua");

const REPETITION_COUNT: usize = 32;

struct LuaJIT {
	library_sections: LibrarySections,
	library_printer: LibraryPrinter,
	references: Vec<&'static str>,

	compiler: Compiler,
	builder: LuaJITBuilder,
	printer: LuaJITPrinter,
	optimized: bool,

	file: Vec<u8>,
}

impl LuaJIT {
	fn new(optimized: bool) -> Self {
		let mut library_sections = LibrarySections::with_built_ins();

		library_sections.parse_from(HARNESS_START_SOURCE);
		library_sections.resolve();

		Self {
			library_sections,
			library_printer: LibraryPrinter::new(),
			references: Vec::new(),

			compiler: Compiler::new(),
			builder: LuaJITBuilder::new(),
			printer: LuaJITPrinter::new(),
			optimized,

			file: Vec::new(),
		}
	}

	fn write_into(mut self, out: &mut dyn Write) -> Result<()> {
		self.references.push("spectest");
		self.references.push("report_failure");

		self.references.sort_unstable();
		self.references.dedup();

		self.library_printer
			.resolve(&self.references, &self.library_sections);

		self.library_printer.print(&self.library_sections, out)?;

		out.write_all(&self.file)?;
		out.write_all(HARNESS_END_SOURCE.as_bytes())?;

		Ok(())
	}

	fn fmt_source(&mut self, data: &[u8]) -> Result<()> {
		let graph = self.compiler.run(data, self.optimized);
		let tree = self.builder.run(&graph);

		NamesFinder::new(&mut self.references).run(&tree);

		self.printer.indent();
		self.printer.print(&tree, &mut self.file)?;
		self.printer.outdent();

		Ok(())
	}

	fn fmt_name(&mut self, id: Id<'_>) -> Result<()> {
		let identifier = id.name().as_bytes().escape_ascii();

		write!(self.file, "named[\"{identifier}\"]")?;

		Ok(())
	}

	fn fmt_optional_name(&mut self, id: Option<Id<'_>>) -> Result<()> {
		if let Some(id) = id {
			self.fmt_name(id)?;
		} else {
			write!(self.file, "rt_export_map")?;
		}

		Ok(())
	}

	fn fmt_named_source(&mut self, id: Option<Id<'_>>, data: &[u8]) -> Result<()> {
		self.fmt_source(data)?;

		writeln!(self.file, "\trt_export_map = {{}}")?;
		writeln!(self.file, "\tmodule()")?;

		if let Some(id) = id {
			self.fmt_name(id)?;

			writeln!(self.file, " = rt_export_map")?;
		}

		Ok(())
	}

	fn fmt_argument_i32(&mut self, value: i32) -> Result<()> {
		write!(self.file, "{value} --[[ 0x{value:08X} ]]")?;

		Ok(())
	}

	fn fmt_argument_i64(&mut self, value: i64) -> Result<()> {
		write!(self.file, "{value}LL --[[ 0x{value:016X} ]]")?;

		Ok(())
	}

	fn fmt_argument_f32(&mut self, value: F32) -> Result<()> {
		let F32 { bits } = value;
		let float = f32::from_bits(bits);
		let bits = i32::from_ne_bytes(bits.to_ne_bytes());

		write!(self.file, "{bits} --[[ {float}_f32 ]]")?;

		Ok(())
	}

	fn fmt_argument_f64(&mut self, value: F64) -> Result<()> {
		let F64 { bits } = value;
		let float = f64::from_bits(bits);
		let bits = i64::from_ne_bytes(bits.to_ne_bytes());

		write!(self.file, "{bits}LL --[[ {float}_f64 ]]")?;

		Ok(())
	}

	fn fmt_argument(&mut self, argument: WastArg<'_>) -> Result<()> {
		let WastArg::Core(argument) = argument else {
			unimplemented!()
		};

		match argument {
			WastArgCore::I32(i32) => {
				self.fmt_argument_i32(i32)?;

				Ok(())
			}
			WastArgCore::I64(i64) => {
				self.fmt_argument_i64(i64)?;

				Ok(())
			}
			WastArgCore::F32(f32) => {
				self.fmt_argument_f32(f32)?;

				Ok(())
			}
			WastArgCore::F64(f64) => {
				self.fmt_argument_f64(f64)?;

				Ok(())
			}
			WastArgCore::V128(_) => unimplemented!(),
			WastArgCore::RefNull(_) => write!(self.file, "nil --[[ heap ]]"),
			WastArgCore::RefExtern(_) => write!(self.file, "newproxy(false) --[[ extern ]]"),
			WastArgCore::RefHost(_) => write!(self.file, "newproxy(false) --[[ host ]]"),
		}?;

		Ok(())
	}

	fn fmt_argument_list(&mut self, arguments: Vec<WastArg<'_>>) -> Result<()> {
		arguments
			.into_iter()
			.enumerate()
			.try_for_each(|(index, argument)| {
				if index != 0 {
					write!(self.file, ", ")?;
				}

				self.fmt_argument(argument)
			})
	}

	fn fmt_export(&mut self, module: Option<Id<'_>>, identifier: &str) -> Result<()> {
		let identifier = identifier.as_bytes().escape_ascii();

		self.fmt_optional_name(module)?;

		write!(self.file, "[\"{identifier}\"]")?;

		Ok(())
	}

	fn fmt_invoke(&mut self, invoke: WastInvoke<'_>) -> Result<()> {
		self.references.push("call_closure");

		write!(self.file, "hn_call_closure(")?;

		self.fmt_export(invoke.module, invoke.name)?;

		if !invoke.args.is_empty() {
			write!(self.file, ", ")?;

			self.fmt_argument_list(invoke.args)?;
		}

		write!(self.file, ")")?;

		Ok(())
	}

	fn fmt_wat(&mut self, mut wat: Wat<'_>) -> Result<()> {
		self.fmt_source(&wat.encode()?)?;

		writeln!(self.file, "rt_export_map = {{}}")?;
		writeln!(self.file, "module()")?;

		Ok(())
	}

	fn fmt_get(&mut self, module: Option<Id<'_>>, global: &str) -> Result<()> {
		self.fmt_export(module, global)?;

		write!(self.file, "[1]")?;

		Ok(())
	}

	fn fmt_execute(&mut self, exec: WastExecute<'_>) -> Result<()> {
		match exec {
			WastExecute::Invoke(wast_invoke) => self.fmt_invoke(wast_invoke),
			WastExecute::Wat(wat) => self.fmt_wat(wat),
			WastExecute::Get { module, global, .. } => self.fmt_get(module, global),
		}
	}

	fn fmt_assert_equal_i32(&mut self, value: i32) -> Result<()> {
		self.references.push("assert_equal_i32");

		write!(
			self.file,
			"hn_assert_equal_i32({value}) --[[ 0x{value:08X} ]]"
		)?;

		Ok(())
	}

	fn fmt_assert_equal_i64(&mut self, value: i64) -> Result<()> {
		self.references.push("assert_equal_i64");

		write!(
			self.file,
			"hn_assert_equal_i64({value}LL) --[[ 0x{value:016X} ]]"
		)?;

		Ok(())
	}

	fn fmt_assert_equal_f32(&mut self, value: F32) -> Result<()> {
		let F32 { bits } = value;
		let float = f32::from_bits(bits);
		let bits = i32::from_ne_bytes(bits.to_ne_bytes());

		self.references.push("assert_equal_f32");

		write!(self.file, "hn_assert_equal_f32({bits}) --[[ {float}_f32 ]]")?;

		Ok(())
	}

	fn fmt_assert_equal_f64(&mut self, value: F64) -> Result<()> {
		let F64 { bits } = value;
		let float = f64::from_bits(bits);
		let bits = i64::from_ne_bytes(bits.to_ne_bytes());

		self.references.push("assert_equal_f64");

		write!(
			self.file,
			"hn_assert_equal_f64({bits}LL) --[[ {float}_f64 ]]"
		)?;

		Ok(())
	}

	#[expect(
		clippy::too_many_lines,
		reason = "exhaustive match over wast result pattern variants"
	)]
	fn fmt_assert_pattern(&mut self, result: WastRet<'_>) -> Result<()> {
		let WastRet::Core(result) = result else {
			unimplemented!()
		};

		match result {
			WastRetCore::I32(i32) => {
				self.fmt_assert_equal_i32(i32)?;

				Ok(())
			}
			WastRetCore::I64(i64) => {
				self.fmt_assert_equal_i64(i64)?;

				Ok(())
			}

			WastRetCore::F32(NanPattern::CanonicalNan) => {
				self.references.push("is_f32_nan_canonical");

				write!(self.file, "hn_is_f32_nan_canonical")
			}
			WastRetCore::F32(NanPattern::ArithmeticNan) => {
				self.references.push("is_f32_nan_arithmetic");

				write!(self.file, "hn_is_f32_nan_arithmetic")
			}
			WastRetCore::F32(NanPattern::Value(f32)) => {
				self.fmt_assert_equal_f32(f32)?;

				Ok(())
			}

			WastRetCore::F64(NanPattern::CanonicalNan) => {
				self.references.push("is_f64_nan_canonical");

				write!(self.file, "hn_is_f64_nan_canonical")
			}
			WastRetCore::F64(NanPattern::ArithmeticNan) => {
				self.references.push("is_f64_nan_arithmetic");

				write!(self.file, "hn_is_f64_nan_arithmetic")
			}
			WastRetCore::F64(NanPattern::Value(f64)) => {
				self.fmt_assert_equal_f64(f64)?;

				Ok(())
			}

			WastRetCore::RefNull(_) => {
				self.references.push("assert_ref_null");

				write!(self.file, "hn_assert_ref_null")
			}
			WastRetCore::RefExtern(_) => {
				self.references.push("assert_ref_extern");

				write!(self.file, "hn_assert_ref_extern")
			}

			WastRetCore::V128(_)
			| WastRetCore::RefHost(_)
			| WastRetCore::RefFunc(_)
			| WastRetCore::RefAny
			| WastRetCore::RefEq
			| WastRetCore::RefArray
			| WastRetCore::RefStruct
			| WastRetCore::RefI31
			| WastRetCore::RefI31Shared
			| WastRetCore::Either(_) => unimplemented!(),
		}?;

		Ok(())
	}
}

impl Visitor for LuaJIT {
	fn visit_module(&mut self, mut quote_wat: QuoteWat<'_>) -> Result<()> {
		let data = quote_wat.encode()?;

		writeln!(self.file, "do")?;

		self.fmt_named_source(quote_wat.name(), &data)?;

		writeln!(self.file, "end")?;

		Ok(())
	}

	fn visit_module_definition(&mut self, _quote_wat: QuoteWat<'_>) -> Result<()> {
		unimplemented!()
	}

	fn visit_module_instance(
		&mut self,
		_span: Span,
		_instance: Option<Id<'_>>,
		_module: Option<Id<'_>>,
	) -> Result<()> {
		unimplemented!()
	}

	fn visit_assert_malformed(
		&mut self,
		_span: Span,
		_module: QuoteWat<'_>,
		_message: &str,
	) -> Result<()> {
		Ok(())
	}

	fn visit_assert_invalid(
		&mut self,
		_span: Span,
		_module: QuoteWat<'_>,
		_message: &str,
	) -> Result<()> {
		Ok(())
	}

	fn visit_register(&mut self, _span: Span, name: &str, module: Option<Id<'_>>) -> Result<()> {
		let name = name.as_bytes().escape_ascii();

		write!(self.file, "rt_import_map[\"{name}\"] = ")?;

		self.fmt_optional_name(module)?;

		writeln!(self.file)?;

		Ok(())
	}

	fn visit_invoke(&mut self, wast_invoke: WastInvoke<'_>) -> Result<()> {
		self.fmt_invoke(wast_invoke)?;

		writeln!(self.file)?;

		Ok(())
	}

	fn visit_assert_trap(
		&mut self,
		_span: Span,
		exec: WastExecute<'_>,
		message: &str,
	) -> Result<()> {
		let message = message.as_bytes().escape_ascii();

		self.references.push("assert_trap");

		writeln!(self.file, "hn_assert_trap(\"{message}\", function()")?;

		self.fmt_execute(exec)?;

		writeln!(self.file, "\nend)")?;

		Ok(())
	}

	fn visit_assert_return(
		&mut self,
		_span: Span,
		exec: WastExecute<'_>,
		results: Vec<WastRet<'_>>,
	) -> Result<()> {
		writeln!(self.file, "do")?;
		write!(self.file, "\tlocal sources = {{ ")?;

		self.fmt_execute(exec)?;

		writeln!(self.file, " }}")?;

		for (result, index) in results.into_iter().zip(1_i32..) {
			write!(self.file, "\t")?;

			self.fmt_assert_pattern(result)?;

			writeln!(self.file, "(sources[{index}])")?;
		}

		writeln!(self.file, "end")?;

		Ok(())
	}

	fn visit_assert_exhaustion(
		&mut self,
		_span: Span,
		_call: WastInvoke<'_>,
		_message: &str,
	) -> Result<()> {
		Ok(())
	}

	fn visit_assert_unlinkable(
		&mut self,
		_span: Span,
		_module: Wat<'_>,
		_message: &str,
	) -> Result<()> {
		Ok(())
	}

	fn visit_assert_exception(&mut self, _span: Span, _exec: WastExecute<'_>) -> Result<()> {
		Ok(())
	}

	fn visit_assert_suspension(
		&mut self,
		_span: Span,
		_exec: WastExecute<'_>,
		_message: &str,
	) -> Result<()> {
		Ok(())
	}

	fn visit_thread(&mut self, _wast_thread: WastThread<'_>) -> Result<()> {
		Ok(())
	}

	fn visit_wait(&mut self, _span: Span, _thread: Id<'_>) -> Result<()> {
		Ok(())
	}
}

fn get_path_target(name: &OsStr, optimized: bool, native: bool) -> Result<Arc<Path>> {
	let mut path = [
		env!("CARGO_TARGET_TMPDIR"),
		if native { "native" } else { "interpreter" },
		if optimized { "O3" } else { "O0" },
	]
	.iter()
	.collect::<PathBuf>();

	fs::create_dir_all(&path)?;

	path.push(name);
	path.set_extension("luajit.lua");

	Ok(path.into())
}

fn compile_test(destination: &Path, tested: &str, optimized: bool) -> Result<()> {
	let mut luajit = LuaJIT::new(optimized);

	luajit.visit(tested)?;

	let mut destination = File::create(destination).map(BufWriter::new)?;

	luajit.write_into(&mut destination)?;

	destination.flush()?;

	Ok(())
}

fn run_file(destination: &Path, optimized: bool, native: bool) -> io::Result<Box<str>> {
	let arguments = [
		OsStr::new(if optimized { "-O3" } else { "-O0" }),
		OsStr::new(if native { "-jon" } else { "-joff" }),
		destination.as_ref(),
	];

	let program = env::var_os("LUAJIT_PATH").unwrap_or_else(|| "luajit".into());
	let output = process::run(&program, &arguments)?;

	Ok(output)
}

fn run_and_assert(path: &Path, optimized: bool, native: bool) -> Result<()> {
	let tested = fs::read_to_string(path)?;
	let destination = get_path_target(path.file_name().unwrap(), optimized, native)?;

	compile_test(&destination, &tested, optimized)?;

	let mut handles = Vec::with_capacity(REPETITION_COUNT);

	for _ in 0..REPETITION_COUNT {
		let destination = Arc::clone(&destination);
		let handle = thread::spawn(move || run_file(&destination, optimized, native));

		handles.push(handle);
	}

	for (index, handle) in handles.into_iter().enumerate() {
		let output = handle.join().unwrap()?;

		assert!(output.is_empty(), "run {index} {output}");
	}

	Ok(())
}

fn bytecode_o0(path: &Path) -> Result<()> {
	run_and_assert(path, false, false)
}

fn bytecode_o3(path: &Path) -> Result<()> {
	run_and_assert(path, true, false)
}

fn native_o0(path: &Path) -> Result<()> {
	run_and_assert(path, false, true)
}

fn native_o3(path: &Path) -> Result<()> {
	run_and_assert(path, true, true)
}

datatest_stable::harness! {
	{ test = bytecode_o0, root = "Suite", pattern = r"^(?!simd_)\w+\.wast$" },
	{ test = bytecode_o3, root = "Suite", pattern = r"^(?!simd_)\w+\.wast$" },
	{ test = native_o0, root = "Suite", pattern = r"^(?!simd_)\w+\.wast$" },
	{ test = native_o3, root = "Suite", pattern = r"^(?!simd_)\w+\.wast$" },
}
