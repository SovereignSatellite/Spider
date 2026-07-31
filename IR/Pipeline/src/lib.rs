//! Fixpoint optimization driver that composes the region-local passes.

extern crate alloc;

use alloc::sync::Arc;

use parking_lot::Mutex;

use ir_graph::{Region, Shape, region::Function, region_driver};
use ir_passes::{
	catalog::{OptimizationStages, Optimizations},
	motion::InvariantPortMover,
	normalize::{ConstantIsolator, DeadPortEliminator, TopologicalCompactor, identity},
	simplify::{CommonNodeEliminator, control_folder, isle},
};

/// Provide the resolved policy consumed by one optimizer run.
#[derive(Debug, PartialEq, Eq)]
pub struct OptimizationConfiguration {
	/// Select the enabled transformations.
	pub optimizations: Optimizations,
	/// Limit graph-changing fixpoint rounds per region.
	pub round_limit: u32,
}

impl OptimizationConfiguration {
	/// Create a configuration with the maximum rewrite-round limit.
	#[must_use]
	pub const fn with_maximum_rounds(optimizations: Optimizations) -> Self {
		Self {
			optimizations,
			round_limit: u32::MAX,
		}
	}
}

/// Compose the region-local passes into a fixpoint optimization loop.
pub struct Optimizer {
	topological_compactor: TopologicalCompactor,
	invariant_port_mover: InvariantPortMover,
	common_node_eliminator: CommonNodeEliminator,
	constant_isolator: ConstantIsolator,
	dead_port_eliminator: DeadPortEliminator,
}

impl Optimizer {
	/// Create reusable optimizer scratch state.
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

	fn run_generic_round(
		&mut self,
		region: &mut Region,
		optimizations: &Optimizations,
		optimization_stages: OptimizationStages,
	) -> bool {
		let mut changed = false;

		changed |= optimization_stages.control_folder
			&& control_folder::run(region.nodes_mut(), optimizations);
		changed |=
			optimizations.move_invariant_ports && self.invariant_port_mover.run(region.nodes_mut());

		changed |= self.dead_port_eliminator.run(region.nodes_mut());

		changed |= optimization_stages.match_output_reductions
			&& isle::reduce_match_outputs(region.nodes_mut(), optimizations);
		changed |= optimizations.eliminate_common_nodes
			&& self.common_node_eliminator.run(region.nodes_mut());
		changed |=
			optimization_stages.isle_rewrites && isle::run(region.nodes_mut(), optimizations);

		changed
	}

	fn optimize_region(
		&mut self,
		region: &mut Region,
		configuration: &OptimizationConfiguration,
		optimization_stages: OptimizationStages,
		lower_target_nodes: &mut dyn FnMut(&mut Region, &Optimizations) -> bool,
	) {
		let optimizations = &configuration.optimizations;

		for _ in 0..configuration.round_limit {
			if self.run_generic_round(region, optimizations, optimization_stages) {
				identity::remove(region.nodes_mut());
				continue;
			}

			if optimization_stages.target_lowering && lower_target_nodes(region, optimizations) {
				identity::remove(region.nodes_mut());
				continue;
			}

			break;
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

	/// Optimize every region in the complete function tree, deepest first.
	/// Run target lowering only after selected generic rewrites settle.
	pub fn run(
		&mut self,
		function: &Arc<Mutex<Function>>,
		configuration: &OptimizationConfiguration,
		lower_target_nodes: &mut dyn FnMut(&mut Region, &Optimizations) -> bool,
	) {
		let optimization_stages = configuration.optimizations.stages();
		let has_optional_work =
			optimization_stages.any_optimization && configuration.round_limit != 0;

		region_driver::run_function(function, &mut |mut region| {
			if has_optional_work {
				self.optimize_region(
					&mut region,
					configuration,
					optimization_stages,
					lower_target_nodes,
				);
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
