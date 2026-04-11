use std::io::{Result, Write};

use super::LuauPrinter;

pub trait Print {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()>;
}

impl<T: Print> Print for &T {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		(*self).print(printer, out)
	}
}
