use alloc::sync::Arc;

use parking_lot::Mutex;

use ir_graph::{Region, region::Function};
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

	pub fn run(
		&mut self,
		data: &[u8],
		should_optimize: bool,
		pass: &mut dyn FnMut(&mut Region) -> bool,
	) -> Arc<Mutex<Function>> {
		let function = self.lifter.run(data);

		self.optimizer.run(&function, should_optimize, pass);

		function
	}
}
