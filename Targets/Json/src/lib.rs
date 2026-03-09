//! JSON printer for data flow graphs.

extern crate alloc;

mod color;
mod interner;
mod label;
mod print;

use std::io::{Result, Write};

use ir_graph::{
	DataFlowGraph, Node,
	control::{GammaOut, LambdaIn, OmegaIn, RegionIn, ThetaIn},
};

use crate::{color::Color, interner::Interner, print::Print as _};

const fn should_skip_node(node: &Node) -> bool {
	matches!(
		node,
		Node::OmegaOut(_)
			| Node::LambdaOut(_)
			| Node::RegionIn(_)
			| Node::RegionOut(_)
			| Node::GammaOut(_)
			| Node::ThetaOut(_)
	)
}

const fn region_to_subgraph(node: &Node, id: u32) -> Option<(u32, u32, u32)> {
	let graph = match *node {
		Node::LambdaIn(LambdaIn { output, .. })
		| Node::ThetaIn(ThetaIn { output, .. })
		| Node::OmegaIn(OmegaIn { output }) => (id, id, output),

		Node::RegionIn(RegionIn { input, output }) => (input, id, output),

		Node::Apply(_)
		| Node::F32(_)
		| Node::F64(_)
		| Node::Fence(_)
		| Node::GammaIn(_)
		| Node::GammaOut(_)
		| Node::GlobalGet(_)
		| Node::GlobalNew(_)
		| Node::GlobalSet(_)
		| Node::Host(_)
		| Node::I32(_)
		| Node::I64(_)
		| Node::Identity(_)
		| Node::Import(_)
		| Node::IntegerBinaryOperation(_)
		| Node::IntegerCompareOperation(_)
		| Node::IntegerConvertToNumber(_)
		| Node::IntegerExtend(_)
		| Node::IntegerNarrow(_)
		| Node::IntegerTransmuteToNumber(_)
		| Node::IntegerUnaryOperation(_)
		| Node::IntegerWiden(_)
		| Node::LambdaOut(_)
		| Node::MemoryCopy(_)
		| Node::MemoryDrop(_)
		| Node::MemoryFill(_)
		| Node::MemoryGrow(_)
		| Node::MemoryLoad(_)
		| Node::MemoryNew(_)
		| Node::MemorySize(_)
		| Node::MemoryStore(_)
		| Node::Null
		| Node::NumberBinaryOperation(_)
		| Node::NumberCompareOperation(_)
		| Node::NumberNarrow(_)
		| Node::NumberTransmuteToInteger(_)
		| Node::NumberTruncateToInteger(_)
		| Node::NumberUnaryOperation(_)
		| Node::NumberWiden(_)
		| Node::OmegaOut(_)
		| Node::RefIsNull(_)
		| Node::RegionOut(_)
		| Node::TableCopy(_)
		| Node::TableDrop(_)
		| Node::TableFill(_)
		| Node::TableGet(_)
		| Node::TableGrow(_)
		| Node::TableNew(_)
		| Node::TableSet(_)
		| Node::TableSize(_)
		| Node::ThetaOut(_)
		| Node::Trap => return None,
	};

	Some(graph)
}

/// A JSON printer for data flow graphs.
pub struct JsonPrinter {
	subgraphs: Vec<u32>,
	nodes: Vec<u32>,
	edges: Vec<u32>,

	scratch: Vec<u8>,
	interner: Interner,
}

impl JsonPrinter {
	/// Creates a new JSON printer.
	#[must_use]
	pub fn new() -> Self {
		Self {
			subgraphs: Vec::new(),
			nodes: Vec::new(),
			edges: Vec::new(),

			scratch: Vec::new(),
			interner: Interner::new(),
		}
	}

	fn get_node_label(&mut self, node: &Node) -> u32 {
		let name = label::get_static(node).unwrap_or_else(|| {
			self.scratch.clear();

			label::write(node, &mut self.scratch).unwrap();
			core::str::from_utf8(&self.scratch).unwrap()
		});

		self.interner.resolve(name)
	}

	fn find_subgraphs(&mut self, graph: &DataFlowGraph) {
		self.subgraphs.clear();

		for (node, id) in graph.nodes().zip(0..) {
			if let Some((node, input, output)) = region_to_subgraph(node, id) {
				self.subgraphs.push(node);
				self.subgraphs.push(input);
				self.subgraphs.push(output);
			}
		}
	}

	fn find_nodes(&mut self, graph: &DataFlowGraph) {
		self.nodes.clear();

		for (node, id) in graph.nodes().zip(0..) {
			if should_skip_node(node) {
				continue;
			}

			let color = Color::from_reference(node).as_string();
			let color = self.interner.resolve(color);
			let name = self.get_node_label(node);

			self.nodes.push(id);
			self.nodes.push(name);
			self.nodes.push(color);
		}
	}

	fn find_edges(&mut self, graph: &DataFlowGraph) {
		self.edges.clear();

		for (node, id) in graph.nodes().zip(0..) {
			let mut port = 0;

			node.for_each_argument(|link| {
				self.edges.push(link.0);
				self.edges.push(link.1.into());
				self.edges.push(id);
				self.edges.push(port);

				port += 1;
			});
		}

		for edge in self.edges.chunks_exact_mut(4) {
			if let Node::GammaOut(GammaOut { input, .. }) = *graph.get(edge[0]) {
				edge[0] = input;
			}
		}
	}

	fn find_all_fields(&mut self, graph: &DataFlowGraph) {
		self.interner.clear();

		self.find_subgraphs(graph);
		self.find_nodes(graph);
		self.find_edges(graph);
	}

	fn print_all_fields(&self, out: &mut dyn Write) -> Result<()> {
		write!(out, "{{")?;

		("subgraphs", &self.subgraphs).print(out)?;
		write!(out, ",")?;

		("nodes", &self.nodes).print(out)?;
		write!(out, ",")?;

		("edges", &self.edges).print(out)?;
		write!(out, ",")?;

		("strings", self.interner.list()).print(out)?;

		write!(out, "}}")
	}

	/// Prints the graph as JSON to the given writer.
	///
	/// # Errors
	///
	/// Returns an error if writing to the output fails.
	pub fn print(&mut self, graph: &DataFlowGraph, out: &mut dyn Write) -> Result<()> {
		self.find_all_fields(graph);
		self.print_all_fields(out)
	}
}

impl Default for JsonPrinter {
	fn default() -> Self {
		Self::new()
	}
}
