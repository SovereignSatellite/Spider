use alloc::sync::Arc;

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
use web_assembly_lifter::WebAssemblyLifter;

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

	pub fn run(&mut self, data: &[u8], optimize: bool) -> Arc<Mutex<Module>> {
		let module = self.web_assembly_lifter.run(data);

		region_driver::run_module(&module, &mut |mut region| {
			if optimize {
				self.optimizer.apply(&mut region);
			}

			self.optimizer.finalize(&mut region);
		});

		module
	}
}
