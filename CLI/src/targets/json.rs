use std::{io::Write, sync::Arc};

use parking_lot::Mutex;

use ir_graph::region::Function;
use json_target::JsonPrinter;

pub fn print(root: &Arc<Mutex<Function>>, out: &mut dyn Write) {
	let mut printer = JsonPrinter::new();

	printer
		.print(root, out)
		.expect("failed to write the compiled function as JSON");
}
