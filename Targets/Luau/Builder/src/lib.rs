//! Builds `Luau` functions from IR functions.

extern crate alloc;

use alloc::sync::Arc;

use parking_lot::Mutex;

use ir_graph::region::Function;
use luau_tree::expression;

use self::{emitter::Emitter, policy::LuauPolicy};

mod assignment_simplifier;
mod code_handler;
mod data_handler;
mod emitter;
mod policy;

/// Builds a `Luau` function from an IR function.
pub struct LuauBuilder {
	allocator: ir_allocator::Allocator,
	policy: LuauPolicy,
}

impl LuauBuilder {
	/// Creates a new Luau builder.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			allocator: ir_allocator::Allocator::new(),
			policy: LuauPolicy::new(),
		}
	}

	/// Builds a `Luau` function.
	pub fn run(&mut self, function: &Arc<Mutex<Function>>) -> expression::Function {
		let guard = function.lock();
		let scope = Arc::as_ptr(function) as usize;
		let mut emitter = Emitter::new(&mut self.allocator, &self.policy);

		emitter.emit_function(&guard, scope)
	}
}

impl Default for LuauBuilder {
	fn default() -> Self {
		Self::new()
	}
}
