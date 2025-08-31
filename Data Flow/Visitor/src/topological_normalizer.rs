use data_flow_graph::{DataFlowGraph, Node};
use set::Set;

struct DepthFirstSearcher {
	seen: Set,
	stack: Vec<(u32, bool)>,
}

impl DepthFirstSearcher {
	const fn new() -> Self {
		Self {
			seen: Set::new(),
			stack: Vec::new(),
		}
	}

	fn add_predecessor(&mut self, id: u32) {
		if self.seen.contains(id.try_into().unwrap()) {
			return;
		}

		self.stack.push((id, false));
	}

	fn add_predecessors(&mut self, node: &Node) {
		let start = self.stack.len();

		node.for_each_requirement(|id| self.add_predecessor(id));
		node.for_each_argument(|link| self.add_predecessor(link.0));

		self.stack[start..].reverse();
	}

	fn run<H>(&mut self, graph: &mut DataFlowGraph, result: u32, mut handler: H)
	where
		H: FnMut(&mut DataFlowGraph, u32),
	{
		self.seen.clear();

		self.add_predecessor(result);

		while let Some((id, post)) = self.stack.pop() {
			if post {
				handler(graph, id);
			} else if !self.seen.grow_insert(id.try_into().unwrap()) {
				self.stack.push((id, true));

				self.add_predecessors(graph.get(id));
			}
		}
	}
}

pub struct TopologicalNormalizer {
	nodes: Vec<Node>,
	id_to_post: Vec<u32>,

	depth_first_searcher: DepthFirstSearcher,
}

impl TopologicalNormalizer {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			nodes: Vec::new(),
			id_to_post: Vec::new(),

			depth_first_searcher: DepthFirstSearcher::new(),
		}
	}

	fn handle_nodes(&mut self, graph: &mut DataFlowGraph, result: u32) {
		let mut post = 0;

		self.nodes.clear();
		self.id_to_post.clear();
		self.id_to_post.resize(graph.len(), u32::MAX);

		self.depth_first_searcher.run(graph, result, |graph, id| {
			let node = std::mem::take(graph.get_mut(id));

			self.nodes.push(node);
			self.id_to_post[usize::try_from(id).unwrap()] = post;

			post += 1;
		});

		std::mem::swap(graph.inner_mut(), &mut self.nodes);
	}

	fn handle_edges(&self, graph: &mut DataFlowGraph, result: u32) -> u32 {
		for node in graph.nodes_mut() {
			node.for_each_mut_id(|id| {
				let index = usize::try_from(*id).unwrap();

				*id = self.id_to_post[index];
			});
		}

		self.id_to_post[usize::try_from(result).unwrap()]
	}

	pub fn run(&mut self, graph: &mut DataFlowGraph, result: u32) -> u32 {
		self.handle_nodes(graph, result);
		self.handle_edges(graph, result)
	}
}

impl Default for TopologicalNormalizer {
	fn default() -> Self {
		Self::new()
	}
}
