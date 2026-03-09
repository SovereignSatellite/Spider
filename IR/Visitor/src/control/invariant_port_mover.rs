//! Invariant port motion.

use hashbrown::HashMap;
use ir_graph::{
	DataFlowGraph, Link, Node,
	control::{GammaIn, GammaOut, RegionOut, ThetaIn, ThetaOut},
};

/// Moves invariant ports out of control flow regions.
pub struct InvariantPortMover {
	map: HashMap<Link, Link>,
}

impl InvariantPortMover {
	/// Creates a new invariant port mover.
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

	/// Runs the invariant port motion pass on the graph.
	pub fn run(&mut self, graph: &mut DataFlowGraph) {
		self.map.clear();

		for node in graph.nodes() {
			match node {
				Node::GammaOut(gamma_out) => self.handle_gamma(graph, gamma_out),
				Node::ThetaOut(theta_out) => self.handle_theta(graph, theta_out),

				Node::Apply(_)
				| Node::F32(_)
				| Node::F64(_)
				| Node::Fence(_)
				| Node::GammaIn(_)
				| Node::GlobalGet(_)
				| Node::GlobalNew(_)
				| Node::GlobalSet(_)
				| Node::Host(_)
				| Node::I32(_)
				| Node::I64(_)
				| Node::Identity(_)
				| Node::Import(_)
				| Node::IntegerBinaryOperation(_)
				| Node::IntegerCompareOperation(_)
				| Node::IntegerConvertToNumber(_)
				| Node::IntegerExtend(_)
				| Node::IntegerNarrow(_)
				| Node::IntegerTransmuteToNumber(_)
				| Node::IntegerUnaryOperation(_)
				| Node::IntegerWiden(_)
				| Node::LambdaIn(_)
				| Node::LambdaOut(_)
				| Node::MemoryCopy(_)
				| Node::MemoryDrop(_)
				| Node::MemoryFill(_)
				| Node::MemoryGrow(_)
				| Node::MemoryLoad(_)
				| Node::MemoryNew(_)
				| Node::MemorySize(_)
				| Node::MemoryStore(_)
				| Node::Null
				| Node::NumberBinaryOperation(_)
				| Node::NumberCompareOperation(_)
				| Node::NumberNarrow(_)
				| Node::NumberTransmuteToInteger(_)
				| Node::NumberTruncateToInteger(_)
				| Node::NumberUnaryOperation(_)
				| Node::NumberWiden(_)
				| Node::OmegaIn(_)
				| Node::OmegaOut(_)
				| Node::RefIsNull(_)
				| Node::RegionIn(_)
				| Node::RegionOut(_)
				| Node::TableCopy(_)
				| Node::TableDrop(_)
				| Node::TableFill(_)
				| Node::TableGet(_)
				| Node::TableGrow(_)
				| Node::TableNew(_)
				| Node::TableSet(_)
				| Node::TableSize(_)
				| Node::ThetaIn(_)
				| Node::Trap => {}
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

impl Default for InvariantPortMover {
	fn default() -> Self {
		Self::new()
	}
}
