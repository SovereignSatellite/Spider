// Resources:
// "Kosaraju's Strongly Connected Components",
//     by S. Rao Kosaraju

use alloc::vec::Vec;
use control_flow_graph::ControlFlowGraph;
use set::Set;

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

		for id in graph.successors(entry) {
			self.seen.remove(id.into());
		}

		self.seen.grow_insert(exit.into());
	}

	fn add_successor(&mut self, id: u16) {
		if self.seen.contains(id.into()) {
			return;
		}

		self.stack.push((id, false));
	}

	fn run<H, I>(&mut self, result: &mut Vec<u16>, entry: u16, successors: H)
	where
		H: Fn(u16) -> I,
		I: IntoIterator<Item = u16>,
	{
		self.add_successor(entry);

		while let Some((id, post)) = self.stack.pop() {
			if !self.seen.grow_insert(id.into()) {
				self.stack.push((id, true));

				for id in successors(id) {
					self.add_successor(id);
				}
			} else if post {
				result.push(id);
			}
		}
	}
}

pub struct StronglyConnectedFinder {
	separators: Vec<usize>,
	results: Vec<u16>,
	post: Vec<u16>,

	depth_first_searcher: DepthFirstSearcher,
}

impl StronglyConnectedFinder {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			separators: Vec::new(),
			results: Vec::new(),
			post: Vec::new(),

			depth_first_searcher: DepthFirstSearcher::new(),
		}
	}

	pub fn for_each<H: FnMut(&[u16])>(&self, mut handler: H) {
		let mut start = 0;

		for &end in &self.separators {
			handler(&self.results[start..end]);

			start = end;
		}
	}

	fn find_successors(&mut self, graph: &ControlFlowGraph, entry: u16, exit: u16) {
		self.post.clear();

		self.depth_first_searcher.reset(graph, entry, exit);
		self.depth_first_searcher
			.run(&mut self.post, entry, |id| graph.successors(id));
	}

	fn should_store(graph: &ControlFlowGraph, list: &[u16]) -> bool {
		if let &[only] = list {
			graph.predecessors(only).any(|id| id == only)
		} else {
			!list.is_empty()
		}
	}

	fn find_predecessors(&mut self, graph: &ControlFlowGraph, entry: u16, exit: u16) {
		self.separators.clear();
		self.results.clear();

		self.depth_first_searcher.reset(graph, entry, exit);

		let mut start = 0;

		while let Some(id) = self.post.pop() {
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
