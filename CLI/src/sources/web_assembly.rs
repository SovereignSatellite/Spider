use ir_graph::DataFlowGraph;
use wasmparser::Validator;
use web_assembly_lifter::WebAssemblyLifter;

pub fn lift(data: &[u8]) -> (DataFlowGraph, u32) {
	Validator::new()
		.validate_all(data)
		.expect("`file` should be a WebAssembly binary");

	let mut graph = DataFlowGraph::new();
	let omega = {
		let mut lifter = WebAssemblyLifter::new();

		lifter.run(&mut graph, data)
	};

	(graph, omega)
}
