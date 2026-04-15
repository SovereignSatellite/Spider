//! Conformance tests for the Luau target.

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

use luajit_builder as _;
use luajit_printer as _;
use luau_builder::LuauBuilder;
use luau_printer::{
	LuauPrinter,
	library::{NamesFinder, Printer as LibraryPrinter, Sections as LibrarySections},
};

use common::{compiler::Compiler, process, visitor::Visitor};

mod common;

const HARNESS_START_SOURCE: &str = include_str!("harness/luau.start.luau");
const HARNESS_END_SOURCE: &str = include_str!("harness/luau.end.luau");

const REPETITION_COUNT: usize = 32;

struct Luau {
	library_sections: LibrarySections,
	library_printer: LibraryPrinter,
	references: Vec<&'static str>,

	compiler: Compiler,
	builder: LuauBuilder,
	printer: LuauPrinter,
	optimized: bool,

	file: Vec<u8>,
}

impl Luau {
	fn new(optimized: bool) -> Self {
		let mut library_sections = LibrarySections::with_built_ins();

		library_sections.parse_from(HARNESS_START_SOURCE);
		library_sections.resolve();

		Self {
			library_sections,
			library_printer: LibraryPrinter::new(),
			references: Vec::new(),

			compiler: Compiler::new(),
			builder: LuauBuilder::new(),
			printer: LuauPrinter::new(),
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
		let module = self.compiler.run(data, self.optimized);
		let tree = self.builder.run(&module);

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
			write!(self.file, "selected")?;
		}

		Ok(())
	}

	fn fmt_named_source(&mut self, id: Option<Id<'_>>, data: &[u8]) -> Result<()> {
		self.fmt_source(data)?;

		writeln!(self.file, "\tselected = module(environment)")?;

		if let Some(id) = id {
			self.fmt_name(id)?;

			writeln!(self.file, " = selected")?;
		}

		Ok(())
	}

	fn fmt_argument_i32(&mut self, value: i32) -> Result<()> {
		let value = u32::from_ne_bytes(value.to_ne_bytes());

		write!(self.file, "{value} --[[ 0x{value:08X} ]]")?;

		Ok(())
	}

	fn fmt_argument_i64(&mut self, value: i64) -> Result<()> {
		let [b1, b2, b3, b4, b5, b6, b7, b8] = value.to_le_bytes();

		let source_1 = u32::from_le_bytes([b1, b2, b3, b4]);
		let source_2 = u32::from_le_bytes([b5, b6, b7, b8]);

		self.references.push("into_bits_i64");

		write!(
			self.file,
			"into_bits_i64({source_1}, {source_2}) --[[ 0x{value:016X} ]]"
		)?;

		Ok(())
	}

	fn fmt_argument_f32(&mut self, value: F32) -> Result<()> {
		let F32 { bits } = value;
		let float = f32::from_bits(bits);

		write!(self.file, "{bits} --[[ {float}_f32 ]]")?;

		Ok(())
	}

	fn fmt_argument_f64(&mut self, value: F64) -> Result<()> {
		let F64 { bits } = value;
		let float = f64::from_bits(bits);

		let [b1, b2, b3, b4, b5, b6, b7, b8] = bits.to_le_bytes();

		let source_1 = u32::from_le_bytes([b1, b2, b3, b4]);
		let source_2 = u32::from_le_bytes([b5, b6, b7, b8]);

		self.references.push("into_bits_i64");

		write!(
			self.file,
			"into_bits_i64({source_1}, {source_2}) --[[ {float}_f64 ]]"
		)?;

		Ok(())
	}

	fn fmt_argument(&mut self, argument: WastArg<'_>) -> Result<()> {
		let WastArg::Core(argument) = argument else {
			unimplemented!()
		};

		match argument {
			WastArgCore::I32(value) => {
				self.fmt_argument_i32(value)?;

				Ok(())
			}
			WastArgCore::I64(value) => {
				self.fmt_argument_i64(value)?;

				Ok(())
			}
			WastArgCore::F32(value) => {
				self.fmt_argument_f32(value)?;

				Ok(())
			}
			WastArgCore::F64(value) => {
				self.fmt_argument_f64(value)?;

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
		self.fmt_export(invoke.module, invoke.name)?;

		write!(self.file, "(")?;

		self.fmt_argument_list(invoke.args)?;

		write!(self.file, ")")?;

		Ok(())
	}

	fn fmt_wat(&mut self, mut wat: Wat<'_>) -> Result<()> {
		self.fmt_source(&wat.encode()?)?;

		writeln!(self.file, "module(environment)")?;

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
		let value = u32::from_ne_bytes(value.to_ne_bytes());

		self.references.push("assert_equal_i32");

		write!(
			self.file,
			"hn_assert_equal_i32({value}) --[[ 0x{value:08X} ]]"
		)?;

		Ok(())
	}

	fn fmt_assert_equal_i64(&mut self, value: i64) -> Result<()> {
		let [b1, b2, b3, b4, b5, b6, b7, b8] = value.to_le_bytes();

		let source_1 = u32::from_le_bytes([b1, b2, b3, b4]);
		let source_2 = u32::from_le_bytes([b5, b6, b7, b8]);

		self.references.push("assert_equal_i64");
		self.references.push("into_bits_i64");

		write!(
			self.file,
			"hn_assert_equal_i64(into_bits_i64({source_1}, {source_2})) --[[ 0x{value:016X} ]]"
		)?;

		Ok(())
	}

	fn fmt_assert_equal_f32(&mut self, value: F32) -> Result<()> {
		let F32 { bits } = value;
		let float = f32::from_bits(bits);

		self.references.push("assert_equal_f32");

		write!(self.file, "hn_assert_equal_f32({bits}) --[[ {float}_f32 ]]")?;

		Ok(())
	}

	fn fmt_assert_equal_f64(&mut self, value: F64) -> Result<()> {
		let F64 { bits } = value;
		let float = f64::from_bits(bits);

		let [b1, b2, b3, b4, b5, b6, b7, b8] = bits.to_le_bytes();

		let source_1 = u32::from_le_bytes([b1, b2, b3, b4]);
		let source_2 = u32::from_le_bytes([b5, b6, b7, b8]);

		self.references.push("assert_equal_f64");
		self.references.push("into_bits_i64");

		write!(
			self.file,
			"hn_assert_equal_f64(into_bits_i64({source_1}, {source_2})) --[[ {float}_f64 ]]"
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
			WastRetCore::I32(value) => {
				self.fmt_assert_equal_i32(value)?;

				Ok(())
			}
			WastRetCore::I64(value) => {
				self.fmt_assert_equal_i64(value)?;

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
			WastRetCore::F32(NanPattern::Value(value)) => {
				self.fmt_assert_equal_f32(value)?;

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
			WastRetCore::F64(NanPattern::Value(value)) => {
				self.fmt_assert_equal_f64(value)?;

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

impl Visitor for Luau {
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

		write!(self.file, "environment[\"{name}\"] = ")?;

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
		if optimized { "O2" } else { "O0" },
	]
	.iter()
	.collect::<PathBuf>();

	fs::create_dir_all(&path)?;

	path.push(name);
	path.set_extension("luau");

	Ok(path.into())
}

fn compile_test(destination: &Path, tested: &str, optimized: bool) -> Result<()> {
	let mut luau = Luau::new(optimized);

	luau.visit(tested)?;

	let mut destination = File::create(destination).map(BufWriter::new)?;

	luau.write_into(&mut destination)?;

	destination.flush()?;

	Ok(())
}

fn run_file(destination: &Path, optimized: bool, native: bool) -> io::Result<Box<str>> {
	let mut arguments = vec![OsStr::new(if optimized { "-O2" } else { "-O0" })];

	if native {
		arguments.push(OsStr::new("--codegen"));
	}

	arguments.push(destination.as_ref());

	let program = env::var_os("LUAU_PATH").unwrap_or_else(|| "luau".into());
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

fn bytecode_o2(path: &Path) -> Result<()> {
	run_and_assert(path, true, false)
}

fn native_o0(path: &Path) -> Result<()> {
	run_and_assert(path, false, true)
}

fn native_o2(path: &Path) -> Result<()> {
	run_and_assert(path, true, true)
}

datatest_stable::harness! {
	{ test = bytecode_o0, root = "Suite", pattern = r"^(?!simd_)\w+\.wast$" },
	{ test = bytecode_o2, root = "Suite", pattern = r"^(?!simd_)\w+\.wast$" },
	{ test = native_o0, root = "Suite", pattern = r"^(?!simd_)\w+\.wast$" },
	{ test = native_o2, root = "Suite", pattern = r"^(?!simd_)\w+\.wast$" },
}
