//! The open extension point for operations outside the core universe.

use core::any::Any;

use crate::Link;

/// Trait for nodes that live outside the core computation universe.
///
/// Source frontends, target lowerings, IO operations, and FFI bindings all
/// express their domain-specific constructs as concrete `Foreign` types.
/// The core optimizer treats unrecognized `Foreign` nodes as black boxes;
/// specialized passes downcast via `TypeId` to match on concrete types.
pub trait Foreign: Any {
	/// Returns the name of this node type.
	fn identifier(&self) -> &'static str;

	/// Returns the number of output ports.
	fn result_count(&self) -> u16;

	/// Calls `handler` for each outer link.
	fn for_each_outer(&self, handler: &mut dyn FnMut(Link)) {
		let _ = handler;
	}

	/// Calls `handler` for each mutable outer link.
	fn for_each_mut_outer(&mut self, handler: &mut dyn FnMut(&mut Link)) {
		let _ = handler;
	}
}
