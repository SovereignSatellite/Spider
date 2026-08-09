use alloc::sync::Arc;
use std::io;

use parking_lot::Mutex;

use ir_graph::region::Function;
use luajit_builder::LuaJITBuilder;
use luajit_printer::LuaJITPrinter;

pub fn compile(root: Arc<Mutex<Function>>) {
	let function = {
		let mut builder = LuaJITBuilder::new();

		builder.run(&root)
	};

	drop(root);

	let mut printer = LuaJITPrinter::new();
	let mut output = io::sink();

	printer
		.print(&function, &mut output)
		.expect("LuaJIT should print");
}
