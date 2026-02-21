use ir_graph::{DataFlowGraph, Link};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct BiLink {
	pub from: u32,
	pub port: u16,
	pub to: u32,
}

pub struct SuccessorFinder {
	successors: Vec<BiLink>,
}

impl SuccessorFinder {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			successors: Vec::new(),
		}
	}

	#[must_use]
	pub fn by_id(&self, id: u32) -> &[BiLink] {
		let start = self.successors.partition_point(|bi| bi.from < id);
		let end = self.successors.partition_point(|bi| bi.from <= id);

		&self.successors[start..end]
	}

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
