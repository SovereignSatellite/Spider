mod expression;
mod print;
mod statement;

pub mod library;

use std::{
	io::{Result, Write},
	sync::Arc,
};

use hashbrown::HashMap;
use luau_tree::{LuauTree, expression::Name};

use self::print::Print;

pub struct LuauPrinter {
	names: HashMap<Name, Arc<str>>,
	depth: u16,
}

impl LuauPrinter {
	#[must_use]
	pub fn new() -> Self {
		Self {
			names: HashMap::new(),
			depth: 0,
		}
	}

	pub(crate) fn tab(&self, out: &mut dyn Write) -> Result<()> {
		(0..self.depth).try_for_each(|_| write!(out, "\t"))
	}

	pub fn get_name(&self, name: Name) -> Option<&str> {
		self.names.get(&name).map(Arc::as_ref)
	}

	pub const fn indent(&mut self) {
		self.depth = self.depth.wrapping_add(1);
	}

	pub const fn outdent(&mut self) {
		self.depth = self.depth.wrapping_sub(1);
	}

	/// # Errors
	///
	/// Returns any IO errors that the `out` produces during the process.
	pub fn print(&mut self, tree: &LuauTree, out: &mut dyn Write) -> Result<()> {
		tree.print(self, out)
	}
}

impl Default for LuauPrinter {
	fn default() -> Self {
		Self::new()
	}
}
