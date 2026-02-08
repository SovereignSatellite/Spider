use std::io::{BufWriter, StdoutLock, Write};

use clap::Parser;
use data_flow_builder::DataFlowBuilder;
use data_flow_graph::{DataFlowGraph, Link};
use data_flow_visitor::{
	control::{
		dead_port_eliminator::DeadPortEliminator, invariant_port_mover::InvariantPortMover,
		region_identity,
	},
	isle,
	topological_normalizer::TopologicalNormalizer,
};
use luajit_builder::LuaJITBuilder;
use luajit_printer::{
	LuaJITPrinter,
	library::{LibraryPrinter, LibrarySections, NamesFinder},
};
use luajit_tree::LuaJITTree;
use wasmparser::Validator;

#[derive(Parser)]
#[command(version)]
struct Arguments {
	/// The WebAssembly file for processing
	file: String,

	/// Embed debug information if present
	#[arg(long, short)]
	debug: bool,

	/// Run all optimization passes on code
	#[arg(long, short)]
	optimize: bool,
}

fn run_isle_optimizations(graph: &mut DataFlowGraph) -> bool {
	let mut applied = false;
	let len = graph.len();

	for id in (0..len.try_into().unwrap()).rev() {
		while isle::simplify_i32(graph, id)
			|| isle::simplify_global(graph, id)
			|| isle::simplify_table(graph, id)
		{
			applied = true;
		}
	}

	applied
}

fn run_all_optimizations(graph: &mut DataFlowGraph, mut omega: u32) -> u32 {
	let mut topological_normalizer = TopologicalNormalizer::new();
	let mut invariant_port_mover = InvariantPortMover::new();
	let mut dead_port_eliminator = DeadPortEliminator::new();

	loop {
		omega = topological_normalizer.run(graph, omega);

		invariant_port_mover.run(graph);
		dead_port_eliminator.run(graph, Link(omega, 0));

		if !run_isle_optimizations(graph) {
			break;
		}

		region_identity::remove(graph);
	}

	omega
}

fn run_post_process(graph: &mut DataFlowGraph, omega: u32) {
	let mut topological_normalizer = TopologicalNormalizer::new();

	region_identity::insert(graph);

	topological_normalizer.run(graph, omega);
}

fn build_data_flow_graph(data: &[u8], optimize: bool) -> DataFlowGraph {
	let mut graph = DataFlowGraph::new();
	let mut builder = DataFlowBuilder::new();

	let omega = builder.run(&mut graph, data);
	let omega = if optimize {
		run_all_optimizations(&mut graph, omega)
	} else {
		omega
	};

	run_post_process(&mut graph, omega);

	graph
}

fn build_luajit_tree(graph: &DataFlowGraph) -> LuaJITTree {
	let mut builder = LuaJITBuilder::new();

	builder.run(graph)
}

fn lock_standard_output() -> BufWriter<StdoutLock<'static>> {
	const DEFAULT_BUF_SIZE: usize = 1024 * 1024;

	BufWriter::with_capacity(DEFAULT_BUF_SIZE, std::io::stdout().lock())
}

fn print_luajit_library(tree: &LuaJITTree) -> std::io::Result<()> {
	let sections = LibrarySections::with_built_ins();
	let mut printer = LibraryPrinter::new();
	let mut references = Vec::new();

	NamesFinder::new(&mut references).run(tree);

	printer.resolve(&references, &sections);

	let mut output = lock_standard_output();

	printer.print(&sections, &mut output)?;
	output.flush()
}

fn print_luajit_tree(tree: &LuaJITTree) -> std::io::Result<()> {
	let mut printer = LuaJITPrinter::new();
	let mut output = lock_standard_output();

	printer.print(tree, &mut output)?;
	output.flush()
}

fn main() {
	let arguments = Arguments::parse();
	let data = std::fs::read(arguments.file).unwrap();

	Validator::new()
		.validate_all(&data)
		.expect("`file` should be a WebAssembly binary");

	let graph = build_data_flow_graph(&data, arguments.optimize);
	let tree = build_luajit_tree(&graph);

	print_luajit_library(&tree).expect("library should print");
	print_luajit_tree(&tree).expect("source should print");
}
