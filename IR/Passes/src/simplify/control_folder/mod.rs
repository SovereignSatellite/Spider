//! Control-flow folding for constant-dispatched matches and run-once repeats.

use ir_graph::{Link, Node, tracer::identity_source};

use crate::catalog::Optimizations;

mod inline;
mod matcher;
mod repeat;

/// Folds constant-condition control flow in the region, reporting whether anything changed.
#[must_use = "propagate whether this pass changed the graph"]
pub fn run(nodes: &mut Vec<Node>, optimizations: &Optimizations) -> bool {
	let original_node_count = nodes.len();
	let mut folded = false;

	for identifier in 0..original_node_count {
		folded |= (optimizations.fold_constant_match && matcher::fold(nodes, identifier))
			|| (optimizations.fold_exiting_repeat && repeat::fold(nodes, identifier));
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
