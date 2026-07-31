use std::{
	io::{BufWriter, StdoutLock, Write as _},
	sync::Arc,
};

use clap::Parser as _;
use parking_lot::Mutex;

use ir_graph::region::Function;
use ir_pipeline::{OptimizationConfiguration, Optimizer};

use self::arguments::{Arguments, Command, CompileArguments, RuntimeTarget, Source, Target};

mod arguments;
mod sources;
mod targets;

fn lock_standard_output() -> BufWriter<StdoutLock<'static>> {
	const DEFAULT_BUFFER_SIZE: usize = 1024 * 1024;

	BufWriter::with_capacity(DEFAULT_BUFFER_SIZE, std::io::stdout().lock())
}

fn build_root(
	data: &[u8],
	configuration: &OptimizationConfiguration,
	source: Source,
	target: Target,
) -> Arc<Mutex<Function>> {
	let root = match source {
		Source::TuringMachine => sources::from_turing_machine(data),
		Source::WebAssembly => sources::from_web_assembly(data),
	};

	let mut optimizer = Optimizer::new();

	match target {
		Target::Luau => optimizer.run(&root, configuration, &mut luau_lower::apply),
		Target::LuaJIT => optimizer.run(&root, configuration, &mut luajit_lower::apply),
		Target::Json => optimizer.run(&root, configuration, &mut |_, _| false),
	}

	root
}

fn print_root(root: &Arc<Mutex<Function>>, target: Target) {
	let mut output = lock_standard_output();

	match target {
		Target::Json => targets::into_json(root, &mut output),
		Target::Luau => targets::into_luau(root, &mut output),
		Target::LuaJIT => targets::into_luajit(root, &mut output),
	}

	output.flush().expect("failed to flush compiled output");
}

fn compile(arguments: &CompileArguments) {
	let data = std::fs::read(&arguments.file).expect("failed to read the input file");
	let root = build_root(
		&data,
		&arguments.optimizations.configuration,
		arguments.source,
		arguments.target,
	);

	print_root(&root, arguments.target);
}

fn print_runtime(target: RuntimeTarget) {
	let mut output = lock_standard_output();

	match target {
		RuntimeTarget::Luau => targets::into_luau_runtime(&mut output),
		RuntimeTarget::LuaJIT => targets::into_luajit_runtime(&mut output),
	}

	output.flush().expect("failed to flush the runtime script");
}

fn main() {
	let arguments = Arguments::parse();

	match &arguments.command {
		Command::Compile(arguments) => compile(arguments),
		Command::Runtime(arguments) => print_runtime(arguments.target),
	}
}
