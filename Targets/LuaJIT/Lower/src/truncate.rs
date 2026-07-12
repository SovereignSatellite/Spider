//! Lowerings for float-to-integer truncation, both trapping and saturating.

use ir_graph::{
	Link, Node,
	operation::{NumberTruncateToInteger, integer, number},
	region::Match,
};
use luajit_foreign::{
	BooleanToInteger, CastI64, CastU64, ForceI32, FromBitsF32, FromBitsF64, LuaJITLessThan,
	LuaJITLessThanEqual, LuaJITNotEqual, MathFloor, MathModf,
};

use crate::boolean::either;

const SIGNED_WORD_LIMIT: f64 = 2_147_483_648.0;
const SIGNED_WORD_FLOOR: f64 = -2_147_483_648.0;
const UNSIGNED_WORD_LIMIT: f64 = 4_294_967_296.0;
const SIGNED_LONG_LIMIT: f64 = f64::from_bits(0x43E0_0000_0000_0000);
const SIGNED_LONG_FLOOR: f64 = f64::from_bits(0xC3E0_0000_0000_0000);
const UNSIGNED_LONG_LIMIT: f64 = f64::from_bits(0x43F0_0000_0000_0000);

#[derive(Clone, Copy)]
struct Target {
	kind: integer::Type,
	is_signed: bool,
}

impl Target {
	const fn limit(self) -> f64 {
		match (self.kind, self.is_signed) {
			(integer::Type::I32, true) => SIGNED_WORD_LIMIT,
			(integer::Type::I32, false) => UNSIGNED_WORD_LIMIT,
			(integer::Type::I64, true) => SIGNED_LONG_LIMIT,
			(integer::Type::I64, false) => UNSIGNED_LONG_LIMIT,
		}
	}

	const fn floor(self) -> f64 {
		match (self.kind, self.is_signed) {
			(integer::Type::I32, true) => SIGNED_WORD_FLOOR,
			(integer::Type::I64, true) => SIGNED_LONG_FLOOR,
			(integer::Type::I32 | integer::Type::I64, false) => 0.0,
		}
	}
}

/// Lowers a float-to-integer truncation.
pub fn to_integer(nodes: &mut Vec<Node>, operation: &NumberTruncateToInteger) -> Link {
	let &NumberTruncateToInteger {
		source,
		is_signed,
		is_saturating,
		to,
		from,
	} = operation;
	let decoded = decode(nodes, source, from);
	let target = Target {
		kind: to,
		is_signed,
	};

	if is_saturating {
		saturate(nodes, decoded, target)
	} else {
		trap(nodes, decoded, target)
	}
}

fn decode(nodes: &mut Vec<Node>, source: Link, from: number::Type) -> Link {
	match from {
		number::Type::F32 => FromBitsF32::add_into(nodes, source),
		number::Type::F64 => FromBitsF64::add_into(nodes, source),
	}
}

fn trap(nodes: &mut Vec<Node>, decoded: Link, target: Target) -> Link {
	let truncated = MathModf::add_into(nodes, decoded);
	let condition = out_of_range(nodes, truncated, target);
	let matcher = Match::add_if_into(
		nodes,
		vec![truncated],
		condition,
		move |nodes, arguments| vec![wrap(nodes, Link(arguments, 0), target)],
		|nodes, _arguments| vec![Node::add_trap_into(nodes)],
	);

	Link(matcher, 0)
}

fn out_of_range(nodes: &mut Vec<Node>, value: Link, target: Target) -> Link {
	let above = above_limit(nodes, value, target.limit());
	let below = below_floor(nodes, value, target.floor());
	let outside = either(nodes, above, below);
	let not_a_number = LuaJITNotEqual::add_into(nodes, value, value);
	let trapping = either(nodes, outside, not_a_number);

	BooleanToInteger::add_into(nodes, trapping)
}

fn above_limit(nodes: &mut Vec<Node>, value: Link, limit: f64) -> Link {
	let boundary = Node::add_f64_into(nodes, limit);
	let boundary = FromBitsF64::add_into(nodes, boundary);

	LuaJITLessThanEqual::add_into(nodes, boundary, value)
}

fn below_floor(nodes: &mut Vec<Node>, value: Link, floor: f64) -> Link {
	let boundary = Node::add_f64_into(nodes, floor);
	let boundary = FromBitsF64::add_into(nodes, boundary);

	LuaJITLessThan::add_into(nodes, value, boundary)
}

fn saturate(nodes: &mut Vec<Node>, source: Link, target: Target) -> Link {
	let above = at_or_above(nodes, source, target.limit());
	let matcher = Match::add_if_into(
		nodes,
		vec![source],
		above,
		move |nodes, arguments| saturate_floor(nodes, Link(arguments, 0), target),
		move |nodes, _arguments| vec![maximum(nodes, target)],
	);

	Link(matcher, 0)
}

fn saturate_floor(nodes: &mut Vec<Node>, source: Link, target: Target) -> Vec<Link> {
	let below = at_or_below(nodes, source, target.floor());
	let matcher = Match::add_if_into(
		nodes,
		vec![source],
		below,
		move |nodes, arguments| saturate_finite(nodes, Link(arguments, 0), target),
		move |nodes, _arguments| vec![minimum(nodes, target)],
	);

	vec![Link(matcher, 0)]
}

fn saturate_finite(nodes: &mut Vec<Node>, source: Link, target: Target) -> Vec<Link> {
	let not_a_number = is_not_a_number(nodes, source);
	let matcher = Match::add_if_into(
		nodes,
		vec![source],
		not_a_number,
		move |nodes, arguments| vec![truncate_and_wrap(nodes, Link(arguments, 0), target)],
		move |nodes, _arguments| vec![zero(nodes, target.kind)],
	);

	vec![Link(matcher, 0)]
}

fn at_or_above(nodes: &mut Vec<Node>, source: Link, limit: f64) -> Link {
	let boundary = Node::add_f64_into(nodes, limit);
	let boundary = FromBitsF64::add_into(nodes, boundary);
	let above = LuaJITLessThanEqual::add_into(nodes, boundary, source);

	BooleanToInteger::add_into(nodes, above)
}

fn at_or_below(nodes: &mut Vec<Node>, source: Link, limit: f64) -> Link {
	let boundary = Node::add_f64_into(nodes, limit);
	let boundary = FromBitsF64::add_into(nodes, boundary);
	let below = LuaJITLessThanEqual::add_into(nodes, source, boundary);

	BooleanToInteger::add_into(nodes, below)
}

fn is_not_a_number(nodes: &mut Vec<Node>, source: Link) -> Link {
	let not_a_number = LuaJITNotEqual::add_into(nodes, source, source);

	BooleanToInteger::add_into(nodes, not_a_number)
}

fn truncate_and_wrap(nodes: &mut Vec<Node>, source: Link, target: Target) -> Link {
	let truncated = if target.is_signed {
		MathModf::add_into(nodes, source)
	} else {
		MathFloor::add_into(nodes, source)
	};

	wrap(nodes, truncated, target)
}

fn maximum(nodes: &mut Vec<Node>, target: Target) -> Link {
	match (target.kind, target.is_signed) {
		(integer::Type::I32, true) => Node::add_i32_into(nodes, i32::MAX),
		(integer::Type::I32, false) => Node::add_i32_into(nodes, -1),
		(integer::Type::I64, true) => Node::add_i64_into(nodes, i64::MAX),
		(integer::Type::I64, false) => Node::add_i64_into(nodes, -1),
	}
}

fn minimum(nodes: &mut Vec<Node>, target: Target) -> Link {
	match (target.kind, target.is_signed) {
		(integer::Type::I32, true) => Node::add_i32_into(nodes, i32::MIN),
		(integer::Type::I64, true) => Node::add_i64_into(nodes, i64::MIN),
		(integer::Type::I32 | integer::Type::I64, false) => zero(nodes, target.kind),
	}
}

fn zero(nodes: &mut Vec<Node>, kind: integer::Type) -> Link {
	match kind {
		integer::Type::I32 => Node::add_i32_into(nodes, 0),
		integer::Type::I64 => Node::add_i64_into(nodes, 0),
	}
}

fn wrap(nodes: &mut Vec<Node>, value: Link, target: Target) -> Link {
	match (target.kind, target.is_signed) {
		(integer::Type::I32, _) => ForceI32::add_into(nodes, value),
		(integer::Type::I64, true) => CastI64::add_into(nodes, value),
		(integer::Type::I64, false) => {
			let value = CastU64::add_into(nodes, value);

			CastI64::add_into(nodes, value)
		}
	}
}
