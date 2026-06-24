//! Lowerings for 64-bit floating-point operations.

use ir_graph::{
	Link, Node,
	operation::number::{BinaryOperator, CompareOperator, UnaryOperator},
	region::Match,
};
use luau_foreign::{
	BooleanToInteger, IsPositive, LuauAdd, LuauDivide, LuauEqual, LuauLessThan, LuauLessThanEqual,
	LuauMultiply, LuauNegate, LuauNotEqual, LuauSubtract, MathAbs, MathCeil, MathFloor, MathMax,
	MathMin, MathModf, MathSqrt,
};

use crate::round;

/// Lowers a 64-bit floating-point unary operation.
pub fn unary(nodes: &mut Vec<Node>, source: Link, operator: UnaryOperator) -> Link {
	match operator {
		UnaryOperator::Absolute => MathAbs::add_into(nodes, source),
		UnaryOperator::Negate => LuauNegate::add_into(nodes, source),
		UnaryOperator::SquareRoot => MathSqrt::add_into(nodes, source),
		UnaryOperator::RoundUp => MathCeil::add_into(nodes, source),
		UnaryOperator::RoundDown => MathFloor::add_into(nodes, source),
		UnaryOperator::Truncate => MathModf::add_into(nodes, source),
		UnaryOperator::Nearest => nearest(nodes, source),
	}
}

/// Lowers a 64-bit floating-point binary operation.
pub fn binary(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, operator: BinaryOperator) -> Link {
	match operator {
		BinaryOperator::Add => LuauAdd::add_into(nodes, lhs, rhs),
		BinaryOperator::Subtract => LuauSubtract::add_into(nodes, lhs, rhs),
		BinaryOperator::Multiply => LuauMultiply::add_into(nodes, lhs, rhs),
		BinaryOperator::Divide => LuauDivide::add_into(nodes, lhs, rhs),
		BinaryOperator::Minimum => minimum(nodes, lhs, rhs),
		BinaryOperator::Maximum => maximum(nodes, lhs, rhs),
		BinaryOperator::CopySign => copy_sign(nodes, lhs, rhs),
	}
}

/// Lowers a 64-bit floating-point comparison.
pub fn compare(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, operator: CompareOperator) -> Link {
	let boolean = match operator {
		CompareOperator::Equal => LuauEqual::add_into(nodes, lhs, rhs),
		CompareOperator::NotEqual => LuauNotEqual::add_into(nodes, lhs, rhs),
		CompareOperator::LessThan => LuauLessThan::add_into(nodes, lhs, rhs),
		CompareOperator::GreaterThan => LuauLessThan::add_into(nodes, rhs, lhs),
		CompareOperator::LessThanEqual => LuauLessThanEqual::add_into(nodes, lhs, rhs),
		CompareOperator::GreaterThanEqual => LuauLessThanEqual::add_into(nodes, rhs, lhs),
	};

	BooleanToInteger::add_into(nodes, boolean)
}

fn is_positive(nodes: &mut Vec<Node>, source: Link) -> Link {
	let positive = IsPositive::add_into(nodes, source);

	BooleanToInteger::add_into(nodes, positive)
}

fn nearest(nodes: &mut Vec<Node>, source: Link) -> Link {
	let magnitude = MathAbs::add_into(nodes, source);
	let rounded = round::round_half_even(nodes, magnitude);
	let condition = is_positive(nodes, source);
	let matcher = Match::add_if_into(
		nodes,
		vec![rounded],
		condition,
		|nodes, arguments| vec![LuauNegate::add_into(nodes, Link(arguments, 0))],
		|_nodes, arguments| vec![Link(arguments, 0)],
	);

	Link(matcher, 0)
}

// Mirrors the runtime: the sign-bearing operand (per `is_positive`) is ordered
// second so `math.min`/`math.max` break a zero/NaN tie the way the spec wants.
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
		|nodes, arguments| vec![combine(nodes, Link(arguments, 0), Link(arguments, 1))],
		|nodes, arguments| vec![combine(nodes, Link(arguments, 1), Link(arguments, 0))],
	);

	Link(matcher, 0)
}

fn minimum(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	ordered(nodes, lhs, rhs, lhs, MathMin::add_into)
}

fn maximum(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	ordered(nodes, lhs, rhs, rhs, MathMax::add_into)
}

fn copy_sign(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let magnitude = MathAbs::add_into(nodes, lhs);
	let condition = is_positive(nodes, rhs);
	let matcher = Match::add_if_into(
		nodes,
		vec![magnitude],
		condition,
		|nodes, arguments| vec![LuauNegate::add_into(nodes, Link(arguments, 0))],
		|_nodes, arguments| vec![Link(arguments, 0)],
	);

	Link(matcher, 0)
}
