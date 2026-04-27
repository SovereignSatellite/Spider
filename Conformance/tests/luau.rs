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
	is_optimized: bool,

	file: Vec<u8>,
}

impl Luau {
	fn new(is_optimized: bool) -> Self {
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
			is_optimized,

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

	fn format_source(&mut self, data: &[u8]) -> Result<()> {
		let root = self.compiler.run(data, self.is_optimized);
		let function = self.builder.run(&root);

		NamesFinder::new(&mut self.references).run(&function);

		self.printer.indent();
		self.printer.print(&function, &mut self.file)?;
		self.printer.outdent();

		Ok(())
	}

	fn format_name(&mut self, id: Id<'_>) -> Result<()> {
		let identifier = id.name().as_bytes().escape_ascii();

		write!(self.file, "named[\"{identifier}\"]")?;

		Ok(())
	}

	fn format_optional_name(&mut self, id: Option<Id<'_>>) -> Result<()> {
		if let Some(id) = id {
			self.format_name(id)?;
		} else {
			write!(self.file, "rt_export_map")?;
		}

		Ok(())
	}

	fn format_named_source(&mut self, id: Option<Id<'_>>, data: &[u8]) -> Result<()> {
		self.format_source(data)?;

		writeln!(self.file, "\trt_export_map = {{}}")?;
		writeln!(self.file, "\tmodule()")?;

		if let Some(id) = id {
			self.format_name(id)?;

			writeln!(self.file, " = rt_export_map")?;
		}

		Ok(())
	}

	fn format_argument_i32(&mut self, value: i32) -> Result<()> {
		let value = u32::from_ne_bytes(value.to_ne_bytes());

		write!(self.file, "{value} --[[ 0x{value:08X} ]]")?;

		Ok(())
	}

	fn format_argument_i64(&mut self, value: i64) -> Result<()> {
		let [b1, b2, b3, b4, b5, b6, b7, b8] = value.to_le_bytes();

		let low_bits = u32::from_le_bytes([b1, b2, b3, b4]);
		let high_bits = u32::from_le_bytes([b5, b6, b7, b8]);

		self.references.push("into_bits_i64");

		write!(
			self.file,
			"into_bits_i64({low_bits}, {high_bits}) --[[ 0x{value:016X} ]]"
		)?;

		Ok(())
	}

	fn format_argument_f32(&mut self, value: F32) -> Result<()> {
		let F32 { bits } = value;
		let float = f32::from_bits(bits);

		write!(self.file, "{bits} --[[ {float}_f32 ]]")?;

		Ok(())
	}

	fn format_argument_f64(&mut self, value: F64) -> Result<()> {
		let F64 { bits } = value;
		let float = f64::from_bits(bits);

		let [b1, b2, b3, b4, b5, b6, b7, b8] = bits.to_le_bytes();

		let low_bits = u32::from_le_bytes([b1, b2, b3, b4]);
		let high_bits = u32::from_le_bytes([b5, b6, b7, b8]);

		self.references.push("into_bits_i64");

		write!(
			self.file,
			"into_bits_i64({low_bits}, {high_bits}) --[[ {float}_f64 ]]"
		)?;

		Ok(())
	}

	fn format_argument(&mut self, argument: WastArg<'_>) -> Result<()> {
		let WastArg::Core(argument) = argument else {
			unimplemented!()
		};

		match argument {
			WastArgCore::I32(value) => {
				self.format_argument_i32(value)?;

				Ok(())
			}
			WastArgCore::I64(value) => {
				self.format_argument_i64(value)?;

				Ok(())
			}
			WastArgCore::F32(value) => {
				self.format_argument_f32(value)?;

				Ok(())
			}
			WastArgCore::F64(value) => {
				self.format_argument_f64(value)?;

				Ok(())
			}
			WastArgCore::V128(_) => unimplemented!(),
			WastArgCore::RefNull(_) => write!(self.file, "nil --[[ heap ]]"),
			WastArgCore::RefExtern(_) => write!(self.file, "newproxy(false) --[[ extern ]]"),
			WastArgCore::RefHost(_) => write!(self.file, "newproxy(false) --[[ host ]]"),
		}?;

		Ok(())
	}

	fn format_argument_list(&mut self, arguments: Vec<WastArg<'_>>) -> Result<()> {
		arguments
			.into_iter()
			.enumerate()
			.try_for_each(|(index, argument)| {
				if index != 0 {
					write!(self.file, ", ")?;
				}

				self.format_argument(argument)
			})
	}

	fn format_export(&mut self, module: Option<Id<'_>>, identifier: &str) -> Result<()> {
		let identifier = identifier.as_bytes().escape_ascii();

		self.format_optional_name(module)?;

		write!(self.file, "[\"{identifier}\"]")?;

		Ok(())
	}

	fn format_invoke(&mut self, invoke: WastInvoke<'_>) -> Result<()> {
		self.references.push("call_closure");

		write!(self.file, "hn_call_closure(")?;

		self.format_export(invoke.module, invoke.name)?;

		if !invoke.args.is_empty() {
			write!(self.file, ", ")?;

			self.format_argument_list(invoke.args)?;
		}

		write!(self.file, ")")?;

		Ok(())
	}

	fn format_wat(&mut self, mut wat: Wat<'_>) -> Result<()> {
		self.format_source(&wat.encode()?)?;

		writeln!(self.file, "rt_export_map = {{}}")?;
		writeln!(self.file, "module()")?;

		Ok(())
	}

	fn format_get(&mut self, module: Option<Id<'_>>, global: &str) -> Result<()> {
		self.format_export(module, global)?;

		write!(self.file, "[1]")?;

		Ok(())
	}

	fn format_execute(&mut self, wast_execute: WastExecute<'_>) -> Result<()> {
		match wast_execute {
			WastExecute::Invoke(wast_invoke) => self.format_invoke(wast_invoke),
			WastExecute::Wat(wat) => self.format_wat(wat),
			WastExecute::Get { module, global, .. } => self.format_get(module, global),
		}
	}

	fn format_assert_equal_i32(&mut self, value: i32) -> Result<()> {
		let value = u32::from_ne_bytes(value.to_ne_bytes());

		self.references.push("assert_equal_i32");

		write!(
			self.file,
			"hn_assert_equal_i32({value}) --[[ 0x{value:08X} ]]"
		)?;

		Ok(())
	}

	fn format_assert_equal_i64(&mut self, value: i64) -> Result<()> {
		let [b1, b2, b3, b4, b5, b6, b7, b8] = value.to_le_bytes();

		let low_bits = u32::from_le_bytes([b1, b2, b3, b4]);
		let high_bits = u32::from_le_bytes([b5, b6, b7, b8]);

		self.references.push("assert_equal_i64");
		self.references.push("into_bits_i64");

		write!(
			self.file,
			"hn_assert_equal_i64(into_bits_i64({low_bits}, {high_bits})) --[[ 0x{value:016X} ]]"
		)?;

		Ok(())
	}

	fn format_assert_equal_f32(&mut self, value: F32) -> Result<()> {
		let F32 { bits } = value;
		let float = f32::from_bits(bits);

		self.references.push("assert_equal_f32");

		write!(self.file, "hn_assert_equal_f32({bits}) --[[ {float}_f32 ]]")?;

		Ok(())
	}

	fn format_assert_equal_f64(&mut self, value: F64) -> Result<()> {
		let F64 { bits } = value;
		let float = f64::from_bits(bits);

		let [b1, b2, b3, b4, b5, b6, b7, b8] = bits.to_le_bytes();

		let low_bits = u32::from_le_bytes([b1, b2, b3, b4]);
		let high_bits = u32::from_le_bytes([b5, b6, b7, b8]);

		self.references.push("assert_equal_f64");
		self.references.push("into_bits_i64");

		write!(
			self.file,
			"hn_assert_equal_f64(into_bits_i64({low_bits}, {high_bits})) --[[ {float}_f64 ]]"
		)?;

		Ok(())
	}

	#[expect(
		clippy::too_many_lines,
		reason = "exhaustive match over wast result pattern variants"
	)]
	fn format_assert_pattern(&mut self, result: WastRet<'_>) -> Result<()> {
		let WastRet::Core(result) = result else {
			unimplemented!()
		};

		match result {
			WastRetCore::I32(value) => {
				self.format_assert_equal_i32(value)?;

				Ok(())
			}
			WastRetCore::I64(value) => {
				self.format_assert_equal_i64(value)?;

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
				self.format_assert_equal_f32(value)?;

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
				self.format_assert_equal_f64(value)?;

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

		self.format_named_source(quote_wat.name(), &data)?;

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

		self.format_optional_name(module)?;

		writeln!(self.file)?;

		Ok(())
	}

	fn visit_invoke(&mut self, wast_invoke: WastInvoke<'_>) -> Result<()> {
		self.format_invoke(wast_invoke)?;

		writeln!(self.file)?;

		Ok(())
	}

	fn visit_assert_trap(
		&mut self,
		_span: Span,
		wast_execute: WastExecute<'_>,
		message: &str,
	) -> Result<()> {
		let message = message.as_bytes().escape_ascii();

		self.references.push("assert_trap");

		writeln!(self.file, "hn_assert_trap(\"{message}\", function()")?;

		self.format_execute(wast_execute)?;

		writeln!(self.file, "\nend)")?;

		Ok(())
	}

	fn visit_assert_return(
		&mut self,
		_span: Span,
		wast_execute: WastExecute<'_>,
		results: Vec<WastRet<'_>>,
	) -> Result<()> {
		writeln!(self.file, "do")?;
		write!(self.file, "\tlocal sources = {{ ")?;

		self.format_execute(wast_execute)?;

		writeln!(self.file, " }}")?;

		for (result, index) in results.into_iter().zip(1_i32..) {
			write!(self.file, "\t")?;

			self.format_assert_pattern(result)?;

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

	fn visit_assert_exception(
		&mut self,
		_span: Span,
		_wast_execute: WastExecute<'_>,
	) -> Result<()> {
		Ok(())
	}

	fn visit_assert_suspension(
		&mut self,
		_span: Span,
		_wast_execute: WastExecute<'_>,
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

fn get_path_target(name: &OsStr, is_optimized: bool, is_native: bool) -> Result<Arc<Path>> {
	let mut path = [
		env!("CARGO_TARGET_TMPDIR"),
		if is_native { "native" } else { "interpreter" },
		if is_optimized { "O2" } else { "O0" },
	]
	.iter()
	.collect::<PathBuf>();

	fs::create_dir_all(&path)?;

	path.push(name);
	path.set_extension("luau");

	Ok(path.into())
}

fn compile_test(destination: &Path, tested: &str, is_optimized: bool) -> Result<()> {
	let mut luau = Luau::new(is_optimized);

	luau.visit(tested)?;

	let mut destination = File::create(destination).map(BufWriter::new)?;

	luau.write_into(&mut destination)?;

	destination.flush()?;

	Ok(())
}

fn run_file(destination: &Path, is_optimized: bool, is_native: bool) -> io::Result<Box<str>> {
	let mut arguments = vec![OsStr::new(if is_optimized { "-O2" } else { "-O0" })];

	if is_native {
		arguments.push(OsStr::new("--codegen"));
	}

	arguments.push(destination.as_ref());

	let program = env::var_os("LUAU_PATH").unwrap_or_else(|| "luau".into());
	let output = process::run(&program, &arguments)?;

	Ok(output)
}

fn run_and_assert(path: &Path, is_optimized: bool, is_native: bool) -> Result<()> {
	let tested = fs::read_to_string(path)?;
	let destination = get_path_target(path.file_name().unwrap(), is_optimized, is_native)?;

	compile_test(&destination, &tested, is_optimized)?;

	let mut handles = Vec::with_capacity(REPETITION_COUNT);

	for _ in 0..REPETITION_COUNT {
		let destination = Arc::clone(&destination);
		let handle = thread::spawn(move || run_file(&destination, is_optimized, is_native));

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
