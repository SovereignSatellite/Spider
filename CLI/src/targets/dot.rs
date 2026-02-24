use std::io::Write;

use ir_graph::{DataFlowGraph, Dot};

pub fn print(graph: &DataFlowGraph, out: &mut dyn Write) {
	let dot = Dot::new(graph);

	writeln!(out, "{dot}").expect("graph should print");
}
