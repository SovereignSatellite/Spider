//! The Policy trait separating allocator mechanism from target conventions.

use ir_graph::{Link, Node};

/// Target-specific allocation policy.
pub trait Policy {
	/// Returns the physical register kind masks, indexed by register id.
	fn registers(&self) -> &[u8];

	/// Returns the nonzero kind mask required by the given output port.
	fn kind(&self, node: &Node, port: u16) -> u8;

	/// Returns whether the node's value should occupy a register.
	fn should_materialize(&self, node: &Node) -> bool;

	/// Returns the operand whose register the given output port would like to
	/// reuse, or `None`.
	fn reuse_hint(&self, node: &Node, port: u16) -> Option<Link>;
}
