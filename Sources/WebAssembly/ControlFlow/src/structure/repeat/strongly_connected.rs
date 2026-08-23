use alloc::vec::Vec;

use set::Set;

use crate::ControlFlowGraph;

struct DepthFirstSearcher {
	seen: Set,
	stack: Vec<(u16, bool)>,
}

impl DepthFirstSearcher {
	const fn new() -> Self {
		Self {
			seen: Set::new(),
			stack: Vec::new(),
		}
	}

	fn reset(&mut self, graph: &ControlFlowGraph, entry: u16, exit: u16) {
		self.seen.clear();
		self.seen.extend(graph.predecessors(entry).map(usize::from));

		for successor_id in graph.successors(entry) {
			self.seen.remove(successor_id.into());
		}

		self.seen.grow_insert(exit.into());
	}

	fn add_successor(&mut self, id: u16) {
		if self.seen.contains(id.into()) {
			return;
		}

		self.stack.push((id, false));
	}

	fn run<Successors, SuccessorIdentifiers>(
		&mut self,
		result: &mut Vec<u16>,
		entry: u16,
		successors: Successors,
	) where
		Successors: Fn(u16) -> SuccessorIdentifiers,
		SuccessorIdentifiers: IntoIterator<Item = u16>,
	{
		self.add_successor(entry);

		while let Some((id, is_postvisit)) = self.stack.pop() {
			if self.seen.grow_insert(id.into()) {
				if is_postvisit {
					result.push(id);
				}
			} else {
				self.stack.push((id, true));

				for successor_id in successors(id) {
					self.add_successor(successor_id);
				}
			}
		}
	}
}

pub struct StronglyConnectedFinder {
	separators: Vec<usize>,
	results: Vec<u16>,
	postorder: Vec<u16>,

	depth_first_searcher: DepthFirstSearcher,
}

impl StronglyConnectedFinder {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			separators: Vec::new(),
			results: Vec::new(),
			postorder: Vec::new(),

			depth_first_searcher: DepthFirstSearcher::new(),
		}
	}

	pub fn for_each<Handler: FnMut(&[u16])>(&self, mut handler: Handler) {
		let mut start = 0;

		for &end in &self.separators {
			handler(&self.results[start..end]);

			start = end;
		}
	}

	fn find_successors(&mut self, graph: &ControlFlowGraph, entry: u16, exit: u16) {
		self.postorder.clear();

		self.depth_first_searcher.reset(graph, entry, exit);
		self.depth_first_searcher
			.run(&mut self.postorder, entry, |id| graph.successors(id));
	}

	fn should_store(graph: &ControlFlowGraph, component: &[u16]) -> bool {
		if let &[only] = component {
			graph.predecessors(only).any(|id| id == only)
		} else {
			!component.is_empty()
		}
	}

	fn find_predecessors(&mut self, graph: &ControlFlowGraph, entry: u16, exit: u16) {
		self.separators.clear();
		self.results.clear();

		self.depth_first_searcher.reset(graph, entry, exit);

		let mut start = 0;

		while let Some(id) = self.postorder.pop() {
			self.depth_first_searcher
				.run(&mut self.results, id, |id| graph.predecessors(id));

			if Self::should_store(graph, &self.results[start..]) {
				start = self.results.len();

				self.separators.push(start);
			} else {
				self.results.truncate(start);
			}
		}
	}

	pub fn run(&mut self, graph: &ControlFlowGraph, entry: u16, exit: u16) {
		self.find_successors(graph, entry, exit);
		self.find_predecessors(graph, entry, exit);
	}
}
