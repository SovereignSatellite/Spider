use alloc::vec::Vec;
use control_flow_graph::{ControlFlowGraph, instruction::Name};
use set::{Set, Slice};

pub struct Single {
	entries: Vec<u16>,
	exits: Vec<u16>,

	region: Set,
	temporary: Vec<u16>,
}

impl Single {
	pub const fn new() -> Self {
		Self {
			entries: Vec::new(),
			exits: Vec::new(),

			region: Set::new(),
			temporary: Vec::new(),
		}
	}

	fn set_region_contents(&mut self, region: &[u16]) {
		let region = region.iter().copied().map(usize::from);

		self.region.clear();
		self.region.extend(region);
	}

	fn find_entries_and_exits(&mut self, graph: &ControlFlowGraph) {
		self.entries.clear();
		self.exits.clear();

		for id in self.region.ascending() {
			let id = id.try_into().unwrap();

			if graph
				.predecessors(id)
				.any(|id| !self.region.contains(id.into()))
			{
				self.entries.push(id);
			}

			self.exits.extend(
				graph
					.successors(id)
					.filter(|&id| !self.region.contains(id.into())),
			);
		}

		self.exits.sort_unstable();
		self.exits.dedup();
	}

	fn set_new_entry(&mut self, graph: &mut ControlFlowGraph) -> u16 {
		let selection = graph.add_selection(Name::C);

		for (&entry, index) in self.entries.iter().zip(0..) {
			self.temporary.clear();
			self.temporary.extend(graph.predecessors(entry));

			for &predecessor in &self.temporary {
				let assignment = graph.add_assignment(Name::C, index);

				graph.replace_edge(predecessor, entry, assignment);
				graph.add_edge(assignment, selection);
			}

			graph.add_edge(selection, entry);
		}

		selection
	}

	fn find_or_set_entry(&mut self, graph: &mut ControlFlowGraph) -> u16 {
		if let &[entry] = self.entries.as_slice() {
			entry
		} else {
			self.set_new_entry(graph)
		}
	}

	fn set_new_exit(&mut self, graph: &mut ControlFlowGraph) -> u16 {
		let selection = graph.add_selection(Name::C);

		for (&exit, index) in self.exits.iter().zip(0..) {
			self.temporary.clear();
			self.temporary.extend(
				graph
					.predecessors(exit)
					.filter(|&id| self.region.contains(id.into())),
			);

			for &predecessor in &self.temporary {
				let assignment = graph.add_assignment(Name::C, index);

				graph.replace_edge(predecessor, exit, assignment);
				graph.add_edge(assignment, selection);
			}

			graph.add_edge(selection, exit);
		}

		selection
	}

	fn find_or_set_exit(&mut self, graph: &mut ControlFlowGraph) -> u16 {
		match self.exits.as_slice() {
			[exit] => *exit,
			[] => graph.add_no_operation(),
			[..] => self.set_new_exit(graph),
		}
	}

	// Either the `target` is in the `region`, or one of its predecessors which was added during this pass is.
	fn in_region(graph: &ControlFlowGraph, region: Slice, target: u16) -> bool {
		region.contains(target.into())
			|| graph
				.predecessors(target)
				.any(|id| region.contains(id.into()))
	}

	fn in_region_acyclic(graph: &ControlFlowGraph, region: Slice, target: u16, exit: u16) -> bool {
		target != exit && Self::in_region(graph, region, target)
	}

	fn find_latch(&self, graph: &ControlFlowGraph, entry: u16, exit: u16) -> Option<u16> {
		let mut repetitions = graph
			.predecessors(entry)
			.filter(|&id| Self::in_region(graph, self.region.as_slice(), id));

		if let Some(repetition) = repetitions.next() {
			let mut escapes = graph
				.predecessors(exit)
				.filter(|&id| Self::in_region_acyclic(graph, self.region.as_slice(), id, exit));

			if let Some(escape) = escapes.next()
				&& repetition == escape
				&& repetitions.next().is_none()
				&& escapes.next().is_none()
			{
				let mut edges = graph.successors(repetition);

				if edges.next().is_some() && edges.next().is_some() && edges.next().is_none() {
					return Some(repetition);
				}
			}
		}

		None
	}

	fn set_break(&mut self, graph: &mut ControlFlowGraph, latch: u16, selection: u16) {
		self.temporary.clear();
		self.temporary.extend(
			graph.predecessors(selection).filter(|&id| {
				Self::in_region_acyclic(graph, self.region.as_slice(), id, selection)
			}),
		);

		for &exit in &self.temporary {
			let assignment = graph.add_assignment(Name::B, 0);

			graph.replace_edge(exit, selection, assignment);
			graph.add_edge(assignment, latch);
		}
	}

	fn set_continue(&mut self, graph: &mut ControlFlowGraph, latch: u16, selection: u16) {
		self.temporary.clear();
		self.temporary.extend(
			graph
				.predecessors(selection)
				.filter(|&id| Self::in_region(graph, self.region.as_slice(), id)),
		);

		for &entry in &self.temporary {
			let assignment = graph.add_assignment(Name::B, 1);

			graph.replace_edge(entry, selection, assignment);
			graph.add_edge(assignment, latch);
		}
	}

	fn set_new_latch(&mut self, graph: &mut ControlFlowGraph, entry: u16, exit: u16) -> u16 {
		let selection = graph.add_selection(Name::B);

		self.set_break(graph, selection, exit);
		self.set_continue(graph, selection, entry);

		graph.add_edge(selection, exit);
		graph.add_edge(selection, entry);

		selection
	}

	fn find_or_set_latch(&mut self, graph: &mut ControlFlowGraph, entry: u16, exit: u16) -> u16 {
		self.find_latch(graph, entry, exit)
			.unwrap_or_else(|| self.set_new_latch(graph, entry, exit))
	}

	pub fn run(&mut self, graph: &mut ControlFlowGraph, region: &[u16]) -> (u16, u16) {
		self.set_region_contents(region);
		self.find_entries_and_exits(graph);

		let entry = self.find_or_set_entry(graph);
		let exit = self.find_or_set_exit(graph);
		let latch = self.find_or_set_latch(graph, entry, exit);

		(entry, latch)
	}
}
