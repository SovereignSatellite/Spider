//! Restructures WebAssembly control flow with Reissmann et al.'s
//! "Efficient Control Flow Restructuring for GPUs".
#![no_std]

extern crate alloc;

use web_assembly_graph::ControlFlowGraph;

use self::{
	branch::bulk::Bulk as Branch, repeat::bulk::Bulk as Repeat,
	single_exit_patcher::SingleExitPatcher,
};

mod branch;
mod repeat;
mod single_exit_patcher;

/// Restructures a control flow graph into structured control flow.
pub struct ControlFlowStructurer {
	repeat: Repeat,
	branch: Branch,

	single_exit_patcher: SingleExitPatcher,
}

impl ControlFlowStructurer {
	#[must_use]
	/// Creates a new control flow structurer.
	pub const fn new() -> Self {
		Self {
			repeat: Repeat::new(),
			branch: Branch::new(),

			single_exit_patcher: SingleExitPatcher::new(),
		}
	}

	/// Identifies and restructures repeat (loop) regions.
	pub fn handle_repeats(&mut self, graph: &mut ControlFlowGraph, entry: u16, exit: u16) {
		self.repeat.run(graph, entry, exit);
	}

	/// Patches multi-exit regions into single-exit regions.
	pub fn handle_exits(&mut self, graph: &mut ControlFlowGraph, entry: u16, exit: u16) {
		self.single_exit_patcher.run(graph, entry, exit);
	}

	/// Disables repeat edges in the graph.
	pub fn disable_repeats(&self, graph: &mut ControlFlowGraph) {
		for &(entry, latch) in self.repeat.regions() {
			graph.replace_edge(latch, entry, latch);
		}
	}

	/// Enables repeat edges in the graph.
	pub fn enable_repeats(&self, graph: &mut ControlFlowGraph) {
		for &(entry, latch) in self.repeat.regions() {
			graph.replace_edge(latch, latch, entry);
		}
	}

	/// Restructures branch regions in the graph.
	pub fn handle_branches(&mut self, graph: &mut ControlFlowGraph, entry: u16, exit: u16) {
		self.branch.run(graph, entry, exit);
	}

	/// Runs the full restructuring pipeline on the graph.
	pub fn run(&mut self, graph: &mut ControlFlowGraph, entry: u16, exit: u16) {
		self.handle_repeats(graph, entry, exit);
		self.handle_exits(graph, entry, exit);
		self.disable_repeats(graph);
		self.handle_branches(graph, entry, exit);
		self.enable_repeats(graph);
	}
}

impl Default for ControlFlowStructurer {
	fn default() -> Self {
		Self::new()
	}
}
