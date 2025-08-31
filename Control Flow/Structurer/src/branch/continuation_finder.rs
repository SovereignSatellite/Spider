use alloc::vec::Vec;
use control_flow_graph::ControlFlowGraph;
use set::Set;

pub struct ContinuationFinder {
	points: Vec<u16>,
	excluded: Vec<u16>,

	expanded: Set,
	seen: Set,
	stack: Vec<u16>,
}

impl ContinuationFinder {
	pub const fn new() -> Self {
		Self {
			points: Vec::new(),
			excluded: Vec::new(),

			expanded: Set::new(),
			seen: Set::new(),
			stack: Vec::new(),
		}
	}

	pub fn set_excluded<I: IntoIterator<Item = u16>>(&mut self, iter: I) {
		self.excluded.clear();
		self.excluded.extend(iter);
		self.excluded.sort_unstable();
		self.excluded.dedup();
	}

	pub fn edges_into(&self, graph: &ControlFlowGraph, container: &mut Vec<(u16, u16)>) {
		for &point in &self.points {
			for predecessor in graph.predecessors(point) {
				if self.expanded.contains(predecessor.into()) {
					container.push((predecessor, point));
				}
			}
		}
	}

	fn dominates(&self, graph: &ControlFlowGraph, id: u16) -> bool {
		self.excluded.binary_search(&id).is_err()
			&& graph
				.predecessors(id)
				.all(|predecessor| id == predecessor || self.expanded.contains(predecessor.into()))
	}

	fn add_successor(&mut self, id: u16) {
		if self.seen.grow_insert(id.into()) {
			return;
		}

		self.stack.push(id);
	}

	fn set_entry(&mut self, graph: &ControlFlowGraph, entry: u16) {
		if self.excluded.binary_search(&entry).is_ok() {
			let predecessor = graph.predecessors(entry).next().unwrap();

			self.points.push(entry);
			self.expanded.grow_insert(predecessor.into());
		} else {
			let entry_usize = entry.into();

			self.seen.clear();
			self.seen.grow_insert(entry_usize);
			self.expanded.grow_insert(entry_usize);

			for successor in graph.successors(entry) {
				self.add_successor(successor);
			}
		}
	}

	fn handle_stack(&mut self, graph: &ControlFlowGraph) -> bool {
		let mut changed = false;

		while let Some(id) = self.stack.pop() {
			if self.dominates(graph, id) {
				self.expanded.grow_insert(id.into());

				for successor in graph.successors(id) {
					self.add_successor(successor);
				}

				changed = true;
			} else {
				self.points.push(id);
			}
		}

		changed
	}

	pub fn run(&mut self, graph: &ControlFlowGraph, entry: u16) {
		self.points.clear();
		self.expanded.clear();

		self.set_entry(graph, entry);

		while self.handle_stack(graph) {
			core::mem::swap(&mut self.points, &mut self.stack);
		}
	}
}
