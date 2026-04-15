use std::{io::Write, sync::Arc};

use parking_lot::Mutex;

use ir_graph::control::Module;
use luau_builder::LuauBuilder;
use luau_printer::{
	LuauPrinter,
	library::{NamesFinder, Printer as LibraryPrinter, Sections as LibrarySections},
};
use luau_tree::LuauTree;

fn build_tree(module: &Arc<Mutex<Module>>) -> LuauTree {
	let mut builder = LuauBuilder::new();

	builder.run(module)
}

fn print_library(tree: &LuauTree, out: &mut dyn Write) -> std::io::Result<()> {
	let mut printer = LibraryPrinter::new();
	let mut references = Vec::new();

	NamesFinder::new(&mut references).run(tree);

	let sections = LibrarySections::with_built_ins();

	printer.resolve(&references, &sections);
	printer.print(&sections, out)?;
	out.flush()
}

fn print_tree(tree: &LuauTree, out: &mut dyn Write) -> std::io::Result<()> {
	let mut printer = LuauPrinter::new();

	printer.print(tree, out)?;
	out.flush()
}

pub fn print(module: &Arc<Mutex<Module>>, out: &mut dyn Write) {
	let tree = build_tree(module);

	print_library(&tree, out).expect("library should print");
	print_tree(&tree, out).expect("source should print");
}
