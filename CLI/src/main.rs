use std::{
	io::{BufWriter, StdoutLock, Write as _},
	sync::Arc,
};

use clap::Parser as _;
use ir_pipeline::Optimizer;
use parking_lot::Mutex;

use ir_graph::region::Module;

use self::arguments::{Arguments, Source, Target};

mod arguments;
mod sources;
mod targets;

fn lock_standard_output() -> BufWriter<StdoutLock<'static>> {
	const DEFAULT_BUF_SIZE: usize = 1024 * 1024;

	BufWriter::with_capacity(DEFAULT_BUF_SIZE, std::io::stdout().lock())
}

fn build_module(data: &[u8], optimize: bool, source: Source) -> Arc<Mutex<Module>> {
	let module = match source {
		Source::TuringMachine => sources::from_turing_machine(data),
		Source::WebAssembly => sources::from_web_assembly(data),
	};

	Optimizer::new().run(&module, optimize);

	module
}

fn print_module(module: &Arc<Mutex<Module>>, target: Target) {
	let mut output = lock_standard_output();

	match target {
		Target::Json => targets::into_json(module, &mut output),
		Target::Luau => targets::into_luau(module, &mut output),
		Target::LuaJIT => targets::into_luajit(module, &mut output),
	}

	output.flush().expect("output should print");
}

fn main() {
	let Arguments {
		file,
		source,
		target,
		optimize,
	} = Arguments::parse();

	let data = std::fs::read(file).expect("failed to read file");
	let module = build_module(&data, optimize, source);

	print_module(&module, target);
}
