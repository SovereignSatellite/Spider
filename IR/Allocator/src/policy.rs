//! The Policy trait separating allocator mechanism from target conventions.

use ir_graph::Node;

/// Target-specific allocation policy.
///
/// Each physical register is described by a bitmask of the value kinds it
/// accepts; the allocator matches values against registers whose mask is a
/// superset of the value's kind, preferring the register that accepts the
/// fewest kinds (the most specific fit) first.
///
/// Register ids at or beyond `registers().len()` are spills. They accept every
/// kind; the allocator hands them out only when no physical register fits.
pub trait Policy {
	/// Returns the physical register kind masks, indexed by register id.
	fn registers(&self) -> &[u8];

	/// Returns the nonzero kind mask required by the given output port.
	///
	/// Consulted for every materialized port, including `Match` and `Repeat`
	/// outputs whose regions the allocator holds locked. Implementations
	/// must answer from the node surface without locking region interiors.
	fn kind(&self, node: &Node, port: u16) -> u8;

	/// Returns whether the node's value should occupy a register.
	///
	/// A node that should not is deferred: it gets no register, its interval is
	/// never colored, and reads of it flow through to its operands, so the target
	/// inlines it at its single use. Only single-output nodes defer.
	fn should_materialize(&self, node: &Node) -> bool;
}
