use data_flow_graph::{
	DataFlowGraph, Link, Node,
	nested::{GammaIn, GammaOut, RegionOut, ThetaIn, ThetaOut},
};
use hashbrown::HashMap;

pub struct FallthroughMover {
	map: HashMap<Link, Link>,
}

impl FallthroughMover {
	#[must_use]
	pub fn new() -> Self {
		Self {
			map: HashMap::new(),
		}
	}

	fn get_reference(&self, link: Link) -> Link {
		self.map.get(&link).copied().unwrap_or(link)
	}

	fn find_argument(&self, result: Link, input: u32, arguments: &[Link]) -> Option<Link> {
		let result = self.get_reference(result);

		(result.0 == input).then(|| {
			let port = usize::from(result.1);

			self.get_reference(arguments[port])
		})
	}

	fn find_shared_reference(
		&self,
		graph: &DataFlowGraph,
		regions: &[u32],
		port: usize,
		arguments: &[Link],
	) -> Option<Link> {
		let mut arguments = regions.iter().map(|&region| {
			let RegionOut { input, results, .. } = graph.get(region).as_region_out().unwrap();

			self.find_argument(results[port], *input, arguments)
		});

		arguments
			.next()
			.unwrap()
			.filter(|&first| arguments.all(|link| link == Some(first)))
	}

	fn handle_simple_reference(
		&mut self,
		arguments: &[Link],
		results: &[Link],
		input: u32,
		output: u32,
	) {
		// Assuming the result of the region is the same value as its argument,
		// then we can replace it with a direct link.
		for (port, &result) in results.iter().enumerate() {
			if let Some(argument) = self.find_argument(result, input, arguments)
				&& argument == self.get_reference(arguments[port])
			{
				let link = Link(output, port.try_into().unwrap());

				self.map.insert(link, argument);
			}
		}
	}

	fn handle_gamma(&mut self, graph: &DataFlowGraph, gamma_out: &GammaOut) {
		let GammaOut { input, regions } = gamma_out;
		let GammaIn {
			output, arguments, ..
		} = graph.get(*input).as_gamma_in().unwrap();

		for port in 0..gamma_out.ports_output(graph) {
			if let Some(argument) = self.find_shared_reference(graph, regions, port, arguments) {
				let link = Link(*output, port.try_into().unwrap());

				self.map.insert(link, argument);
			}
		}
	}

	fn handle_theta(&mut self, graph: &DataFlowGraph, theta_out: &ThetaOut) {
		let ThetaOut { input, results, .. } = theta_out;
		let ThetaIn { output, arguments } = graph.get(*input).as_theta_in().unwrap();

		self.handle_simple_reference(arguments, results, *input, *output);
	}

	pub fn run(&mut self, graph: &mut DataFlowGraph) {
		self.map.clear();

		for node in graph.nodes() {
			match node {
				Node::GammaOut(gamma_out) => self.handle_gamma(graph, gamma_out),
				Node::ThetaOut(theta_out) => self.handle_theta(graph, theta_out),

				_ => {}
			}
		}

		for node in graph.nodes_mut() {
			node.for_each_mut_argument(|old| {
				if let Some(new) = self.map.get(old) {
					*old = *new;
				}
			});
		}
	}
}

impl Default for FallthroughMover {
	fn default() -> Self {
		Self::new()
	}
}
