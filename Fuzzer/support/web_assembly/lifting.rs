use web_assembly_lifter::WebAssemblyLifter;

pub fn lift(bytes: &[u8]) {
	let mut lifter = WebAssemblyLifter::new();

	let _root = lifter.run(bytes);
}
