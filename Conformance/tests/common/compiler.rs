use alloc::sync::Arc;

use parking_lot::Mutex;

use ir_graph::region::Module;
use ir_pipeline::Optimizer;
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

	pub fn run(&mut self, data: &[u8], should_optimize: bool) -> Arc<Mutex<Module>> {
		let module = self.lifter.run(data);

		self.optimizer.run(&module, should_optimize);

		module
	}
}
