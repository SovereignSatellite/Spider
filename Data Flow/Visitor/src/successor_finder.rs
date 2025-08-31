use std::slice::Iter;

use data_flow_graph::DataFlowGraph;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Item {
	pub from: u32,
	pub to: u32,
	pub port: u16,
}

pub struct SuccessorFinder {
	successors: Vec<Item>,
}

impl SuccessorFinder {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			successors: Vec::new(),
		}
	}

	pub fn at(&self, id: u32) -> Iter<'_, Item> {
		let start = self.successors.partition_point(|item| item.from < id);
		let end = self.successors.partition_point(|item| item.from <= id);

		self.successors[start..end].iter()
	}

	pub fn find_last_successors(&self, id: u32, buffer: &mut [u32]) {
		for &Item { to, port, .. } in self.at(id) {
			let port = usize::from(port);

			if let Some(reference) = buffer.get_mut(port) {
				// We don't need to `max` since the ID is ascending.
				*reference = to;
			}
		}
	}

	pub fn run(&mut self, graph: &DataFlowGraph) {
		self.successors.clear();

		for (node, id) in graph.nodes().zip(0..) {
			node.for_each_argument(|link| {
				self.successors.push(Item {
					from: link.0,
					to: id,
					port: link.1,
				});
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
