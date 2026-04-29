use alloc::sync::Arc;

use parking_lot::Mutex;

use ir_graph::region::Function;
use ir_pipeline::Optimizer;
use turing_machine_lifter::TuringMachineLifter;

pub fn optimize(bytes: &[u8], should_optimize: bool) -> Arc<Mutex<Function>> {
	let source = str::from_utf8(bytes).expect("source should be valid UTF-8");
	let root = {
		let mut lifter = TuringMachineLifter::new();

		lifter.run(source)
	};

	{
		let mut optimizer = Optimizer::new();

		optimizer.run(&root, should_optimize);
	}

	root
}
