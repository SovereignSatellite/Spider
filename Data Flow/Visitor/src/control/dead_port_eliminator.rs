use data_flow_graph::{
	DataFlowGraph, Link, Node,
	control::{GammaIn, GammaOut, LambdaIn, LambdaOut, RegionIn, RegionOut, ThetaIn, ThetaOut},
};
use hashbrown::HashMap;
use set::Set;

pub struct DeadPortEliminator {
	map: HashMap<Link, Link>,

	seen: Set,
	stack: Vec<Link>,
}

impl DeadPortEliminator {
	#[must_use]
	pub fn new() -> Self {
		Self {
			map: HashMap::new(),

			seen: Set::new(),
			stack: Vec::new(),
		}
	}

	fn add_predecessor(&mut self, link: Link) {
		if self.seen.grow_insert(link.into_usize()) {
			return;
		}

		self.stack.push(link);
	}

	fn add_region_sides(&mut self, links: &[Link], id: u32) {
		let len = links.len();

		self.seen.extend(
			(0..)
				.map(|port| Link(id, port))
				.map(Link::into_usize)
				.take(len),
		);

		for &link in links {
			self.add_predecessor(link);
		}
	}

	fn mark_lambda_in(&mut self, node: &LambdaIn, id: u32) {
		let LambdaIn { dependencies, .. } = node;

		self.add_region_sides(dependencies, id);
	}

	fn mark_lambda_out(&mut self, node: &LambdaOut, id: u32) {
		let LambdaOut { results, .. } = node;

		self.add_region_sides(results, id);
	}

	fn mark_region_in(&mut self, node: &RegionIn, port: u16) {
		let RegionIn { input, .. } = *node;

		self.add_predecessor(Link(input, port));
	}

	fn mark_region_out(&mut self, node: &RegionOut, port: u16) {
		let RegionOut { results, .. } = node;

		self.add_predecessor(results[usize::from(port)]);
	}

	fn mark_gamma_in(&mut self, graph: &DataFlowGraph, node: &GammaIn, port: u16) {
		let GammaIn {
			output,
			ref arguments,
			..
		} = *node;
		let GammaOut { regions, .. } = graph.get(output).as_gamma_out().unwrap();

		for &region in regions {
			let RegionOut { input, .. } = *graph.get(region).as_region_out().unwrap();

			self.add_predecessor(Link(input, port));
		}

		self.add_predecessor(arguments[usize::from(port)]);
	}

	fn mark_gamma_out(&mut self, graph: &DataFlowGraph, node: &GammaOut, port: u16) {
		let GammaOut { input, ref regions } = *node;
		let GammaIn { condition, .. } = *graph.get(input).as_gamma_in().unwrap();

		self.add_predecessor(condition);

		for &region in regions {
			self.add_predecessor(Link(region, port));
		}
	}

	fn mark_theta_in(&mut self, node: &ThetaIn, port: u16) {
		let ThetaIn {
			output,
			ref arguments,
		} = *node;

		self.add_predecessor(Link(output, port));
		self.add_predecessor(arguments[usize::from(port)]);
	}

	fn mark_theta_out(&mut self, node: &ThetaOut, port: u16) {
		let ThetaOut {
			input,
			ref results,
			condition,
		} = *node;

		self.add_predecessor(Link(input, port));
		self.add_predecessor(results[usize::from(port)]);
		self.add_predecessor(condition);
	}

	fn mark_operation(&mut self, node: &Node) {
		node.for_each_argument(|link| self.add_predecessor(link));
	}

	fn mark(&mut self, graph: &DataFlowGraph, result: Link) {
		self.seen.clear();

		self.add_predecessor(result);

		while let Some(Link(id, port)) = self.stack.pop() {
			match graph.get(id) {
				Node::LambdaIn(node) => self.mark_lambda_in(node, id),
				Node::LambdaOut(node) => self.mark_lambda_out(node, id),
				Node::RegionIn(node) => self.mark_region_in(node, port),
				Node::RegionOut(node) => self.mark_region_out(node, port),
				Node::GammaIn(node) => self.mark_gamma_in(graph, node, port),
				Node::GammaOut(node) => self.mark_gamma_out(graph, node, port),
				Node::ThetaIn(node) => self.mark_theta_in(node, port),
				Node::ThetaOut(node) => self.mark_theta_out(node, port),

				node => self.mark_operation(node),
			}
		}
	}

	fn sweep_outputs(&mut self, graph: &DataFlowGraph, id: u32) {
		let Some(ports) = graph.get(id).ports_output(graph) else {
			return;
		};

		let mut outputs = (0..).map(|port| Link(id, port));

		for from in outputs.clone().take(ports) {
			if !self.seen.contains(from.into_usize()) {
				continue;
			}

			let to = outputs.next().unwrap();

			if from.1 != to.1 {
				self.map.insert(from, to);
			}
		}
	}

	fn sweep_inputs(&self, graph: &mut DataFlowGraph, id: u32) {
		let Some(arguments) = graph.get_mut(id).as_mut_ports() else {
			return;
		};

		let mut inputs = (0..).map(|port| Link(id, port)).map(Link::into_usize);

		arguments.retain(|_| self.seen.contains(inputs.next().unwrap()));
	}

	fn sweep(&mut self, graph: &mut DataFlowGraph) {
		let len = graph.len();

		self.map.clear();

		for id in (0..len.try_into().unwrap()).rev() {
			self.sweep_outputs(graph, id);
			self.sweep_inputs(graph, id);
		}

		for node in graph.nodes_mut() {
			node.for_each_mut_argument(|old| {
				if let Some(new) = self.map.get(old) {
					*old = *new;
				}
			});
		}
	}

	pub fn run(&mut self, graph: &mut DataFlowGraph, result: Link) {
		self.mark(graph, result);
		self.sweep(graph);
	}
}

impl Default for DeadPortEliminator {
	fn default() -> Self {
		Self::new()
	}
}
