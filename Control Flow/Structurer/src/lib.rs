// Resources:
// "Efficient Control Flow Restructuring for GPUs",
//     by Nico Reissmann, Thomas L. Falch, Benjamin A. Bjørnseth,
//        Helge Bahmann, Jan Christian Meyer, and Magnus Jahre.
#![no_std]

extern crate alloc;

mod branch;
mod repeat;
mod single_exit_patcher;

use control_flow_graph::ControlFlowGraph;

use self::{
	branch::bulk::Bulk as Branch, repeat::bulk::Bulk as Repeat,
	single_exit_patcher::SingleExitPatcher,
};

pub struct Structurer {
	repeat: Repeat,
	branch: Branch,

	single_exit_patcher: SingleExitPatcher,
}

impl Structurer {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			repeat: Repeat::new(),
			branch: Branch::new(),

			single_exit_patcher: SingleExitPatcher::new(),
		}
	}

	pub fn handle_repeats(&mut self, graph: &mut ControlFlowGraph, entry: u16, exit: u16) {
		self.repeat.run(graph, entry, exit);
	}

	pub fn handle_exits(&mut self, graph: &mut ControlFlowGraph, entry: u16, exit: u16) {
		self.single_exit_patcher.run(graph, entry, exit);
	}

	pub fn disable_repeats(&self, graph: &mut ControlFlowGraph) {
		for &(entry, latch) in self.repeat.infos() {
			graph.replace_edge(latch, entry, latch);
		}
	}

	pub fn enable_repeats(&self, graph: &mut ControlFlowGraph) {
		for &(entry, latch) in self.repeat.infos() {
			graph.replace_edge(latch, latch, entry);
		}
	}

	pub fn handle_branches(&mut self, graph: &mut ControlFlowGraph, entry: u16, exit: u16) {
		self.branch.run(graph, entry, exit);
	}

	pub fn run(&mut self, graph: &mut ControlFlowGraph, entry: u16, exit: u16) {
		self.handle_repeats(graph, entry, exit);
		self.handle_exits(graph, entry, exit);
		self.disable_repeats(graph);
		self.handle_branches(graph, entry, exit);
		self.enable_repeats(graph);
	}
}

impl Default for Structurer {
	fn default() -> Self {
		Self::new()
	}
}
