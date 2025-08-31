use alloc::vec::Vec;
use control_flow_graph::ControlFlowGraph;

use super::{single::Single, strongly_connected_finder::StronglyConnectedFinder};

pub struct Bulk {
	single: Single,

	infos: Vec<(u16, u16)>,
	strongly_connected_finder: StronglyConnectedFinder,
}

impl Bulk {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			single: Single::new(),
			infos: Vec::new(),

			strongly_connected_finder: StronglyConnectedFinder::new(),
		}
	}

	#[must_use]
	pub fn infos(&self) -> &[(u16, u16)] {
		&self.infos
	}

	fn handle_region(&mut self, graph: &mut ControlFlowGraph, entry: u16, exit: u16) {
		self.strongly_connected_finder.run(graph, entry, exit);
		self.strongly_connected_finder.for_each(|region| {
			let item = self.single.run(graph, region);

			self.infos.push(item);
		});
	}

	pub fn run(&mut self, graph: &mut ControlFlowGraph, entry: u16, exit: u16) {
		let mut index = 0;

		self.infos.clear();

		self.handle_region(graph, entry, exit);

		while index < self.infos.len() {
			let (entry, latch) = self.infos[index];

			index += 1;

			self.handle_region(graph, entry, latch);
		}
	}
}
