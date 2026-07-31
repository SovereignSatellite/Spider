use alloc::sync::Arc;

use parking_lot::Mutex;

use ir_graph::{Region, region::Function};
use ir_passes::catalog::Optimizations;
use ir_pipeline::{OptimizationConfiguration, Optimizer};
use web_assembly_lifter::WebAssemblyLifter;

pub struct Compiler {
	lifter: WebAssemblyLifter,
	optimizer: Optimizer,
}

impl Compiler {
	pub fn new() -> Self {
		Self {
			lifter: WebAssemblyLifter::new(),
			optimizer: Optimizer::new(),
		}
	}

	pub fn run(
		&mut self,
		data: &[u8],
		optimizations: Optimizations,
		lower_target_nodes: &mut dyn FnMut(&mut Region, &Optimizations) -> bool,
	) -> Arc<Mutex<Function>> {
		let function = self.lifter.run(data);
		let configuration = OptimizationConfiguration::with_maximum_rounds(optimizations);

		self.optimizer
			.run(&function, &configuration, lower_target_nodes);

		function
	}
}
