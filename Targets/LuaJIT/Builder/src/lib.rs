//! Builds `LuaJIT` functions from IR functions.

extern crate alloc;

use alloc::sync::Arc;

use parking_lot::Mutex;

use ir_graph::region::Function;
use luajit_tree::expression;

use self::{emitter::Emitter, policy::LuaJITPolicy};

mod assignment_simplifier;
mod code_handler;
mod data_handler;
mod emitter;
mod policy;

/// Builds a `LuaJIT` function from an IR function.
pub struct LuaJITBuilder {
	allocator: ir_allocator::Allocator,
	policy: LuaJITPolicy,
}

impl LuaJITBuilder {
	/// Creates a new `LuaJIT` builder.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			allocator: ir_allocator::Allocator::new(),
			policy: LuaJITPolicy::new(),
		}
	}

	/// Builds a `LuaJIT` function.
	pub fn run(&mut self, function: &Arc<Mutex<Function>>) -> expression::Function {
		let guard = function.lock();
		let scope = Arc::as_ptr(function) as usize;
		let mut emitter = Emitter::new(&mut self.allocator, &self.policy);

		emitter.emit_function(&guard, scope)
	}
}

impl Default for LuaJITBuilder {
	fn default() -> Self {
		Self::new()
	}
}
