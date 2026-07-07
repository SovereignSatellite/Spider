//! Fixpoint optimization driver that composes the region-local passes.

extern crate alloc;

use alloc::sync::Arc;

use parking_lot::Mutex;

use ir_graph::{Region, region::Function, region_driver};
use ir_passes::{
	control_folder, dead_port_eliminator::DeadPortEliminator, identity,
	invariant_port_mover::InvariantPortMover, isle, topological_compactor::TopologicalCompactor,
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

	fn apply(&mut self, region: &mut Region, pass: &mut dyn FnMut(&mut Region) -> bool) {
		#[cfg(debug_assertions)]
		let mut iterations_allowed = region.nodes().len().saturating_mul(64).saturating_add(1024);

		loop {
			self.topological_compactor.run(region);
			self.invariant_port_mover.run(region.nodes_mut());
			self.dead_port_eliminator.run(region.nodes_mut());

			let folded = control_folder::run(region.nodes_mut());
			let simplified = isle::run(region.nodes_mut());

			if !folded && !simplified && !pass(region) {
				break;
			}

			#[cfg(debug_assertions)]
			{
				assert!(
					iterations_allowed > 0,
					"optimizer did not converge; a non-shrinking rewrite is likely oscillating"
				);

				iterations_allowed -= 1;
			}

			identity::remove(region.nodes_mut());
		}
	}

	fn finalize(&mut self, region: &mut Region) {
		identity::insert(region);
		self.topological_compactor.run(region);
	}

	/// Runs the optimization pipeline over every region in the function.
	///
	/// `pass` runs each round once the generic passes settle, letting a target fold its own
	/// lowering into the same fixpoint; it reports whether it changed the region.
	pub fn run(
		&mut self,
		function: &Arc<Mutex<Function>>,
		should_optimize: bool,
		pass: &mut dyn FnMut(&mut Region) -> bool,
	) {
		region_driver::run_function(function, &mut |mut region| {
			if should_optimize {
				self.apply(&mut region, pass);
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
