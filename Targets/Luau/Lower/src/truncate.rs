//! Lowerings for float-to-integer truncation, both trapping and saturating.

use ir_graph::{
	Link, Node,
	operation::{NumberTruncateToInteger, integer, number},
	region::Match,
};
use luau_foreign::{
	Bit32Or, BooleanToInteger, FromBitsF32, LuauLessThan, LuauLessThanEqual, LuauNotEqual,
	MathFloor, MathModf,
};

use crate::boolean::either;

const SIGNED_LIMIT: f64 = 2_147_483_648.0;
const SIGNED_FLOOR: f64 = -2_147_483_648.0;
const UNSIGNED_LIMIT: f64 = 4_294_967_296.0;
const UNSIGNED_FLOOR: f64 = 0.0;
const SIGNED_MAXIMUM: f64 = 2_147_483_647.0;
const SIGNED_MINIMUM: f64 = 2_147_483_648.0;
const UNSIGNED_MAXIMUM: f64 = 4_294_967_295.0;

/// Lowers a float-to-integer truncation that targets a 32-bit integer.
pub fn to_integer(nodes: &mut Vec<Node>, operation: &NumberTruncateToInteger) -> Option<Link> {
	let &NumberTruncateToInteger {
		source,
		is_signed,
		is_saturating,
		to,
		from,
	} = operation;

	match to {
		integer::Type::I32 => Some(to_word(nodes, source, is_signed, is_saturating, from)),
		integer::Type::I64 => None,
	}
}

fn to_word(
	nodes: &mut Vec<Node>,
	source: Link,
	is_signed: bool,
	is_saturating: bool,
	from: number::Type,
) -> Link {
	let decoded = decode(nodes, source, from);

	if is_saturating {
		saturate(nodes, decoded, is_signed)
	} else {
		trap(nodes, decoded, is_signed)
	}
}

fn decode(nodes: &mut Vec<Node>, source: Link, from: number::Type) -> Link {
	match from {
		number::Type::F32 => FromBitsF32::add_into(nodes, source),
		number::Type::F64 => source,
	}
}

fn trap(nodes: &mut Vec<Node>, decoded: Link, is_signed: bool) -> Link {
	let truncated = MathModf::add_into(nodes, decoded);
	let condition = out_of_range(nodes, truncated, is_signed);
	let matcher = Match::add_if_into(
		nodes,
		vec![truncated],
		condition,
		|nodes, arguments| vec![wrap(nodes, Link(arguments, 0))],
		|nodes, _arguments| vec![Node::add_trap_into(nodes)],
	);

	Link(matcher, 0)
}

fn out_of_range(nodes: &mut Vec<Node>, value: Link, is_signed: bool) -> Link {
	let above = above_limit(nodes, value, is_signed);
	let below = below_floor(nodes, value, is_signed);
	let outside = either(nodes, above, below);
	let not_a_number = LuauNotEqual::add_into(nodes, value, value);
	let trapping = either(nodes, outside, not_a_number);

	BooleanToInteger::add_into(nodes, trapping)
}

fn above_limit(nodes: &mut Vec<Node>, value: Link, is_signed: bool) -> Link {
	let limit = if is_signed {
		SIGNED_LIMIT
	} else {
		UNSIGNED_LIMIT
	};
	let boundary = Node::add_f64_into(nodes, limit);

	LuauLessThanEqual::add_into(nodes, boundary, value)
}

fn below_floor(nodes: &mut Vec<Node>, value: Link, is_signed: bool) -> Link {
	let floor = if is_signed {
		SIGNED_FLOOR
	} else {
		UNSIGNED_FLOOR
	};
	let boundary = Node::add_f64_into(nodes, floor);

	LuauLessThan::add_into(nodes, value, boundary)
}

fn saturate(nodes: &mut Vec<Node>, decoded: Link, is_signed: bool) -> Link {
	if is_signed {
		saturate_signed(nodes, decoded)
	} else {
		saturate_unsigned(nodes, decoded)
	}
}

fn saturate_signed(nodes: &mut Vec<Node>, source: Link) -> Link {
	let above = at_or_above(nodes, source, SIGNED_LIMIT);
	let matcher = Match::add_if_into(
		nodes,
		vec![source],
		above,
		|nodes, arguments| vec![saturate_signed_floor(nodes, Link(arguments, 0))],
		|nodes, _arguments| vec![constant(nodes, SIGNED_MAXIMUM)],
	);

	Link(matcher, 0)
}

fn saturate_signed_floor(nodes: &mut Vec<Node>, source: Link) -> Link {
	let below = at_or_below(nodes, source, SIGNED_FLOOR);
	let matcher = Match::add_if_into(
		nodes,
		vec![source],
		below,
		|nodes, arguments| vec![saturate_finite(nodes, Link(arguments, 0))],
		|nodes, _arguments| vec![constant(nodes, SIGNED_MINIMUM)],
	);

	Link(matcher, 0)
}

fn saturate_finite(nodes: &mut Vec<Node>, source: Link) -> Link {
	let not_a_number = is_not_a_number(nodes, source);
	let matcher = Match::add_if_into(
		nodes,
		vec![source],
		not_a_number,
		|nodes, arguments| vec![truncate_and_wrap(nodes, Link(arguments, 0))],
		|nodes, _arguments| vec![zero(nodes)],
	);

	Link(matcher, 0)
}

fn saturate_unsigned(nodes: &mut Vec<Node>, source: Link) -> Link {
	let above = at_or_above(nodes, source, UNSIGNED_LIMIT);
	let matcher = Match::add_if_into(
		nodes,
		vec![source],
		above,
		|nodes, arguments| vec![saturate_unsigned_floor(nodes, Link(arguments, 0))],
		|nodes, _arguments| vec![constant(nodes, UNSIGNED_MAXIMUM)],
	);

	Link(matcher, 0)
}

fn saturate_unsigned_floor(nodes: &mut Vec<Node>, source: Link) -> Link {
	let condition = below_zero_or_nan(nodes, source);
	let matcher = Match::add_if_into(
		nodes,
		vec![source],
		condition,
		|nodes, arguments| vec![floor_and_wrap(nodes, Link(arguments, 0))],
		|nodes, _arguments| vec![zero(nodes)],
	);

	Link(matcher, 0)
}

fn at_or_above(nodes: &mut Vec<Node>, source: Link, limit: f64) -> Link {
	let boundary = Node::add_f64_into(nodes, limit);
	let above = LuauLessThanEqual::add_into(nodes, boundary, source);

	BooleanToInteger::add_into(nodes, above)
}

fn at_or_below(nodes: &mut Vec<Node>, source: Link, limit: f64) -> Link {
	let boundary = Node::add_f64_into(nodes, limit);
	let below = LuauLessThanEqual::add_into(nodes, source, boundary);

	BooleanToInteger::add_into(nodes, below)
}

fn below_zero_or_nan(nodes: &mut Vec<Node>, source: Link) -> Link {
	let boundary = Node::add_f64_into(nodes, UNSIGNED_FLOOR);
	let below = LuauLessThanEqual::add_into(nodes, source, boundary);
	let not_a_number = LuauNotEqual::add_into(nodes, source, source);
	let below_or_nan = either(nodes, below, not_a_number);

	BooleanToInteger::add_into(nodes, below_or_nan)
}

fn is_not_a_number(nodes: &mut Vec<Node>, source: Link) -> Link {
	let not_a_number = LuauNotEqual::add_into(nodes, source, source);

	BooleanToInteger::add_into(nodes, not_a_number)
}

fn truncate_and_wrap(nodes: &mut Vec<Node>, source: Link) -> Link {
	let truncated = MathModf::add_into(nodes, source);

	wrap(nodes, truncated)
}

fn floor_and_wrap(nodes: &mut Vec<Node>, source: Link) -> Link {
	let floored = MathFloor::add_into(nodes, source);

	wrap(nodes, floored)
}

fn constant(nodes: &mut Vec<Node>, value: f64) -> Link {
	Node::add_f64_into(nodes, value)
}

fn zero(nodes: &mut Vec<Node>) -> Link {
	Node::add_i32_into(nodes, 0)
}

fn wrap(nodes: &mut Vec<Node>, value: Link) -> Link {
	Bit32Or::add_fast_into(nodes, value, 0)
}
