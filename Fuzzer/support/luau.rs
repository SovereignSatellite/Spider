use std::io;

use luau_builder::LuauBuilder;
use luau_printer::LuauPrinter;

use crate::optimization::optimize;

pub fn compile(bytes: &[u8], should_optimize: bool) {
	let function = {
		let root = optimize(bytes, should_optimize);
		let mut builder = LuauBuilder::new();

		builder.run(&root)
	};

	let mut printer = LuauPrinter::new();
	let mut output = io::sink();

	printer
		.print(&function, &mut output)
		.expect("Luau should print");
}
