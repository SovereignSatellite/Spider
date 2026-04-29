use alloc::sync::Arc;
use std::io;

use parking_lot::Mutex;

use ir_graph::region::Function;
use luau_builder::LuauBuilder;
use luau_printer::LuauPrinter;

pub fn compile(root: &Arc<Mutex<Function>>) {
	let function = {
		let mut builder = LuauBuilder::new();

		builder.run(root)
	};

	let mut printer = LuauPrinter::new();
	let mut output = io::sink();

	printer
		.print(&function, &mut output)
		.expect("Luau should print");
}
