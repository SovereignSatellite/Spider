use turing_machine_lifter::TuringMachineLifter;

pub fn lift(bytes: &[u8]) {
	let source = str::from_utf8(bytes).expect("source should be valid UTF-8");
	let mut lifter = TuringMachineLifter::new();

	let _root = lifter.run(source);
}
