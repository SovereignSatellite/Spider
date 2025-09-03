use std::{
	fs::File,
	io::{Read, Write},
	path::{Path, PathBuf},
	process::{Child, Command, ExitStatus, Stdio},
	time::{Duration, Instant},
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

use self::common::{loader::Loader, runner::Runner};

mod common;

const LUAU_TIMEOUT: Duration = Duration::from_secs(1);

const HARNESS_SOURCE: &str = include_str!("harness.luau");

struct Luau {
	file: Vec<u8>,

	library_sections: LibrarySections,
	library_printer: LibraryPrinter,
	references: Vec<&'static str>,

	loader: Loader,
	luau_builder: LuauBuilder,
	luau_printer: LuauPrinter,
}

impl Luau {
	fn new() -> Self {
		let mut library_sections = LibrarySections::with_built_ins();

		library_sections.parse_from(HARNESS_SOURCE);
		library_sections.resolve();

		Self {
			file: Vec::new(),

			library_sections,
			library_printer: LibraryPrinter::new(),
			references: Vec::new(),

			loader: Loader::new(),
			luau_builder: LuauBuilder::new(),
			luau_printer: LuauPrinter::new(),
		}
	}

	fn write_into(mut self, path: &Path) -> Result<()> {
		let end = self.file.len();

		self.references.push("environment");

		self.references.sort_unstable();
		self.references.dedup();

		self.library_printer
			.resolve(&self.references, &self.library_sections);

		self.library_printer
			.print(&self.library_sections, &mut self.file)?;

		let (source, library) = self.file.split_at(end);

		let mut file = File::create(path)?;

		file.write_all(library)?;
		file.write_all(source)?;

		Ok(())
	}

	fn fmt_source(&mut self, data: &[u8]) -> Result<()> {
		let graph = self.loader.run(data);
		let tree = self.luau_builder.run(&graph);

		NamesFinder::new(&mut self.references).run(&tree);

		self.luau_printer.indent();
		self.luau_printer.print(&tree, &mut self.file)?;
		self.luau_printer.outdent();

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
		let source_1 = u32::from_le_bytes(source.to_le_bytes());

		write!(self.file, "0x{source_1:08X} --[[ {source}_i32 ]]")?;

		Ok(())
	}

	fn fmt_argument_i64(&mut self, source: i64) -> Result<()> {
		let [b1, b2, b3, b4, b5, b6, b7, b8] = source.to_le_bytes();

		let source_1 = u32::from_le_bytes([b1, b2, b3, b4]);
		let source_2 = u32::from_le_bytes([b5, b6, b7, b8]);

		self.references.push("create_i64_from_u32");

		write!(
			self.file,
			"rt_create_i64_from_u32(0x{source_1:08X}, 0x{source_2:08X}) --[[ {source}_i64 ]]"
		)?;

		Ok(())
	}

	fn fmt_argument_f32(&mut self, source: F32) -> Result<()> {
		let F32 { bits } = source;
		let source = f32::from_bits(bits);

		self.references.push("transmute_i32_to_f32");

		write!(
			self.file,
			"rt_transmute_i32_to_f32(0x{bits:08X}) --[[ {source}_f32 ]]",
		)?;

		Ok(())
	}

	fn fmt_argument_f64(&mut self, source: F64) -> Result<()> {
		let F64 { bits } = source;
		let source = f64::from_bits(bits);

		let [b1, b2, b3, b4, b5, b6, b7, b8] = bits.to_le_bytes();

		let source_1 = u32::from_le_bytes([b1, b2, b3, b4]);
		let source_2 = u32::from_le_bytes([b5, b6, b7, b8]);

		self.references.push("create_f64_from_u32");

		write!(
			self.file,
			"rt_create_f64_from_u32(0x{source_1:08X}, 0x{source_2:08X}) --[[ {source}_f64 ]]"
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
		let source_1 = u32::from_le_bytes(source.to_le_bytes());

		self.references.push("assert_equal_i32");

		write!(
			self.file,
			"hn_assert_equal_i32(0x{source_1:08X}) --[[ {source}_i32 ]]"
		)?;

		Ok(())
	}

	fn fmt_assert_equal_i64(&mut self, source: i64) -> Result<()> {
		let [b1, b2, b3, b4, b5, b6, b7, b8] = source.to_le_bytes();

		let source_1 = u32::from_le_bytes([b1, b2, b3, b4]);
		let source_2 = u32::from_le_bytes([b5, b6, b7, b8]);

		self.references.push("assert_equal_i64");

		write!(
			self.file,
			"hn_assert_equal_i64(0x{source_1:08X}, 0x{source_2:08X}) --[[ {source}_i64 ]]"
		)?;

		Ok(())
	}

	fn fmt_assert_equal_f32(&mut self, source: F32) -> Result<()> {
		let F32 { bits } = source;
		let source = f32::from_bits(bits);

		self.references.push("assert_equal_f32");

		write!(
			self.file,
			"hn_assert_equal_f32(0x{bits:08X}) --[[ {source}_f32 ]]"
		)?;

		Ok(())
	}

	fn fmt_assert_equal_f64(&mut self, source: F64) -> Result<()> {
		let F64 { bits } = source;
		let source = f64::from_bits(bits);

		let [b1, b2, b3, b4, b5, b6, b7, b8] = source.to_le_bytes();

		let source_1 = u32::from_le_bytes([b1, b2, b3, b4]);
		let source_2 = u32::from_le_bytes([b5, b6, b7, b8]);

		self.references.push("assert_equal_f64");

		write!(
			self.file,
			"hn_assert_equal_f64(0x{source_1:08X}, 0x{source_2:08X}) --[[ {source}_f64 ]]"
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

impl Runner for Luau {
	fn on_module(&mut self, mut quote_wat: QuoteWat) -> Result<()> {
		let data = quote_wat.encode()?;

		writeln!(self.file, "do")?;

		self.fmt_named_source(quote_wat.name(), &data)?;

		writeln!(self.file, "end")?;

		Ok(())
	}

	fn on_module_definition(&mut self, _quote_wat: QuoteWat) -> Result<()> {
		unimplemented!()
	}

	fn on_module_instance(
		&mut self,
		_span: Span,
		_instance: Option<Id>,
		_module: Option<Id>,
	) -> Result<()> {
		unimplemented!()
	}

	fn on_assert_malformed(
		&mut self,
		_span: Span,
		_module: QuoteWat,
		_message: &str,
	) -> Result<()> {
		Ok(())
	}

	fn on_assert_invalid(&mut self, _span: Span, _module: QuoteWat, _message: &str) -> Result<()> {
		Ok(())
	}

	fn on_register(&mut self, _span: Span, name: &str, module: Option<Id>) -> Result<()> {
		let name = name.as_bytes().escape_ascii();

		write!(self.file, "environment[\"{name}\"] = ")?;

		self.fmt_optional_name(module)?;

		writeln!(self.file)?;

		Ok(())
	}

	fn on_invoke(&mut self, wast_invoke: WastInvoke) -> Result<()> {
		self.fmt_invoke(wast_invoke)?;

		writeln!(self.file)?;

		Ok(())
	}

	fn on_assert_trap(&mut self, _span: Span, exec: WastExecute, message: &str) -> Result<()> {
		let message = message.as_bytes().escape_ascii();

		self.references.push("assert_trap");

		writeln!(self.file, "hn_assert_trap(\"{message}\", function()")?;

		self.fmt_execute(exec)?;

		writeln!(self.file, "\nend)")?;

		Ok(())
	}

	fn on_assert_return(
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

	fn on_assert_exhaustion(
		&mut self,
		_span: Span,
		_call: WastInvoke,
		_message: &str,
	) -> Result<()> {
		Ok(())
	}

	fn on_assert_unlinkable(&mut self, _span: Span, _module: Wat, _message: &str) -> Result<()> {
		Ok(())
	}

	fn on_assert_exception(&mut self, _span: Span, _exec: WastExecute) -> Result<()> {
		Ok(())
	}

	fn on_assert_suspension(
		&mut self,
		_span: Span,
		_exec: WastExecute,
		_message: &str,
	) -> Result<()> {
		Ok(())
	}

	fn on_thread(&mut self, _wast_thread: WastThread) -> Result<()> {
		Ok(())
	}

	fn on_wait(&mut self, _span: Span, _thread: Id) -> Result<()> {
		Ok(())
	}
}

fn load_output_path(path: &Path) -> PathBuf {
	const TEMP_DIRECTORY: &str = env!("CARGO_TARGET_TMPDIR");

	let name = path.file_name().expect("should have file name");

	Path::new(TEMP_DIRECTORY).join(name).with_extension("luau")
}

fn poll_until_timeout(child: &mut Child, duration: Duration) -> Result<ExitStatus> {
	let now = Instant::now();

	while now.elapsed() < duration {
		std::thread::yield_now();

		if let Some(status) = child.try_wait()? {
			return Ok(status);
		}
	}

	child.kill()?;

	Err(std::io::Error::new(
		std::io::ErrorKind::TimedOut,
		"the sub-process has timed out",
	)
	.into())
}

fn run_and_verify(path: &Path) -> Result<()> {
	let luau = std::env::var_os("LUAU_PATH").ok_or("`LUAU_PATH` should be set")?;
	let mut child = Command::new(luau)
		.arg(path)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;

	if poll_until_timeout(&mut child, LUAU_TIMEOUT)?.success() {
		Ok(())
	} else {
		let Child { stdout, stderr, .. } = child;

		let mut result = String::new();

		stdout.unwrap().read_to_string(&mut result)?;
		result.push('\n');
		stderr.unwrap().read_to_string(&mut result)?;

		panic!("{result}");
	}
}

fn luau(path: &Path) -> Result<()> {
	let output = load_output_path(path);
	let test = std::fs::read_to_string(path)?;

	// SAFETY: I'm not sure, but it's not a problem in practice.
	unsafe {
		std::env::set_var("RUST_BACKTRACE", "1");
	}

	let mut luau = Luau::new();

	luau.run(&test)?;
	luau.write_into(&output)?;

	run_and_verify(&output)
}

datatest_stable::harness! {
	{ test = luau, root = "Suite", pattern = r"^(?!simd_)\w+\.wast$" },
}
