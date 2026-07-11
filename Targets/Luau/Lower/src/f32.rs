//! Lowerings for 32-bit floating-point operations on their bit patterns.

use ir_graph::{
	Link, Node,
	operation::number::{BinaryOperator, CompareOperator, UnaryOperator},
	region::Match,
};
use luau_foreign::{
	Bit32And, Bit32Or, Bit32Xor, BooleanToInteger, FromBitsF32, IntoBitsF32, LuauAdd, LuauDivide,
	LuauEqual, LuauLessThan, LuauLessThanEqual, LuauMultiply, LuauNegate, LuauNotEqual,
	LuauSubtract, MathAbs, MathCeil, MathFloor, MathMax, MathMin, MathModf, MathSqrt, VectorCreate,
	VectorX,
};

use crate::round;

const SIGN_BIT: u32 = 0x8000_0000;
const MAGNITUDE_MASK: u32 = 0x7FFF_FFFF;
const SIGN_THRESHOLD: f64 = 2_147_483_648.0;

/// Lowers a 32-bit floating-point unary operation.
pub fn unary(nodes: &mut Vec<Node>, source: Link, operator: UnaryOperator) -> Link {
	match operator {
		UnaryOperator::Absolute => Bit32And::add_fast_into(nodes, source, MAGNITUDE_MASK),
		UnaryOperator::Negate => Bit32Xor::add_fast_into(nodes, source, SIGN_BIT),
		UnaryOperator::SquareRoot => native(nodes, source, MathSqrt::add_into),
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
		|nodes, arguments| vec![LuauNegate::add_into(nodes, Link(arguments, 0))],
		|_nodes, arguments| vec![Link(arguments, 0)],
	);

	IntoBitsF32::add_into(nodes, Link(matcher, 0))
}

// The f32 sign lives in bit 31, so the unsigned bit pattern is positive exactly
// while it stays below the sign-bit threshold.
fn is_positive(nodes: &mut Vec<Node>, source: Link) -> Link {
	let threshold = Node::add_f64_into(nodes, SIGN_THRESHOLD);
	let positive = LuauLessThan::add_into(nodes, source, threshold);

	BooleanToInteger::add_into(nodes, positive)
}

/// Lowers a 32-bit floating-point binary operation.
pub fn binary(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, operator: BinaryOperator) -> Link {
	match operator {
		BinaryOperator::Add => arithmetic(nodes, lhs, rhs, LuauAdd::add_into),
		BinaryOperator::Subtract => arithmetic(nodes, lhs, rhs, LuauSubtract::add_into),
		BinaryOperator::Multiply => arithmetic(nodes, lhs, rhs, LuauMultiply::add_into),
		BinaryOperator::Divide => divide(nodes, lhs, rhs),
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
	sign: Link,
	combine: fn(&mut Vec<Node>, Link, Link) -> Link,
) -> Link {
	let condition = is_positive(nodes, sign);
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
	ordered(nodes, lhs, rhs, lhs, MathMin::add_into)
}

fn maximum(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	ordered(nodes, lhs, rhs, rhs, MathMax::add_into)
}

// An f32 op of two f32 values has an exact f64 intermediate (the 24-bit mantissas
// fit), so decoding to native doubles, applying the operator, and re-encoding
// rounds to f32 exactly once. Division is excluded: its f64 result is inexact, so
// re-encoding would round twice.
fn arithmetic(
	nodes: &mut Vec<Node>,
	lhs: Link,
	rhs: Link,
	combine: fn(&mut Vec<Node>, Link, Link) -> Link,
) -> Link {
	let lhs = FromBitsF32::add_into(nodes, lhs);
	let rhs = FromBitsF32::add_into(nodes, rhs);
	let result = combine(nodes, lhs, rhs);

	IntoBitsF32::add_into(nodes, result)
}

// Division's f64 quotient is inexact, so it cannot round once through the operator
// like the others. Dividing single-lane vectors performs the division at f32
// precision directly, and the lane-zero projection recovers the rounded scalar.
fn divide(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let lhs = FromBitsF32::add_into(nodes, lhs);
	let rhs = FromBitsF32::add_into(nodes, rhs);
	let lhs = VectorCreate::add_into(nodes, lhs);
	let rhs = VectorCreate::add_into(nodes, rhs);
	let quotient = LuauDivide::add_into(nodes, lhs, rhs);
	let scalar = VectorX::add_into(nodes, quotient);

	IntoBitsF32::add_into(nodes, scalar)
}

fn copy_sign(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let magnitude = Bit32And::add_fast_into(nodes, lhs, MAGNITUDE_MASK);
	let sign = Bit32And::add_fast_into(nodes, rhs, SIGN_BIT);

	Bit32Or::add_into(nodes, magnitude, sign)
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

	LuauEqual::add_into(nodes, lhs, rhs)
}

fn not_equal(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let (lhs, rhs) = split_both(nodes, lhs, rhs);

	LuauNotEqual::add_into(nodes, lhs, rhs)
}

fn less_than(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let (lhs, rhs) = split_both(nodes, lhs, rhs);

	LuauLessThan::add_into(nodes, lhs, rhs)
}

fn less_than_equal(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let (lhs, rhs) = split_both(nodes, lhs, rhs);

	LuauLessThanEqual::add_into(nodes, lhs, rhs)
}
