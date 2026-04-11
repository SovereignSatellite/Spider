use std::io::{Result, Write};

use super::LuaJITPrinter;

pub trait Print {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()>;
}

impl<T: Print> Print for &T {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		(*self).print(printer, out)
	}
}
