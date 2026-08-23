use alloc::vec::Vec;

use super::{rewrite::RepeatRewriter, strongly_connected::StronglyConnectedFinder};
use crate::ControlFlowGraph;

pub struct RepeatNormalizer {
	rewriter: RepeatRewriter,

	regions: Vec<(u16, u16)>,
	components: StronglyConnectedFinder,
}

impl RepeatNormalizer {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			rewriter: RepeatRewriter::new(),

			regions: Vec::new(),
			components: StronglyConnectedFinder::new(),
		}
	}

	#[must_use]
	pub fn regions(&self) -> &[(u16, u16)] {
		&self.regions
	}

	fn handle_region(&mut self, graph: &mut ControlFlowGraph, entry: u16, exit: u16) {
		self.components.run(graph, entry, exit);
		self.components.for_each(|region| {
			let item = self.rewriter.run(graph, region);

			self.regions.push(item);
		});
	}

	pub fn run(&mut self, graph: &mut ControlFlowGraph, entry: u16, exit: u16) {
		let mut index = 0;

		self.regions.clear();

		self.handle_region(graph, entry, exit);

		while index < self.regions.len() {
			let (region_entry, latch) = self.regions[index];

			index += 1;

			self.handle_region(graph, region_entry, latch);
		}
	}
}
