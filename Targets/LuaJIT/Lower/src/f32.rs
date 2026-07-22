//! Lowerings for 32-bit floating-point operations on their bit patterns.

use ir_graph::{
	Link, Node,
	operation::number::{BinaryOperator, CompareOperator, UnaryOperator},
	region::Match,
};
use luajit_foreign::{
	BitAnd, BitOr, BitXor, BooleanToInteger, FromBitsF32, IntoBitsF32, LuaJITEqual, LuaJITLessThan,
	LuaJITLessThanEqual, LuaJITNegate, LuaJITNotEqual, MathAbs, MathCeil, MathFloor, MathMax,
	MathMin, MathModf, NativeAddF32, NativeDivideF32, NativeMultiplyF32, NativeSquareRootF32,
	NativeSubtractF32,
};

use crate::round;

const SIGN_BIT: u32 = 0x8000_0000;
const MAGNITUDE_MASK: u32 = 0x7FFF_FFFF;

/// Lowers a 32-bit floating-point unary operation.
pub fn unary(nodes: &mut Vec<Node>, source: Link, operator: UnaryOperator) -> Link {
	match operator {
		UnaryOperator::Absolute => BitAnd::add_fast_into(nodes, source, MAGNITUDE_MASK),
		UnaryOperator::Negate => BitXor::add_fast_into(nodes, source, SIGN_BIT),
		UnaryOperator::SquareRoot => NativeSquareRootF32::add_into(nodes, source),
		UnaryOperator::RoundUp => native(nodes, source, MathCeil::add_into),
		UnaryOperator::RoundDown => native(nodes, source, MathFloor::add_into),
		UnaryOperator::Truncate => native(nodes, source, MathModf::add_into),
		UnaryOperator::Nearest => nearest(nodes, source),
	}
}

// The math builtin runs on the native double; the single f32 rounding falls out of
// the narrower re-encode.
fn native(nodes: &mut Vec<Node>, source: Link, math: fn(&mut Vec<Node>, Link) -> Link) -> Link {
	let value = FromBitsF32::add_into(nodes, source);
	let result = math(nodes, value);

	IntoBitsF32::add_into(nodes, result)
}

fn nearest(nodes: &mut Vec<Node>, source: Link) -> Link {
	let native = FromBitsF32::add_into(nodes, source);
	let magnitude = MathAbs::add_into(nodes, native);
	let rounded = round::round_half_even(nodes, magnitude);
	let condition = is_positive(nodes, source);
	let matcher = Match::add_if_into(
		nodes,
		vec![rounded],
		condition,
		|nodes, arguments| vec![LuaJITNegate::add_into(nodes, Link(arguments, 0))],
		|_nodes, arguments| vec![Link(arguments, 0)],
	);

	IntoBitsF32::add_into(nodes, Link(matcher, 0))
}

// LuaJIT stores f32 values as signed bit patterns, so sign-clear patterns are
// nonnegative and sign-set patterns are negative.
fn is_positive(nodes: &mut Vec<Node>, source: Link) -> Link {
	let zero = Node::add_i32_into(nodes, 0);
	let positive = LuaJITLessThanEqual::add_into(nodes, zero, source);

	BooleanToInteger::add_into(nodes, positive)
}

/// Lowers a 32-bit floating-point binary operation.
pub fn binary(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, operator: BinaryOperator) -> Link {
	match operator {
		BinaryOperator::Add => NativeAddF32::add_into(nodes, lhs, rhs),
		BinaryOperator::Subtract => NativeSubtractF32::add_into(nodes, lhs, rhs),
		BinaryOperator::Multiply => NativeMultiplyF32::add_into(nodes, lhs, rhs),
		BinaryOperator::Divide => NativeDivideF32::add_into(nodes, lhs, rhs),
		BinaryOperator::Minimum => minimum(nodes, lhs, rhs),
		BinaryOperator::Maximum => maximum(nodes, lhs, rhs),
		BinaryOperator::CopySign => copy_sign(nodes, lhs, rhs),
	}
}

// Mirrors the runtime: swap on the sign-bearing operand's bits so the decoded
// pair feeds `math.min`/`math.max` in the order that breaks a zero/NaN tie the way
// the spec wants, then re-encode the chosen operand.
fn ordered(
	nodes: &mut Vec<Node>,
	lhs: Link,
	rhs: Link,
	condition: Link,
	combine: fn(&mut Vec<Node>, Link, Link) -> Link,
) -> Link {
	let matcher = Match::add_if_into(
		nodes,
		vec![lhs, rhs],
		condition,
		|_nodes, arguments| vec![Link(arguments, 0), Link(arguments, 1)],
		|_nodes, arguments| vec![Link(arguments, 1), Link(arguments, 0)],
	);
	let first = FromBitsF32::add_into(nodes, Link(matcher, 0));
	let second = FromBitsF32::add_into(nodes, Link(matcher, 1));
	let result = combine(nodes, first, second);

	IntoBitsF32::add_into(nodes, result)
}

fn minimum(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let condition = is_positive(nodes, rhs);

	ordered(nodes, lhs, rhs, condition, MathMin::add_into)
}

fn maximum(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let zero = Node::add_i32_into(nodes, 0);
	let negative = LuaJITLessThan::add_into(nodes, rhs, zero);
	let condition = BooleanToInteger::add_into(nodes, negative);

	ordered(nodes, lhs, rhs, condition, MathMax::add_into)
}

fn copy_sign(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let magnitude = BitAnd::add_fast_into(nodes, lhs, MAGNITUDE_MASK);
	let sign = BitAnd::add_fast_into(nodes, rhs, SIGN_BIT);

	BitOr::add_into(nodes, magnitude, sign)
}

/// Lowers a 32-bit floating-point comparison.
pub fn compare(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, operator: CompareOperator) -> Link {
	let boolean = match operator {
		CompareOperator::Equal => equal(nodes, lhs, rhs),
		CompareOperator::NotEqual => not_equal(nodes, lhs, rhs),
		CompareOperator::LessThan => less_than(nodes, lhs, rhs),
		CompareOperator::LessThanEqual => less_than_equal(nodes, lhs, rhs),
	};

	BooleanToInteger::add_into(nodes, boolean)
}

fn split_both(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> (Link, Link) {
	let lhs = FromBitsF32::add_into(nodes, lhs);
	let rhs = FromBitsF32::add_into(nodes, rhs);

	(lhs, rhs)
}

fn equal(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let (lhs, rhs) = split_both(nodes, lhs, rhs);

	LuaJITEqual::add_into(nodes, lhs, rhs)
}

fn not_equal(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let (lhs, rhs) = split_both(nodes, lhs, rhs);

	LuaJITNotEqual::add_into(nodes, lhs, rhs)
}

fn less_than(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let (lhs, rhs) = split_both(nodes, lhs, rhs);

	LuaJITLessThan::add_into(nodes, lhs, rhs)
}

fn less_than_equal(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let (lhs, rhs) = split_both(nodes, lhs, rhs);

	LuaJITLessThanEqual::add_into(nodes, lhs, rhs)
}
