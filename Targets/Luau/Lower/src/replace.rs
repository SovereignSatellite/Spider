//! In-place replacement of a lowered node with its expansion.

use core::mem;

use ir_graph::{Link, Node, operation::Identity};

fn replace_with_identity(nodes: &mut [Node], destination: u32, sources: &[Link]) {
	let sources = sources.iter().copied().collect();

	nodes[usize::try_from(destination).unwrap()] = Node::Identity(Identity { sources });
}

fn replace_with_direct(nodes: &mut [Node], destination: u32, source: u32) {
	let source = mem::take(&mut nodes[usize::try_from(source).unwrap()]);

	nodes[usize::try_from(destination).unwrap()] = source;
}

/// Replaces the node at `destination` with the expansion rooted at `sources`.
///
/// A single fresh result port is spliced in directly; anything else becomes an
/// identity node forwarding each original output port to its replacement link.
pub fn replace_node(nodes: &mut [Node], destination: u32, sources: &[Link]) {
	if let &[source] = sources
		&& source.1 == 0
		&& source.0 > destination
	{
		replace_with_direct(nodes, destination, source.0);
	} else {
		replace_with_identity(nodes, destination, sources);
	}
}
