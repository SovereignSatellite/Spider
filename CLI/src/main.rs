use std::io::{BufWriter, StdoutLock, Write};

use clap::Parser;
use ir_graph::DataFlowGraph;

use crate::{
	arguments::{Arguments, Source, Target},
	common::{run_all_optimizations, run_post_process},
};

mod arguments;
mod common;
mod sources;
mod targets;

fn lock_standard_output() -> BufWriter<StdoutLock<'static>> {
	const DEFAULT_BUF_SIZE: usize = 1024 * 1024;

	BufWriter::with_capacity(DEFAULT_BUF_SIZE, std::io::stdout().lock())
}

fn build_graph(data: &[u8], optimize: bool, source: Source) -> DataFlowGraph {
	let (mut graph, mut omega) = match source {
		Source::TuringMachine => sources::from_turing_machine(data),
		Source::WebAssembly => sources::from_web_assembly(data),
	};

	if optimize {
		omega = run_all_optimizations(&mut graph, omega);
	}

	run_post_process(&mut graph, omega);

	graph
}

fn print_graph(graph: &DataFlowGraph, target: Target) {
	let mut output = lock_standard_output();

	match target {
		Target::Dot => targets::into_dot(graph, &mut output),
		Target::Luau => targets::into_luau(graph, &mut output),
		Target::LuaJIT => targets::into_luajit(graph, &mut output),
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
	let graph = build_graph(&data, optimize, source);

	print_graph(&graph, target);
}
