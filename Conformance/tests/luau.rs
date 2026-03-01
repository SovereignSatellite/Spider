use std::{
	ffi::OsStr,
	fs::File,
	io::{BufWriter, Write},
	path::{Path, PathBuf},
};

use datatest_stable::Result;
use luau_builder::LuauBuilder;
use luau_printer::{
	LuauPrinter,
	library::{LibraryPrinter, LibrarySections, NamesFinder},
};
use wast::{
	QuoteWat, WastArg, WastExecute, WastInvoke, WastRet, WastThread, Wat,
	core::{NanPattern, WastArgCore, WastRetCore},
	token::{F32, F64, Id, Span},
};

use common::{compiler::Compiler, process, visitor::Visitor};

mod common;

const HARNESS_START_SOURCE: &str = include_str!("harness/luau.start.luau");
const HARNESS_END_SOURCE: &str = include_str!("harness/luau.end.luau");

struct Luau {
	library_sections: LibrarySections,
	library_printer: LibraryPrinter,
	references: Vec<&'static str>,

	compiler: Compiler,
	builder: LuauBuilder,
	printer: LuauPrinter,

	file: Vec<u8>,
}

impl Luau {
	fn new() -> Self {
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
		let graph = self.compiler.run(data);
		let tree = self.builder.run(&graph);

		NamesFinder::new(&mut self.references).run(&tree);

		self.printer.indent();
		self.printer.print(&tree, &mut self.file)?;
		self.printer.outdent();

		Ok(())
	}

	fn fmt_name(&mut self, id: Id) -> Result<()> {
		let identifier = id.name().as_bytes().escape_ascii();

		write!(self.file, "named[\"{identifier}\"]")?;

		Ok(())
	}

	fn fmt_optional_name(&mut self, id: Option<Id>) -> Result<()> {
		if let Some(id) = id {
			self.fmt_name(id)?;
		} else {
			write!(self.file, "selected")?;
		}

		Ok(())
	}

	fn fmt_named_source(&mut self, id: Option<Id>, data: &[u8]) -> Result<()> {
		self.fmt_source(data)?;

		writeln!(self.file, "\tselected = module(environment)")?;

		if let Some(id) = id {
			self.fmt_name(id)?;

			writeln!(self.file, " = selected")?;
		}

		Ok(())
	}

	fn fmt_argument_i32(&mut self, source: i32) -> Result<()> {
		let source = u32::from_ne_bytes(source.to_ne_bytes());

		write!(self.file, "{source} --[[ 0x{source:08X} ]]")?;

		Ok(())
	}

	fn fmt_argument_i64(&mut self, source: i64) -> Result<()> {
		let [b1, b2, b3, b4, b5, b6, b7, b8] = source.to_le_bytes();

		let source_1 = u32::from_le_bytes([b1, b2, b3, b4]);
		let source_2 = u32::from_le_bytes([b5, b6, b7, b8]);

		self.references.push("into_bits_i64");

		write!(
			self.file,
			"into_bits_i64({source_1}, {source_2}) --[[ 0x{source:016X} ]]"
		)?;

		Ok(())
	}

	fn fmt_argument_f32(&mut self, source: F32) -> Result<()> {
		let F32 { bits } = source;
		let source = f32::from_bits(bits);

		write!(self.file, "{bits} --[[ {source}_f32 ]]")?;

		Ok(())
	}

	fn fmt_argument_f64(&mut self, source: F64) -> Result<()> {
		let F64 { bits } = source;
		let source = f64::from_bits(bits);

		let [b1, b2, b3, b4, b5, b6, b7, b8] = bits.to_le_bytes();

		let source_1 = u32::from_le_bytes([b1, b2, b3, b4]);
		let source_2 = u32::from_le_bytes([b5, b6, b7, b8]);

		self.references.push("into_bits_i64");

		write!(
			self.file,
			"into_bits_i64({source_1}, {source_2}) --[[ {source}_f64 ]]"
		)?;

		Ok(())
	}

	fn fmt_argument(&mut self, argument: WastArg) -> Result<()> {
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

	fn fmt_argument_list(&mut self, arguments: Vec<WastArg>) -> Result<()> {
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

	fn fmt_export(&mut self, module: Option<Id>, identifier: &str) -> Result<()> {
		let identifier = identifier.as_bytes().escape_ascii();

		self.fmt_optional_name(module)?;

		write!(self.file, "[\"{identifier}\"]")?;

		Ok(())
	}

	fn fmt_invoke(&mut self, invoke: WastInvoke) -> Result<()> {
		self.fmt_export(invoke.module, invoke.name)?;

		write!(self.file, "(")?;

		self.fmt_argument_list(invoke.args)?;

		write!(self.file, ")")?;

		Ok(())
	}

	fn fmt_wat(&mut self, mut wat: Wat) -> Result<()> {
		self.fmt_source(&wat.encode()?)?;

		writeln!(self.file, "module(environment)")?;

		Ok(())
	}

	fn fmt_get(&mut self, module: Option<Id>, global: &str) -> Result<()> {
		self.fmt_export(module, global)?;

		write!(self.file, "[1]")?;

		Ok(())
	}

	fn fmt_execute(&mut self, exec: WastExecute) -> Result<()> {
		match exec {
			WastExecute::Invoke(wast_invoke) => self.fmt_invoke(wast_invoke),
			WastExecute::Wat(wat) => self.fmt_wat(wat),
			WastExecute::Get { module, global, .. } => self.fmt_get(module, global),
		}
	}

	fn fmt_assert_equal_i32(&mut self, source: i32) -> Result<()> {
		let source = u32::from_ne_bytes(source.to_ne_bytes());

		self.references.push("assert_equal_i32");

		write!(
			self.file,
			"hn_assert_equal_i32({source}) --[[ 0x{source:08X} ]]"
		)?;

		Ok(())
	}

	fn fmt_assert_equal_i64(&mut self, source: i64) -> Result<()> {
		let [b1, b2, b3, b4, b5, b6, b7, b8] = source.to_le_bytes();

		let source_1 = u32::from_le_bytes([b1, b2, b3, b4]);
		let source_2 = u32::from_le_bytes([b5, b6, b7, b8]);

		self.references.push("assert_equal_i64");
		self.references.push("into_bits_i64");

		write!(
			self.file,
			"hn_assert_equal_i64(into_bits_i64({source_1}, {source_2})) --[[ 0x{source:016X} ]]"
		)?;

		Ok(())
	}

	fn fmt_assert_equal_f32(&mut self, source: F32) -> Result<()> {
		let F32 { bits } = source;
		let source = f32::from_bits(bits);

		self.references.push("assert_equal_f32");

		write!(
			self.file,
			"hn_assert_equal_f32({bits}) --[[ {source}_f32 ]]"
		)?;

		Ok(())
	}

	fn fmt_assert_equal_f64(&mut self, source: F64) -> Result<()> {
		let F64 { bits } = source;
		let source = f64::from_bits(bits);

		let [b1, b2, b3, b4, b5, b6, b7, b8] = bits.to_le_bytes();

		let source_1 = u32::from_le_bytes([b1, b2, b3, b4]);
		let source_2 = u32::from_le_bytes([b5, b6, b7, b8]);

		self.references.push("assert_equal_f64");
		self.references.push("into_bits_i64");

		write!(
			self.file,
			"hn_assert_equal_f64(into_bits_i64({source_1}, {source_2})) --[[ {source}_f64 ]]"
		)?;

		Ok(())
	}

	fn fmt_assert_pattern(&mut self, result: WastRet) -> Result<()> {
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

impl Visitor for Luau {
	fn visit_module(&mut self, mut quote_wat: QuoteWat) -> Result<()> {
		let data = quote_wat.encode()?;

		writeln!(self.file, "do")?;

		self.fmt_named_source(quote_wat.name(), &data)?;

		writeln!(self.file, "end")?;

		Ok(())
	}

	fn visit_module_definition(&mut self, _quote_wat: QuoteWat) -> Result<()> {
		unimplemented!()
	}

	fn visit_module_instance(
		&mut self,
		_span: Span,
		_instance: Option<Id>,
		_module: Option<Id>,
	) -> Result<()> {
		unimplemented!()
	}

	fn visit_assert_malformed(
		&mut self,
		_span: Span,
		_module: QuoteWat,
		_message: &str,
	) -> Result<()> {
		Ok(())
	}

	fn visit_assert_invalid(
		&mut self,
		_span: Span,
		_module: QuoteWat,
		_message: &str,
	) -> Result<()> {
		Ok(())
	}

	fn visit_register(&mut self, _span: Span, name: &str, module: Option<Id>) -> Result<()> {
		let name = name.as_bytes().escape_ascii();

		write!(self.file, "environment[\"{name}\"] = ")?;

		self.fmt_optional_name(module)?;

		writeln!(self.file)?;

		Ok(())
	}

	fn visit_invoke(&mut self, wast_invoke: WastInvoke) -> Result<()> {
		self.fmt_invoke(wast_invoke)?;

		writeln!(self.file)?;

		Ok(())
	}

	fn visit_assert_trap(&mut self, _span: Span, exec: WastExecute, message: &str) -> Result<()> {
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
		exec: WastExecute,
		results: Vec<WastRet>,
	) -> Result<()> {
		writeln!(self.file, "do")?;
		write!(self.file, "\tlocal sources = {{ ")?;

		self.fmt_execute(exec)?;

		writeln!(self.file, " }}")?;

		for (result, index) in results.into_iter().zip(1..) {
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
		_call: WastInvoke,
		_message: &str,
	) -> Result<()> {
		Ok(())
	}

	fn visit_assert_unlinkable(&mut self, _span: Span, _module: Wat, _message: &str) -> Result<()> {
		Ok(())
	}

	fn visit_assert_exception(&mut self, _span: Span, _exec: WastExecute) -> Result<()> {
		Ok(())
	}

	fn visit_assert_suspension(
		&mut self,
		_span: Span,
		_exec: WastExecute,
		_message: &str,
	) -> Result<()> {
		Ok(())
	}

	fn visit_thread(&mut self, _wast_thread: WastThread) -> Result<()> {
		Ok(())
	}

	fn visit_wait(&mut self, _span: Span, _thread: Id) -> Result<()> {
		Ok(())
	}
}

fn get_path_target(name: &OsStr, optimized: bool, native: bool) -> Result<PathBuf> {
	let mut path = PathBuf::new();

	path.push(env!("CARGO_TARGET_TMPDIR"));
	path.push(if native { "native" } else { "interpreter" });
	path.push(if optimized { "O2" } else { "O0" });

	std::fs::create_dir_all(&path)?;

	path.push(name);
	path.set_extension("luau");

	Ok(path)
}

fn compile_test(destination: &Path, source: &str) -> Result<()> {
	let mut luau = Luau::new();

	luau.visit(source)?;

	let mut destination = File::create(destination).map(BufWriter::new)?;

	luau.write_into(&mut destination)?;

	destination.flush()?;

	Ok(())
}

fn run_file(destination: &Path, optimized: bool, native: bool) -> Result<Box<str>> {
	let mut arguments = vec![OsStr::new(if optimized { "-O2" } else { "-O0" })];

	if native {
		arguments.push(OsStr::new("--codegen"));
	}

	arguments.push(destination.as_ref());

	let program = std::env::var_os("LUAU_PATH").unwrap_or_else(|| "luau".into());
	let output = process::run(&program, &arguments)?;

	Ok(output)
}

fn run_and_assert(path: &Path, optimized: bool, native: bool) -> Result<()> {
	let source = std::fs::read_to_string(path)?;
	let destination = get_path_target(path.file_name().unwrap(), optimized, native)?;

	compile_test(&destination, &source)?;

	let output = run_file(&destination, optimized, native)?;

	assert!(output.is_empty(), "{output}");

	Ok(())
}

fn bytecode_o0(path: &Path) -> Result<()> {
	crate::run_and_assert(path, false, false)
}

fn bytecode_o2(path: &Path) -> Result<()> {
	crate::run_and_assert(path, true, false)
}

fn native_o0(path: &Path) -> Result<()> {
	crate::run_and_assert(path, false, true)
}

fn native_o2(path: &Path) -> Result<()> {
	crate::run_and_assert(path, true, true)
}

datatest_stable::harness! {
	{ test = bytecode_o0, root = "Suite", pattern = r"^(?!simd_)\w+\.wast$" },
	{ test = bytecode_o2, root = "Suite", pattern = r"^(?!simd_)\w+\.wast$" },
	{ test = native_o0, root = "Suite", pattern = r"^(?!simd_)\w+\.wast$" },
	{ test = native_o2, root = "Suite", pattern = r"^(?!simd_)\w+\.wast$" },
}
