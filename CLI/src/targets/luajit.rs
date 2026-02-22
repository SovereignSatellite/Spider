use std::io::Write;

use ir_graph::DataFlowGraph;
use luajit_builder::LuaJITBuilder;
use luajit_printer::{
	LuaJITPrinter,
	library::{LibraryPrinter, LibrarySections, NamesFinder},
};
use luajit_tree::LuaJITTree;

fn build_tree(graph: &DataFlowGraph) -> LuaJITTree {
	let mut builder = LuaJITBuilder::new();

	builder.run(graph)
}

fn print_library(tree: &LuaJITTree, out: &mut dyn Write) -> std::io::Result<()> {
	let mut printer = LibraryPrinter::new();
	let mut references = Vec::new();

	NamesFinder::new(&mut references).run(tree);

	let sections = LibrarySections::with_built_ins();

	printer.resolve(&references, &sections);
	printer.print(&sections, out)?;
	out.flush()
}

fn print_tree(tree: &LuaJITTree, out: &mut dyn Write) -> std::io::Result<()> {
	let mut printer = LuaJITPrinter::new();

	printer.print(tree, out)?;
	out.flush()
}

pub fn print(graph: &DataFlowGraph, out: &mut dyn Write) {
	let tree = build_tree(graph);

	print_library(&tree, out).expect("library should print");
	print_tree(&tree, out).expect("source should print");
}
