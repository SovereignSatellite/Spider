//! Fixpoint optimization driver that composes the region-local passes.

extern crate alloc;

use alloc::sync::Arc;

use parking_lot::Mutex;

use ir_graph::{Node, Region, region::Function, region_driver};
use ir_passes::{
	dead_port_eliminator::DeadPortEliminator, identity, invariant_port_mover::InvariantPortMover,
	isle, topological_compactor::TopologicalCompactor,
};

/// Composes the region-local passes into a fixpoint optimization loop.
pub struct Optimizer {
	topological_compactor: TopologicalCompactor,
	invariant_port_mover: InvariantPortMover,
	dead_port_eliminator: DeadPortEliminator,
}

impl Optimizer {
	/// Creates a new optimizer.
	#[must_use]
	pub fn new() -> Self {
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

			identity::remove(region.nodes_mut());
		}
	}

	fn finalize(&mut self, region: &mut Region) {
		identity::insert(region);
		self.topological_compactor.run(region);
	}

	/// Runs the optimization pipeline over every region in the function.
	pub fn run(&mut self, function: &Arc<Mutex<Function>>, should_optimize: bool) {
		region_driver::run_function(function, &mut |mut region| {
			if should_optimize {
				self.apply(&mut region);
			}

			self.finalize(&mut region);
		});
	}
}

impl Default for Optimizer {
	fn default() -> Self {
		Self::new()
	}
}
