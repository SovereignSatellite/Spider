use alloc::sync::Arc;

use parking_lot::Mutex;

use ir_graph::region::Function;
use ir_pipeline::Optimizer;
use web_assembly_lifter::WebAssemblyLifter;

pub fn lift(bytes: &[u8], should_optimize: bool) -> Arc<Mutex<Function>> {
	let root = {
		let mut lifter = WebAssemblyLifter::new();

		lifter.run(bytes)
	};

	{
		let mut optimizer = Optimizer::new();

		optimizer.run(&root, should_optimize);
	}

	root
}
