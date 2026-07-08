//! Prints `Luau` functions.

extern crate alloc;

use alloc::sync::Arc;
use std::io::{Result, Write};

use hashbrown::HashMap;

use luau_tree::expression::{Function, Name};

use self::print::Print as _;

mod expression;
mod print;
mod statement;

/// Runtime library section management.
pub mod library;

/// Prints a `Luau` function into a writer.
pub struct LuauPrinter {
	names: HashMap<Name, Arc<str>>,
	depth: u16,
}

impl LuauPrinter {
	/// Creates a new `LuauPrinter`.
	#[must_use]
	pub fn new() -> Self {
		Self {
			names: HashMap::new(),
			depth: 0,
		}
	}

	/// Writes the current indentation level to the writer.
	///
	/// # Errors
	///
	/// Returns any IO errors that the `out` produces during the process.
	pub fn write_indent(&self, out: &mut dyn Write) -> Result<()> {
		(0..self.depth).try_for_each(|_| write!(out, "\t"))
	}

	/// Returns the name associated with the given `Name`, if any.
	pub fn get_name(&self, name: Name) -> Option<&str> {
		self.names.get(&name).map(Arc::as_ref)
	}

	/// Increases the indentation level by one.
	pub const fn indent(&mut self) {
		self.depth = self.depth.wrapping_add(1);
	}

	/// Decreases the indentation level by one.
	pub const fn outdent(&mut self) {
		self.depth = self.depth.wrapping_sub(1);
	}

	/// Prints the function into the writer.
	///
	/// # Errors
	///
	/// Returns any IO errors that the `out` produces during the process.
	pub fn print(&mut self, function: &Function, out: &mut dyn Write) -> Result<()> {
		self.write_indent(out)?;
		write!(out, "local module = ")?;
		function.print(self, out)?;
		writeln!(out)
	}
}

impl Default for LuauPrinter {
	fn default() -> Self {
		Self::new()
	}
}
