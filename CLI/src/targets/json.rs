use std::{io::Write, sync::Arc};

use parking_lot::Mutex;

use ir_graph::region::Module;
use json_target::JsonPrinter;

pub fn print(module: &Arc<Mutex<Module>>, out: &mut dyn Write) {
	let mut printer = JsonPrinter::new();

	printer.print(module, out).expect("module should print");
}
