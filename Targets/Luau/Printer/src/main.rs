use std::io::{BufWriter, StdoutLock, Write};

use clap::Parser;
use data_flow_builder::DataFlowBuilder;
use data_flow_graph::{DataFlowGraph, Link};
use data_flow_visitor::{
	dead_port_eliminator::DeadPortEliminator, fallthrough_mover::FallthroughMover, region_identity,
	topological_normalizer::TopologicalNormalizer,
};
use luau_builder::LuauBuilder;
use luau_printer::{
	library::{LibraryPrinter, LibrarySections, NamesFinder},
	LuauPrinter,
};
use luau_tree::LuauTree;
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

fn run_optimizations(graph: &mut DataFlowGraph, omega: u32) -> u32 {
	let mut topological_normalizer = TopologicalNormalizer::new();

	let omega = topological_normalizer.run(graph, omega);

	let mut fallthrough_mover = FallthroughMover::new();

	fallthrough_mover.run(graph);

	let mut dead_port_eliminator = DeadPortEliminator::new();

	dead_port_eliminator.run(graph, Link(omega, 0));

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

	let omega = builder.run(&mut graph, &data);
	let omega = if optimize {
		run_optimizations(&mut graph, omega)
	} else {
		omega
	};

	run_post_process(&mut graph, omega);

	graph
}

fn build_luau_tree(graph: &DataFlowGraph) -> LuauTree {
	let mut builder = LuauBuilder::new();

	builder.run(graph)
}

fn lock_standard_output() -> BufWriter<StdoutLock<'static>> {
	const DEFAULT_BUF_SIZE: usize = 1024 * 1024 * 1;

	BufWriter::with_capacity(DEFAULT_BUF_SIZE, std::io::stdout().lock())
}

fn print_luau_library(tree: &LuauTree) -> std::io::Result<()> {
	let sections = LibrarySections::with_built_ins();
	let mut printer = LibraryPrinter::new();
	let mut references = Vec::new();

	NamesFinder::new(&mut references).run(tree);

	printer.resolve(&references, &sections);

	let mut output = lock_standard_output();

	printer.print(&sections, &mut output)?;
	output.flush()
}

fn print_luau_tree(tree: &LuauTree) -> std::io::Result<()> {
	let mut printer = LuauPrinter::new();
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
	let tree = build_luau_tree(&graph);

	print_luau_library(&tree).expect("library should print");
	print_luau_tree(&tree).expect("source should print");
}
