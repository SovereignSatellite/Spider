//! In-place replacement of a lowered node with its expansion.

use core::iter;

use ir_graph::{
	Link, Node,
	operation::{Fence, Identity},
};

/// Replaces the node at `destination` with an identity forwarding each output port to its replacement link.
pub fn replace_node(nodes: &mut [Node], destination: u32, sources: &[Link]) {
	let sources = sources.iter().copied().collect();

	nodes[usize::try_from(destination).unwrap()] = Node::Identity(Identity { sources });
}

/// Lowers a read to `[value, state]`. `value` is forwarded directly so later passes can simplify
/// its producer; `state` forwards the reference but fences the side-effecting `reads`.
pub fn replace_read(
	nodes: &mut Vec<Node>,
	destination: u32,
	value: Link,
	reference: Link,
	reads: &[Link],
) {
	let fence = Fence::add_into(
		nodes,
		iter::once(reference).chain(reads.iter().copied()).collect(),
	);

	replace_node(nodes, destination, &[value, Link(fence, 0)]);
}
