//! Successor relationship analysis.

use alloc::vec::Vec;
use ir_graph::{DataFlowGraph, Link};

/// A bidirectional link between two nodes.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct BiLink {
	/// The source node ID.
	pub from: u32,
	/// The source port index.
	pub port: u16,
	/// The destination node ID.
	pub to: u32,
}

/// Finds successor relationships in a data flow graph.
pub struct SuccessorFinder {
	successors: Vec<BiLink>,
}

impl SuccessorFinder {
	/// Creates a new empty successor finder.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			successors: Vec::new(),
		}
	}

	/// Returns the successors of a node by its ID.
	#[must_use]
	pub fn by_id(&self, id: u32) -> &[BiLink] {
		let start = self.successors.partition_point(|bi| bi.from < id);
		let end = self.successors.partition_point(|bi| bi.from <= id);

		&self.successors[start..end]
	}

	/// Returns the successors of a node by its link.
	#[must_use]
	pub fn by_link(&self, link: Link) -> &[BiLink] {
		let start = self
			.successors
			.partition_point(|bi| Link(bi.from, bi.port) < link);

		let end = self
			.successors
			.partition_point(|bi| Link(bi.from, bi.port) <= link);

		&self.successors[start..end]
	}

	/// Computes all successor relationships in the graph.
	pub fn run(&mut self, graph: &DataFlowGraph) {
		self.successors.clear();

		for (node, to) in graph.nodes().zip(0..) {
			node.for_each_argument(|Link(from, port)| {
				self.successors.push(BiLink { from, port, to });
			});
		}

		self.successors.sort_unstable();
	}
}

impl Default for SuccessorFinder {
	fn default() -> Self {
		Self::new()
	}
}
