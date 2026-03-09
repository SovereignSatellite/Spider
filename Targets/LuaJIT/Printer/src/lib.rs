//! Prints `LuaJIT` trees.

extern crate alloc;

mod expression;
mod print;
mod statement;

/// Runtime library section management.
pub mod library;

use alloc::sync::Arc;
use std::io::{Result, Write};

use hashbrown::HashMap;
use luajit_tree::{LuaJITTree, expression::Name};

use self::print::Print as _;

/// Prints a `LuaJIT` tree into a writer.
pub struct LuaJITPrinter {
	names: HashMap<Name, Arc<str>>,
	depth: u16,
}

impl LuaJITPrinter {
	/// Creates a new `LuaJITPrinter`.
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
	pub fn tab(&self, out: &mut dyn Write) -> Result<()> {
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

	/// Prints the tree into the writer.
	///
	/// # Errors
	///
	/// Returns any IO errors that the `out` produces during the process.
	pub fn print(&mut self, tree: &LuaJITTree, out: &mut dyn Write) -> Result<()> {
		tree.print(self, out)
	}
}

impl Default for LuaJITPrinter {
	fn default() -> Self {
		Self::new()
	}
}
