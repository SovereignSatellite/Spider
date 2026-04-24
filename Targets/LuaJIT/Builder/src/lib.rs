//! Builds `LuaJIT` trees from IR modules.

extern crate alloc;

use alloc::sync::Arc;

use parking_lot::Mutex;

use ir_graph::region::Module;
use luajit_tree::LuaJITTree;

use self::{emitter::Emitter, policy::LuaJITPolicy};

mod assignment_simplifier;
mod code_handler;
mod data_handler;
mod emitter;
mod policy;

/// Builds a `LuaJIT` tree from an IR module.
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

	/// Builds a `LuaJIT` tree.
	pub fn run(&mut self, module: &Arc<Mutex<Module>>) -> LuaJITTree {
		let guard = module.lock();
		let scope = Arc::as_ptr(module) as usize;
		let mut emitter = Emitter::new(&mut self.allocator, &self.policy);

		emitter.emit_module(&guard, scope)
	}
}

impl Default for LuaJITBuilder {
	fn default() -> Self {
		Self::new()
	}
}
