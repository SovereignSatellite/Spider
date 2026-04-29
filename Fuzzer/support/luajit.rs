use std::io;

use luajit_builder::LuaJITBuilder;
use luajit_printer::LuaJITPrinter;

use crate::optimization::optimize;

pub fn compile(bytes: &[u8], should_optimize: bool) {
	let function = {
		let root = optimize(bytes, should_optimize);
		let mut builder = LuaJITBuilder::new();

		builder.run(&root)
	};

	let mut printer = LuaJITPrinter::new();
	let mut output = io::sink();

	printer
		.print(&function, &mut output)
		.expect("LuaJIT should print");
}
