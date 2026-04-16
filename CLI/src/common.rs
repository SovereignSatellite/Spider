use std::sync::Arc;

use parking_lot::Mutex;

use ir_graph::{Node, Region, control::Module};

use ir_visitor::{
	control::{
		dead_port_eliminator::DeadPortEliminator, invariant_port_mover::InvariantPortMover,
		region_identity,
	},
	isle, region_driver,
	topological_compactor::TopologicalCompactor,
};

struct Optimizer {
	topological_compactor: TopologicalCompactor,
	invariant_port_mover: InvariantPortMover,
	dead_port_eliminator: DeadPortEliminator,
}

impl Optimizer {
	fn new() -> Self {
		Self {
			topological_compactor: TopologicalCompactor::new(),
			invariant_port_mover: InvariantPortMover::new(),
			dead_port_eliminator: DeadPortEliminator::new(),
		}
	}

	fn apply_isle(nodes: &mut Vec<Node>) -> bool {
		let mut applied = false;
		let len = nodes.len();

		for id in (0..len.try_into().unwrap()).rev() {
			while isle::simplify_i32(nodes, id)
				|| isle::simplify_mutable(nodes, id)
				|| isle::simplify_table(nodes, id)
				|| isle::simplify_memory(nodes, id)
			{
				applied = true;
			}
		}

		applied
	}

	fn apply(&mut self, region: &mut Region) {
		loop {
			self.topological_compactor.run(region);
			self.invariant_port_mover.run(region.nodes_mut());
			self.dead_port_eliminator.run(region.nodes_mut());

			if !Self::apply_isle(region.nodes_mut()) {
				break;
			}

			region_identity::remove(region.nodes_mut());
		}
	}

	fn finalize(&mut self, region: &mut Region) {
		region_identity::insert(region);
		self.topological_compactor.run(region);
	}
}

pub fn process_module(module: &Arc<Mutex<Module>>, optimize: bool) {
	let mut optimizer = Optimizer::new();

	region_driver::run_module(module, &mut |mut region| {
		if optimize {
			optimizer.apply(&mut region);
		}

		optimizer.finalize(&mut region);
	});
}
