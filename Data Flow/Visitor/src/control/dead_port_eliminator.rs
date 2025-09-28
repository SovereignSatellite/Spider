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

	fn mark_lambda_in(&mut self, lambda_in: &LambdaIn, id: u32) {
		let LambdaIn { dependencies, .. } = lambda_in;

		self.add_region_sides(dependencies, id);
	}

	fn mark_lambda_out(&mut self, lambda_out: &LambdaOut, id: u32) {
		let LambdaOut { results, .. } = lambda_out;

		self.add_region_sides(results, id);
	}

	fn mark_region_in(&mut self, region_in: &RegionIn, port: u16) {
		let RegionIn { input, .. } = *region_in;

		self.add_predecessor(Link(input, port));
	}

	fn mark_region_out(&mut self, region_out: &RegionOut, port: u16) {
		let RegionOut { results, .. } = region_out;

		self.add_predecessor(results[usize::from(port)]);
	}

	fn mark_gamma_in(&mut self, graph: &DataFlowGraph, gamma_in: &GammaIn, port: u16) {
		let GammaIn {
			output,
			ref arguments,
			..
		} = *gamma_in;
		let GammaOut { regions, .. } = graph.get(output).as_gamma_out().unwrap();

		for &region in regions {
			let RegionOut { input, .. } = *graph.get(region).as_region_out().unwrap();

			self.add_predecessor(Link(input, port));
		}

		self.add_predecessor(arguments[usize::from(port)]);
	}

	fn mark_gamma_out(&mut self, graph: &DataFlowGraph, gamma_out: &GammaOut, port: u16) {
		let GammaOut { input, ref regions } = *gamma_out;
		let GammaIn { condition, .. } = *graph.get(input).as_gamma_in().unwrap();

		self.add_predecessor(condition);

		for &region in regions {
			self.add_predecessor(Link(region, port));
		}
	}

	fn mark_theta_in(&mut self, theta_in: &ThetaIn, port: u16) {
		let ThetaIn {
			output,
			ref arguments,
		} = *theta_in;

		self.add_predecessor(Link(output, port));
		self.add_predecessor(arguments[usize::from(port)]);
	}

	fn mark_theta_out(&mut self, theta_out: &ThetaOut, port: u16) {
		let ThetaOut {
			input,
			ref results,
			condition,
		} = *theta_out;

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
				Node::LambdaIn(lambda_in) => self.mark_lambda_in(lambda_in, id),
				Node::LambdaOut(lambda_out) => self.mark_lambda_out(lambda_out, id),
				Node::RegionIn(region_in) => self.mark_region_in(region_in, port),
				Node::RegionOut(region_out) => self.mark_region_out(region_out, port),
				Node::GammaIn(gamma_in) => self.mark_gamma_in(graph, gamma_in, port),
				Node::GammaOut(gamma_out) => self.mark_gamma_out(graph, gamma_out, port),
				Node::ThetaIn(theta_in) => self.mark_theta_in(theta_in, port),
				Node::ThetaOut(theta_out) => self.mark_theta_out(theta_out, port),

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
