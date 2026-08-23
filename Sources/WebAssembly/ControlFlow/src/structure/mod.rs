//! Implement Reissmann et al.'s “Efficient Control Flow Restructuring for GPUs” algorithm.

use self::{branch::BranchNormalizer, exits::ExitNormalizer, repeat::RepeatNormalizer};
use super::ControlFlowGraph;

mod branch;
mod exits;
mod repeat;

/// Restructures a control flow graph into structured control flow.
pub struct ControlFlowStructurer {
	repeat: RepeatNormalizer,
	exits: ExitNormalizer,
	branch: BranchNormalizer,
}

impl ControlFlowStructurer {
	/// Creates a new control flow structurer.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			repeat: RepeatNormalizer::new(),
			exits: ExitNormalizer::new(),
			branch: BranchNormalizer::new(),
		}
	}

	fn disable_repeats(&self, graph: &mut ControlFlowGraph) {
		for &(entry, latch) in self.repeat.regions() {
			graph.replace_edge(latch, entry, latch);
		}
	}

	fn enable_repeats(&self, graph: &mut ControlFlowGraph) {
		for &(entry, latch) in self.repeat.regions() {
			graph.replace_edge(latch, latch, entry);
		}
	}

	/// Runs the full restructuring pipeline on the graph.
	pub fn run(&mut self, graph: &mut ControlFlowGraph, entry: u16, exit: u16) {
		self.repeat.run(graph, entry, exit);
		self.exits.run(graph, entry, exit);
		self.disable_repeats(graph);
		self.branch.run(graph, entry, exit);
		self.enable_repeats(graph);
	}
}

impl Default for ControlFlowStructurer {
	fn default() -> Self {
		Self::new()
	}
}
