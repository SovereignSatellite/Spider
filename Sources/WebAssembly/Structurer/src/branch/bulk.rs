use alloc::vec::Vec;
use web_assembly_graph::ControlFlowGraph;

use super::single::Single;

pub struct Bulk {
	single: Single,

	infos: Vec<(u16, u16)>,
}

impl Bulk {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			single: Single::new(),

			infos: Vec::new(),
		}
	}

	fn find_next_branch(graph: &ControlFlowGraph, mut entry: u16, exit: u16) -> Option<u16> {
		while entry != exit {
			let mut successors = graph.successors(entry).filter(|&id| id != entry);
			let successor = successors.next().unwrap();

			if successors.next().is_none() {
				entry = successor;
			} else {
				return Some(entry);
			}
		}

		None
	}

	fn add_branch(&mut self, graph: &ControlFlowGraph, entry: u16, exit: u16) {
		if let Some(branch_entry) = Self::find_next_branch(graph, entry, exit) {
			self.infos.push((branch_entry, exit));
		}
	}

	fn handle_region(&mut self, graph: &mut ControlFlowGraph, entry: u16, exit: u16) {
		let point = self.single.run(graph, entry);

		self.add_branch(graph, point, exit);

		for successor in graph.successors(entry) {
			self.add_branch(graph, successor, point);
		}
	}

	pub fn run(&mut self, graph: &mut ControlFlowGraph, entry: u16, exit: u16) {
		self.add_branch(graph, entry, exit);

		while let Some((region_entry, region_exit)) = self.infos.pop() {
			self.handle_region(graph, region_entry, region_exit);
		}
	}
}
