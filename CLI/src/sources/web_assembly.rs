use std::sync::Arc;

use parking_lot::Mutex;
use wasmparser::Validator;

use ir_graph::region::Function;
use web_assembly_lifter::WebAssemblyLifter;

pub fn lift(data: &[u8]) -> Arc<Mutex<Function>> {
	Validator::new()
		.validate_all(data)
		.expect("`file` should be a WebAssembly binary");

	let mut lifter = WebAssemblyLifter::new();

	lifter.run(data)
}
