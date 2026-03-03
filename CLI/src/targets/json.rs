use std::io::Write;

use ir_graph::DataFlowGraph;
use json_printer::JsonPrinter;

pub fn print(graph: &DataFlowGraph, out: &mut dyn Write) {
	let mut printer = JsonPrinter::new();

	printer.print(graph, out).expect("graph should print");
}
