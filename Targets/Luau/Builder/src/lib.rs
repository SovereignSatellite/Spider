//! Builds `Luau` trees from IR modules.

extern crate alloc;

use alloc::sync::Arc;

use parking_lot::Mutex;

use ir_graph::region::Module;
use luau_tree::LuauTree;

use self::{emitter::Emitter, policy::LuauPolicy};

mod assignment_simplifier;
mod code_handler;
mod data_handler;
mod emitter;
mod policy;

/// Builds a Luau tree from an IR module.
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

	/// Builds a Luau tree.
	pub fn run(&mut self, module: &Arc<Mutex<Module>>) -> LuauTree {
		let guard = module.lock();
		let scope = Arc::as_ptr(module) as usize;
		let mut emitter = Emitter::new(&mut self.allocator, &self.policy);

		emitter.emit_module(&guard, scope)
	}
}

impl Default for LuauBuilder {
	fn default() -> Self {
		Self::new()
	}
}
