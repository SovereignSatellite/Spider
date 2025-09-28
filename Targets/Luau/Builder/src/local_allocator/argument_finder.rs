use alloc::vec::Vec;
use data_flow_graph::{
	DataFlowGraph, Link, Node,
	control::{LambdaIn, LambdaOut, OmegaIn, OmegaOut},
};
use hashbrown::HashMap;
use set::Set;

pub fn get_region_range(graph: &DataFlowGraph, node: &Node) -> Option<(u32, u32)> {
	let range = match *node {
		Node::OmegaOut(OmegaOut { input, .. }) => {
			let OmegaIn { output } = *graph.get(input).as_omega_in().unwrap();

			(input, output)
		}
		Node::LambdaOut(LambdaOut { input, .. }) => {
			let LambdaIn { output, .. } = *graph.get(input).as_lambda_in().unwrap();

			(input, output)
		}

		_ => return None,
	};

	Some(range)
}

pub struct ArgumentFinder {
	seen: Set,
	stack: Vec<u32>,
}

impl ArgumentFinder {
	pub const fn new() -> Self {
		Self {
			seen: Set::new(),
			stack: Vec::new(),
		}
	}

	pub fn clear(&mut self) {
		self.seen.clear();
	}

	fn handle_arguments(
		&mut self,
		arguments: &mut Vec<(Link, Link)>,
		preferences: &HashMap<Link, Link>,
		graph: &DataFlowGraph,
		id: u32,
	) {
		if self.seen.grow_insert(id.try_into().unwrap()) {
			return;
		}

		graph.get(id).for_each_argument(|link| {
			if let Some(&preferred) = preferences.get(&link) {
				arguments.push((link, preferred));
			} else {
				self.stack.push(link.0);
			}
		});
	}

	pub fn run(
		&mut self,
		arguments: &mut Vec<(Link, Link)>,
		preferences: &HashMap<Link, Link>,
		graph: &DataFlowGraph,
		start: u32,
	) {
		arguments.clear();

		self.handle_arguments(arguments, preferences, graph, start);

		while let Some(id) = self.stack.pop() {
			let id = get_region_range(graph, graph.get(id)).map_or(id, |range| range.0);

			self.handle_arguments(arguments, preferences, graph, id);
		}
	}
}
