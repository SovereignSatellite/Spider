use alloc::vec::Vec;

use set::Set;

use crate::ControlFlowGraph;

pub struct ExitNormalizer {
	seen: Set,
	stack: Vec<u16>,
}

impl ExitNormalizer {
	pub const fn new() -> Self {
		Self {
			seen: Set::new(),
			stack: Vec::new(),
		}
	}

	fn add_successor(&mut self, id: u16) {
		if self.seen.grow_insert(id.into()) {
			return;
		}

		self.stack.push(id);
	}

	pub fn run(&mut self, graph: &mut ControlFlowGraph, entry: u16, exit: u16) {
		self.seen.clear();
		self.seen.grow_insert(exit.into());

		self.add_successor(entry);

		while let Some(id) = self.stack.pop() {
			let mut sink = true;

			for successor_id in graph.successors(id) {
				self.add_successor(successor_id);

				sink = false;
			}

			if sink {
				graph.add_edge(id, exit);
			}
		}
	}
}
