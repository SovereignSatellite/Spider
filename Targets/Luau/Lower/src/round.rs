//! Shared round-half-to-even helpers for floating-point nearest lowering.

use ir_graph::{Link, Node, region::Match};
use luau_foreign::{
	BooleanToInteger, LuauAdd, LuauEqual, LuauLessThan, LuauModulo, LuauSubtract, MathModf,
};

use crate::boolean::{both, either};

const HALF: f64 = 0.5;

/// Rounds the magnitude of a native double to the nearest integer, breaking ties toward even.
pub fn round_half_even(nodes: &mut Vec<Node>, magnitude: Link) -> Link {
	let rounded = MathModf::add_into(nodes, magnitude);
	let remainder = LuauSubtract::add_into(nodes, magnitude, rounded);
	let condition = should_round_up(nodes, remainder, rounded);
	let matcher = Match::add_if_into(
		nodes,
		vec![rounded],
		condition,
		|_nodes, arguments| vec![Link(arguments, 0)],
		|nodes, arguments| vec![increment(nodes, Link(arguments, 0))],
	);

	Link(matcher, 0)
}

fn should_round_up(nodes: &mut Vec<Node>, remainder: Link, rounded: Link) -> Link {
	let boundary = Node::add_f64_into(nodes, HALF);
	let over = LuauLessThan::add_into(nodes, boundary, remainder);
	let tie = is_tie(nodes, remainder, rounded);
	let bump = either(nodes, over, tie);

	BooleanToInteger::add_into(nodes, bump)
}

fn is_tie(nodes: &mut Vec<Node>, remainder: Link, rounded: Link) -> Link {
	let boundary = Node::add_f64_into(nodes, HALF);
	let half = LuauEqual::add_into(nodes, remainder, boundary);
	let odd = is_odd(nodes, rounded);

	both(nodes, half, odd)
}

fn is_odd(nodes: &mut Vec<Node>, rounded: Link) -> Link {
	let divisor = Node::add_i32_into(nodes, 2);
	let parity = LuauModulo::add_into(nodes, rounded, divisor);
	let odd = Node::add_i32_into(nodes, 1);

	LuauEqual::add_into(nodes, parity, odd)
}

fn increment(nodes: &mut Vec<Node>, source: Link) -> Link {
	let one = Node::add_i32_into(nodes, 1);

	LuauAdd::add_into(nodes, source, one)
}
