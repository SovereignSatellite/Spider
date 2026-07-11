//! Fixpoint optimization driver that composes the region-local passes.

extern crate alloc;

use alloc::sync::Arc;

use parking_lot::Mutex;

use ir_graph::{Region, Shape, region::Function, region_driver};
use ir_passes::{
	motion::InvariantPortMover,
	normalize::{ConstantIsolator, DeadPortEliminator, TopologicalCompactor, identity},
	simplify::{CommonNodeEliminator, control_folder, isle},
};

/// Composes the region-local passes into a fixpoint optimization loop.
pub struct Optimizer {
	topological_compactor: TopologicalCompactor,
	invariant_port_mover: InvariantPortMover,
	common_node_eliminator: CommonNodeEliminator,
	constant_isolator: ConstantIsolator,
	dead_port_eliminator: DeadPortEliminator,
}

impl Optimizer {
	/// Creates a new optimizer.
	#[must_use]
	pub fn new() -> Self {
		Self {
			topological_compactor: TopologicalCompactor::new(),
			invariant_port_mover: InvariantPortMover::new(),
			common_node_eliminator: CommonNodeEliminator::new(),
			constant_isolator: ConstantIsolator::new(),
			dead_port_eliminator: DeadPortEliminator::new(),
		}
	}

	fn apply(&mut self, region: &mut Region, pass: &mut dyn FnMut(&mut Region) -> bool) {
		#[cfg(debug_assertions)]
		let mut iterations_allowed = region.nodes().len().saturating_mul(64).saturating_add(1024);

		loop {
			self.invariant_port_mover.run(region.nodes_mut());

			let folded = control_folder::run(region.nodes_mut());
			let simplified = isle::run(region.nodes_mut());
			let merged = self.common_node_eliminator.run(region.nodes_mut());

			if !folded && !simplified && !merged && !pass(region) {
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
		self.constant_isolator.run(region.nodes_mut());
		self.topological_compactor.run(region);
	}

	fn trim_children(&mut self, region: &Region) {
		for node in region.nodes() {
			match node.shape() {
				Shape::Plain | Shape::BranchResults(_) | Shape::RepeatResults(_) => {}
				Shape::Function(child) => {
					self.trim_tree_unbounded(Region::Function(Mutex::lock_arc(child)));
				}
				Shape::Match(child) => {
					for branch in &child.lock().branches {
						self.trim_tree_unbounded(Region::Branch(Mutex::lock_arc(branch)));
					}
				}
				Shape::Repeat(child) => {
					self.trim_tree_unbounded(Region::Repeat(Mutex::lock_arc(child)));
				}
			}
		}
	}

	fn trim_tree_unbounded(&mut self, region: Region) {
		stacker::maybe_grow(0x1_0000, 0x10_0000, || self.trim_tree(region));
	}

	fn trim_tree(&mut self, mut region: Region) {
		loop {
			self.trim_children(&region);

			self.topological_compactor.run(&mut region);

			if !self.dead_port_eliminator.run(region.nodes_mut()) {
				return;
			}
		}
	}

	/// Optimizes every region in the complete function tree, deepest first.
	/// When `should_optimize`, `pass` runs after generic rewrites settle and reports its changes.
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

		self.trim_tree_unbounded(Region::Function(Mutex::lock_arc(function)));
	}
}

impl Default for Optimizer {
	fn default() -> Self {
		Self::new()
	}
}
