use alloc::sync::Arc;

use parking_lot::Mutex;

use ir_graph::region::Function;
use ir_passes::catalog::Optimizations;
use ir_pipeline::{OptimizationConfiguration, Optimizer};
use web_assembly_lifter::WebAssemblyLifter;

pub fn lift(bytes: Vec<u8>, optimizations: Optimizations) -> Arc<Mutex<Function>> {
	let root = {
		let mut lifter = WebAssemblyLifter::new();

		lifter.run(&bytes)
	};

	drop(bytes);

	{
		let mut optimizer = Optimizer::new();
		let configuration = OptimizationConfiguration::with_maximum_rounds(optimizations);

		optimizer.run(&root, &configuration, &mut |_, _| false);
	}

	root
}
