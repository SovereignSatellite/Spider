use std::{
	io::{BufWriter, StdoutLock, Write as _},
	sync::Arc,
};

use clap::Parser as _;
use ir_pipeline::Optimizer;
use parking_lot::Mutex;

use ir_graph::region::Function;

use self::arguments::{Arguments, Source, Target};

mod arguments;
mod sources;
mod targets;

fn lock_standard_output() -> BufWriter<StdoutLock<'static>> {
	const DEFAULT_BUFFER_SIZE: usize = 1024 * 1024;

	BufWriter::with_capacity(DEFAULT_BUFFER_SIZE, std::io::stdout().lock())
}

fn build_root(data: &[u8], should_optimize: bool, source: Source) -> Arc<Mutex<Function>> {
	let root = match source {
		Source::TuringMachine => sources::from_turing_machine(data),
		Source::WebAssembly => sources::from_web_assembly(data),
	};

	Optimizer::new().run(&root, should_optimize);

	root
}

fn print_root(root: &Arc<Mutex<Function>>, target: Target) {
	let mut output = lock_standard_output();

	match target {
		Target::Json => targets::into_json(root, &mut output),
		Target::Luau => targets::into_luau(root, &mut output),
		Target::LuaJIT => targets::into_luajit(root, &mut output),
	}

	output.flush().expect("output should print");
}

fn main() {
	let Arguments {
		file,
		source,
		target,
		should_optimize,
	} = Arguments::parse();

	let data = std::fs::read(file).expect("failed to read file");
	let root = build_root(&data, should_optimize, source);

	print_root(&root, target);
}
