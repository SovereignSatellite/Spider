//! Control-flow folding for constant-dispatched matches and run-once repeats.

use ir_graph::{Link, Node, tracer::identity_source};

mod inline;
mod matcher;
mod repeat;

/// Folds constant-condition control flow in the region, reporting whether anything changed.
pub fn run(nodes: &mut Vec<Node>) -> bool {
	let original = nodes.len();
	let mut folded = false;

	for id in 0..original {
		folded |= matcher::fold(nodes, id) || repeat::fold(nodes, id);
	}

	folded
}

fn constant_at(nodes: &[Node], link: Link) -> Option<i32> {
	let source = identity_source(nodes, link);

	if let &Node::I32(value) = &nodes[usize::try_from(source.0).unwrap()] {
		Some(value)
	} else {
		None
	}
}
