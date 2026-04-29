use alloc::sync::Arc;

use parking_lot::Mutex;

use ir_graph::region::Function;
use ir_pipeline::Optimizer;
use turing_machine_lifter::TuringMachineLifter;

pub fn lift(source: &str, should_optimize: bool) -> Arc<Mutex<Function>> {
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
