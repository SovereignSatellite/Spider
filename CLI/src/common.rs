use ir_graph::{DataFlowGraph, Link};
use ir_visitor::{
	control::{
		dead_port_eliminator::DeadPortEliminator, invariant_port_mover::InvariantPortMover,
		region_identity,
	},
	isle,
	topological_normalizer::TopologicalNormalizer,
};

fn run_isle_optimizations(graph: &mut DataFlowGraph) -> bool {
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

pub fn run_all_optimizations(graph: &mut DataFlowGraph, mut omega: u32) -> u32 {
	let mut topological_normalizer = TopologicalNormalizer::new();
	let mut invariant_port_mover = InvariantPortMover::new();
	let mut dead_port_eliminator = DeadPortEliminator::new();

	loop {
		omega = topological_normalizer.run(graph, omega);

		invariant_port_mover.run(graph);
		dead_port_eliminator.run(graph, Link(omega, 0));

		if !run_isle_optimizations(graph) {
			break;
		}

		region_identity::remove(graph);
	}

	omega
}

pub fn run_post_process(graph: &mut DataFlowGraph, omega: u32) {
	let mut topological_normalizer = TopologicalNormalizer::new();

	region_identity::insert(graph);

	topological_normalizer.run(graph, omega);
}
