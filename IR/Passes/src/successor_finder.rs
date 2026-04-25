//! Successor relationship analysis.

use ir_graph::{Link, Node};

/// A directed edge from a producer port to a consumer node.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Successor {
	/// The producer node id.
	pub from: u32,
	/// The producer port index.
	pub port: u16,
	/// The consumer node id.
	pub to: u32,
}

/// Finds successor relationships in a data flow graph.
pub struct SuccessorFinder {
	successors: Vec<Successor>,
}

impl SuccessorFinder {
	/// Creates a new empty successor finder.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			successors: Vec::new(),
		}
	}

	/// Returns the successors of a node by its id.
	#[must_use]
	pub fn by_id(&self, id: u32) -> &[Successor] {
		let start = self.successors.partition_point(|entry| entry.from < id);
		let end = self.successors.partition_point(|entry| entry.from <= id);

		&self.successors[start..end]
	}

	/// Returns the successors reachable from a specific producer port.
	#[must_use]
	pub fn by_link(&self, link: Link) -> &[Successor] {
		let start = self
			.successors
			.partition_point(|entry| Link(entry.from, entry.port) < link);

		let end = self
			.successors
			.partition_point(|entry| Link(entry.from, entry.port) <= link);

		&self.successors[start..end]
	}

	/// Recomputes the successor table from the current region's nodes.
	pub fn run(&mut self, nodes: &[Node]) {
		self.successors.clear();

		for (node, to) in nodes.iter().zip(0..) {
			node.for_each_outer(|Link(from, port)| {
				self.successors.push(Successor { from, port, to });
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
