use alloc::sync::Arc;

use parking_lot::Mutex;

use ir_graph::region::Function;
use ir_passes::catalog::Optimizations;
use ir_pipeline::{OptimizationConfiguration, Optimizer};
use turing_machine_lifter::TuringMachineLifter;

pub fn lift(source: String, optimizations: Optimizations) -> Arc<Mutex<Function>> {
	let root = {
		let mut lifter = TuringMachineLifter::new();

		lifter.run(&source)
	};

	drop(source);

	{
		let mut optimizer = Optimizer::new();
		let configuration = OptimizationConfiguration::with_maximum_rounds(optimizations);

		optimizer.run(&root, &configuration, &mut |_, _| false);
	}

	root
}
