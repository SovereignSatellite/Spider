use ir_graph::DataFlowGraph;
use turing_machine_lifter::TuringMachineLifter;

fn has_balanced_brackets(source: &str) -> bool {
	let mut open = 0;

	for character in source.chars() {
		match character {
			']' if open == 0 => return false,

			']' => open -= 1,
			'[' => open += 1,

			_ => {}
		}
	}

	open == 0
}

pub fn lift(data: &[u8]) -> (DataFlowGraph, u32) {
	let source = str::from_utf8(data).expect("`file` should be a valid UTF-8 string");

	assert!(
		has_balanced_brackets(source),
		"`file` should have balanced brackets"
	);

	let mut graph = DataFlowGraph::new();
	let omega = {
		let mut lifter = TuringMachineLifter::new();

		lifter.run(&mut graph, source)
	};

	(graph, omega)
}
