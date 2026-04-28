use std::{io::Write, sync::Arc};

use parking_lot::Mutex;

use ir_graph::region::Function;
use luajit_builder::LuaJITBuilder;
use luajit_printer::{
	LuaJITPrinter,
	library::{NamesFinder, Printer as LibraryPrinter, Sections as LibrarySections},
};
use luajit_tree::expression;

fn build_function(root: &Arc<Mutex<Function>>) -> expression::Function {
	let mut builder = LuaJITBuilder::new();

	builder.run(root)
}

fn print_library(function: &expression::Function, out: &mut dyn Write) -> std::io::Result<()> {
	let mut printer = LibraryPrinter::new();
	let mut references = Vec::new();

	NamesFinder::new(&mut references).run(function);

	let sections = LibrarySections::with_built_ins();

	printer.resolve(&references, &sections);
	printer.print(&sections, out)?;
	out.flush()
}

fn print_function(function: &expression::Function, out: &mut dyn Write) -> std::io::Result<()> {
	let mut printer = LuaJITPrinter::new();

	printer.print(function, out)?;
	out.flush()
}

fn print_full_library(out: &mut dyn Write) -> std::io::Result<()> {
	let mut printer = LibraryPrinter::new();
	let sections = LibrarySections::with_built_ins();
	let references: Vec<_> = sections
		.as_slice()
		.iter()
		.map(|section| section.name)
		.collect();

	printer.resolve(&references, &sections);
	printer.print(&sections, out)?;
	out.flush()
}

pub fn print(root: &Arc<Mutex<Function>>, out: &mut dyn Write) {
	let function = build_function(root);

	print_library(&function, out).expect("library should print");
	print_function(&function, out).expect("source should print");
}

pub fn print_runtime(out: &mut dyn Write) {
	print_full_library(out).expect("library should print");
}
