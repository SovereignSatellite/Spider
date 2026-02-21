use ir_graph::{DataFlowGraph, Link};
use ir_visitor::{
	control::{
		dead_port_eliminator::DeadPortEliminator, invariant_port_mover::InvariantPortMover,
		region_identity,
	},
	isle,
	topological_normalizer::TopologicalNormalizer,
};
use web_assembly_lifter::WebAssemblyLifter;

struct Optimizer {
	invariant_port_mover: InvariantPortMover,
	dead_port_eliminator: DeadPortEliminator,
	topological_normalizer: TopologicalNormalizer,
}

impl Optimizer {
	fn new() -> Self {
		Self {
			invariant_port_mover: InvariantPortMover::new(),
			dead_port_eliminator: DeadPortEliminator::new(),
			topological_normalizer: TopologicalNormalizer::new(),
		}
	}

	fn apply_isle(graph: &mut DataFlowGraph) -> bool {
		let mut applied = false;
		let len = graph.len();

		for id in (0..len.try_into().unwrap()).rev() {
			while isle::simplify_i32(graph, id)
				|| isle::simplify_global(graph, id)
				|| isle::simplify_table(graph, id)
			{
				applied = true;
			}
		}

		applied
	}

	fn apply(&mut self, graph: &mut DataFlowGraph, mut omega: u32) -> u32 {
		loop {
			omega = self.topological_normalizer.run(graph, omega);

			self.invariant_port_mover.run(graph);
			self.dead_port_eliminator.run(graph, Link(omega, 0));

			if !Self::apply_isle(graph) {
				break;
			}

			region_identity::remove(graph);
		}

		omega
	}

	fn finalize(&mut self, graph: &mut DataFlowGraph, omega: u32) {
		region_identity::insert(graph);

		self.topological_normalizer.run(graph, omega);
	}
}

pub struct Compiler {
	web_assembly_lifter: WebAssemblyLifter,
	optimizer: Optimizer,
}

impl Compiler {
	pub fn new() -> Self {
		Self {
			web_assembly_lifter: WebAssemblyLifter::new(),
			optimizer: Optimizer::new(),
		}
	}

	pub fn run(&mut self, data: &[u8]) -> DataFlowGraph {
		let mut graph = DataFlowGraph::new();

		let omega = self.web_assembly_lifter.run(&mut graph, data);
		let omega = self.optimizer.apply(&mut graph, omega);

		self.optimizer.finalize(&mut graph, omega);

		graph
	}
}
