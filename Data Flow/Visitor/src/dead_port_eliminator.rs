use data_flow_graph::{
	nested::{GammaIn, GammaOut, LambdaIn, LambdaOut, RegionIn, RegionOut, ThetaIn, ThetaOut},
	DataFlowGraph, Link, Node,
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

	fn add_all_links(&mut self, links: &[Link], id: u32) {
		let len = links.len().try_into().unwrap();

		self.seen
			.extend((0..len).map(|port| Link(id, port)).map(Link::into_usize));

		for &link in links {
			self.add_predecessor(link);
		}
	}

	fn mark_lambda_in(&mut self, lambda_in: &LambdaIn, id: u32) {
		self.add_all_links(&lambda_in.dependencies, id);
	}

	fn mark_lambda_out(&mut self, lambda_out: &LambdaOut, id: u32) {
		self.add_all_links(&lambda_out.results, id);
	}

	fn mark_region_in(&mut self, region_in: &RegionIn, port: u16) {
		self.add_predecessor(Link(region_in.input, port));
	}

	fn mark_region_out(&mut self, region_out: &RegionOut, port: u16) {
		self.add_predecessor(region_out.results[usize::from(port)]);
	}

	fn mark_gamma_in(&mut self, graph: &DataFlowGraph, gamma_in: &GammaIn, port: u16) {
		let GammaIn {
			output, arguments, ..
		} = gamma_in;
		let GammaOut { regions, .. } = graph.get(*output).as_gamma_out().unwrap();

		for &region in regions {
			let RegionOut { input, .. } = graph.get(region).as_region_out().unwrap();

			self.add_predecessor(Link(*input, port));
		}

		self.add_predecessor(arguments[usize::from(port)]);
	}

	fn mark_gamma_out(&mut self, graph: &DataFlowGraph, gamma_out: &GammaOut, port: u16) {
		let GammaOut { input, regions } = gamma_out;
		let GammaIn { condition, .. } = graph.get(*input).as_gamma_in().unwrap();

		self.add_predecessor(*condition);

		for &region in regions {
			self.add_predecessor(Link(region, port));
		}
	}

	fn mark_theta_in(&mut self, theta_in: &ThetaIn, port: u16) {
		self.add_predecessor(Link(theta_in.output, port));
		self.add_predecessor(theta_in.arguments[usize::from(port)]);
	}

	fn mark_theta_out(&mut self, theta_out: &ThetaOut, port: u16) {
		self.add_predecessor(Link(theta_out.input, port));
		self.add_predecessor(theta_out.results[usize::from(port)]);
		self.add_predecessor(theta_out.condition);
	}

	fn mark_operation(&mut self, node: &Node) {
		node.for_each_argument(|link| {
			self.add_predecessor(link);
		});
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

		let mut positions = (0..u16::MAX).map(|port| Link(id, port));

		for from in (0..u16::try_from(ports).unwrap()).map(|port| Link(id, port)) {
			if !self.seen.contains(from.into_usize()) {
				continue;
			}

			let to = positions.next().unwrap();

			if from.1 != to.1 {
				self.map.insert(from, to);
			}
		}
	}

	fn sweep_inputs(&self, graph: &mut DataFlowGraph, id: u32) {
		let Some(arguments) = graph.get_mut(id).as_mut_ports() else {
			return;
		};

		let mut links = (0..u16::MAX)
			.map(|port| Link(id, port))
			.map(Link::into_usize);

		arguments.retain(|_| self.seen.contains(links.next().unwrap()));
	}

	fn sweep(&mut self, graph: &mut DataFlowGraph) {
		self.map.clear();

		for id in (0..graph.len().try_into().unwrap()).rev() {
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
