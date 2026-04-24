//! The Policy trait separating allocator mechanism from target conventions.

use ir_graph::Link;

/// Target-specific allocation policy.
///
/// Each physical register is described by a bitmask of the value kinds it
/// accepts; the allocator matches ports against registers whose mask is a
/// superset of the port's kind, preferring the most specific register first.
///
/// Register ids at or beyond `registers().len()` are spills. They accept every
/// kind; the allocator hands them out only when no physical register fits.
pub trait Policy {
	/// Returns the physical register kind masks, indexed by register id.
	fn registers(&self) -> &[u8];

	/// Returns the bitmask of kinds the given port requires.
	fn kind(&self, scope: usize, link: Link) -> u8;

	/// Returns whether a port should occupy a register.
	fn should_materialize(&self, scope: usize, link: Link) -> bool;
}
