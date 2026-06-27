//! Boolean `and`/`or` lowered to two-branch select matches.

use ir_graph::{Link, Node, region::Match};
use luau_foreign::BooleanToInteger;

/// `lhs and rhs`.
pub fn both(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	select(nodes, lhs, rhs, lhs)
}

/// `lhs or rhs`.
pub fn either(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	select(nodes, lhs, lhs, rhs)
}

fn select(nodes: &mut Vec<Node>, condition: Link, on_true: Link, on_false: Link) -> Link {
	let selector = BooleanToInteger::add_into(nodes, condition);
	let matcher = Match::add_if_into(
		nodes,
		vec![on_false, on_true],
		selector,
		|_nodes, arguments| vec![Link(arguments, 0)],
		|_nodes, arguments| vec![Link(arguments, 1)],
	);

	Link(matcher, 0)
}
