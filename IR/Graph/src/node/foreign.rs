//! The open extension point for operations outside the core universe.

use core::any::Any;

use crate::Link;

/// Extends the graph with concrete, type-identified operations.
/// Unrecognized operations remain semantically opaque, but generic traversals see their declared ports.
pub trait Foreign: Any {
	/// Returns the identifier string for this node type.
	#[must_use]
	fn identifier(&self) -> &'static str;

	/// Returns the number of output ports.
	#[must_use]
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
