//! Lowerings for 64-bit floating-point operations on their bit patterns.

use ir_graph::{
	Link, Node,
	operation::number::{BinaryOperator, CompareOperator, UnaryOperator},
	region::Match,
};
use luajit_foreign::{
	BitAnd, BitOr, BitXor, BooleanToInteger, FromBitsF64, IntoBitsF64, LuaJITAdd, LuaJITDivide,
	LuaJITEqual, LuaJITLessThan, LuaJITLessThanEqual, LuaJITMultiply, LuaJITNegate, LuaJITNotEqual,
	LuaJITSubtract, MathAbs, MathCeil, MathFloor, MathMax, MathMin, MathModf, MathSqrt,
};

use crate::round;

const SIGN_BIT: i64 = i64::MIN;
const MAGNITUDE_MASK: i64 = i64::MAX;

/// Lowers a 64-bit floating-point unary operation.
pub fn unary(nodes: &mut Vec<Node>, source: Link, operator: UnaryOperator) -> Link {
	match operator {
		UnaryOperator::Absolute => absolute(nodes, source),
		UnaryOperator::Negate => negate(nodes, source),
		UnaryOperator::SquareRoot => native(nodes, source, MathSqrt::add_into),
		UnaryOperator::RoundUp => native(nodes, source, MathCeil::add_into),
		UnaryOperator::RoundDown => native(nodes, source, MathFloor::add_into),
		UnaryOperator::Truncate => native(nodes, source, MathModf::add_into),
		UnaryOperator::Nearest => nearest(nodes, source),
	}
}

fn absolute(nodes: &mut Vec<Node>, source: Link) -> Link {
	let mask = Node::add_i64_into(nodes, MAGNITUDE_MASK);

	BitAnd::add_into(nodes, source, mask)
}

fn negate(nodes: &mut Vec<Node>, source: Link) -> Link {
	let sign = Node::add_i64_into(nodes, SIGN_BIT);

	BitXor::add_into(nodes, source, sign)
}

fn native(nodes: &mut Vec<Node>, source: Link, math: fn(&mut Vec<Node>, Link) -> Link) -> Link {
	let value = FromBitsF64::add_into(nodes, source);
	let result = math(nodes, value);

	IntoBitsF64::add_into(nodes, result)
}

/// Lowers a 64-bit floating-point binary operation.
pub fn binary(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, operator: BinaryOperator) -> Link {
	match operator {
		BinaryOperator::Add => arithmetic(nodes, lhs, rhs, LuaJITAdd::add_into),
		BinaryOperator::Subtract => arithmetic(nodes, lhs, rhs, LuaJITSubtract::add_into),
		BinaryOperator::Multiply => arithmetic(nodes, lhs, rhs, LuaJITMultiply::add_into),
		BinaryOperator::Divide => arithmetic(nodes, lhs, rhs, LuaJITDivide::add_into),
		BinaryOperator::Minimum => minimum(nodes, lhs, rhs),
		BinaryOperator::Maximum => maximum(nodes, lhs, rhs),
		BinaryOperator::CopySign => copy_sign(nodes, lhs, rhs),
	}
}

fn arithmetic(
	nodes: &mut Vec<Node>,
	lhs: Link,
	rhs: Link,
	combine: fn(&mut Vec<Node>, Link, Link) -> Link,
) -> Link {
	let lhs = FromBitsF64::add_into(nodes, lhs);
	let rhs = FromBitsF64::add_into(nodes, rhs);
	let result = combine(nodes, lhs, rhs);

	IntoBitsF64::add_into(nodes, result)
}

/// Lowers a 64-bit floating-point comparison.
pub fn compare(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, operator: CompareOperator) -> Link {
	let boolean = match operator {
		CompareOperator::Equal => compare_native(nodes, lhs, rhs, LuaJITEqual::add_into),
		CompareOperator::NotEqual => compare_native(nodes, lhs, rhs, LuaJITNotEqual::add_into),
		CompareOperator::LessThan => compare_native(nodes, lhs, rhs, LuaJITLessThan::add_into),
		CompareOperator::LessThanEqual => {
			compare_native(nodes, lhs, rhs, LuaJITLessThanEqual::add_into)
		}
	};

	BooleanToInteger::add_into(nodes, boolean)
}

fn compare_native(
	nodes: &mut Vec<Node>,
	lhs: Link,
	rhs: Link,
	compare: fn(&mut Vec<Node>, Link, Link) -> Link,
) -> Link {
	let lhs = FromBitsF64::add_into(nodes, lhs);
	let rhs = FromBitsF64::add_into(nodes, rhs);

	compare(nodes, lhs, rhs)
}

fn is_positive(nodes: &mut Vec<Node>, source: Link) -> Link {
	let zero = Node::add_i64_into(nodes, 0);
	let positive = LuaJITLessThanEqual::add_into(nodes, zero, source);

	BooleanToInteger::add_into(nodes, positive)
}

fn nearest(nodes: &mut Vec<Node>, source: Link) -> Link {
	let native = FromBitsF64::add_into(nodes, source);
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

	IntoBitsF64::add_into(nodes, Link(matcher, 0))
}

// Mirrors the runtime: the sign-bearing operand determines whether to swap the
// decoded pair so zero and NaN ties preserve the WebAssembly-selected operand.
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
		|nodes, arguments| vec![combine(nodes, Link(arguments, 0), Link(arguments, 1))],
		|nodes, arguments| vec![combine(nodes, Link(arguments, 1), Link(arguments, 0))],
	);

	Link(matcher, 0)
}

fn minimum(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let condition = is_positive(nodes, rhs);
	let lhs = FromBitsF64::add_into(nodes, lhs);
	let rhs = FromBitsF64::add_into(nodes, rhs);
	let result = ordered(nodes, lhs, rhs, condition, MathMin::add_into);

	IntoBitsF64::add_into(nodes, result)
}

fn maximum(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let zero = Node::add_i64_into(nodes, 0);
	let negative = LuaJITLessThan::add_into(nodes, rhs, zero);
	let condition = BooleanToInteger::add_into(nodes, negative);
	let lhs = FromBitsF64::add_into(nodes, lhs);
	let rhs = FromBitsF64::add_into(nodes, rhs);
	let result = ordered(nodes, lhs, rhs, condition, MathMax::add_into);

	IntoBitsF64::add_into(nodes, result)
}

fn copy_sign(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let magnitude_mask = Node::add_i64_into(nodes, MAGNITUDE_MASK);
	let magnitude = BitAnd::add_into(nodes, lhs, magnitude_mask);
	let sign_bit = Node::add_i64_into(nodes, SIGN_BIT);
	let sign = BitAnd::add_into(nodes, rhs, sign_bit);

	BitOr::add_into(nodes, magnitude, sign)
}
