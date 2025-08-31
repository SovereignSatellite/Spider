use data_flow_builder::DataFlowBuilder;
use data_flow_graph::{DataFlowGraph, Link};
use data_flow_visitor::{
	dead_port_eliminator::DeadPortEliminator, fallthrough_mover::FallthroughMover, region_identity,
	topological_normalizer::TopologicalNormalizer,
};

pub struct Loader {
	data_flow_builder: DataFlowBuilder,

	fallthrough_mover: FallthroughMover,
	dead_port_eliminator: DeadPortEliminator,
	topological_normalizer: TopologicalNormalizer,
}

impl Loader {
	pub fn new() -> Self {
		Self {
			data_flow_builder: DataFlowBuilder::new(),

			fallthrough_mover: FallthroughMover::new(),
			dead_port_eliminator: DeadPortEliminator::new(),
			topological_normalizer: TopologicalNormalizer::new(),
		}
	}

	pub fn run(&mut self, data: &[u8]) -> DataFlowGraph {
		let mut graph = DataFlowGraph::new();

		let omega = self.data_flow_builder.run(&mut graph, data);
		let omega = self.topological_normalizer.run(&mut graph, omega);

		self.fallthrough_mover.run(&mut graph);
		self.dead_port_eliminator.run(&mut graph, Link(omega, 0));

		region_identity::insert(&mut graph);

		self.topological_normalizer.run(&mut graph, omega);

		graph
	}
}
