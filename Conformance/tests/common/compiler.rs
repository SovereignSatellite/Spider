use alloc::sync::Arc;

use parking_lot::Mutex;

use ir_graph::region::Module;
use ir_pipeline::Optimizer;
use web_assembly_lifter::WebAssemblyLifter;

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

		self.optimizer.run(&module, optimize);

		module
	}
}
