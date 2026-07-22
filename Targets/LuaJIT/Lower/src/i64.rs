//! Lowerings for 64-bit integer operations on native FFI cdata.

use ir_graph::{
	Link, Node,
	operation::integer::{BinaryOperator, CompareOperator, UnaryOperator},
	region::Match,
};
use luajit_foreign::{
	BitAnd, BitArShift, BitLRotate, BitLShift, BitOr, BitRRotate, BitRShift, BitXor,
	BooleanToInteger, CastI64, CastU64, LuaJITAdd, LuaJITDivide, LuaJITEqual, LuaJITLessThan,
	LuaJITLessThanEqual, LuaJITModulo, LuaJITMultiply, LuaJITNotEqual, LuaJITSubtract,
};

use crate::boolean::{both, either};

const SHIFT_MASK: i64 = 0x3F;

/// Lowers a 64-bit integer unary operation.
pub fn unary(nodes: &mut Vec<Node>, source: Link, operator: UnaryOperator) -> Link {
	match operator {
		UnaryOperator::CountOnes => count_ones(nodes, source),
		UnaryOperator::LeadingZeros => leading_zeros(nodes, source),
		UnaryOperator::TrailingZeros => trailing_zeros(nodes, source),
	}
}

/// Lowers a 64-bit integer binary operation.
pub fn binary(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, operator: BinaryOperator) -> Link {
	match operator {
		BinaryOperator::Add => LuaJITAdd::add_into(nodes, lhs, rhs),
		BinaryOperator::Subtract => LuaJITSubtract::add_into(nodes, lhs, rhs),
		BinaryOperator::Multiply => LuaJITMultiply::add_into(nodes, lhs, rhs),
		BinaryOperator::Divide { is_signed: true } => divide_signed(nodes, lhs, rhs),
		BinaryOperator::Divide { is_signed: false } => divide_unsigned(nodes, lhs, rhs),
		BinaryOperator::Remainder { is_signed: true } => remainder_signed(nodes, lhs, rhs),
		BinaryOperator::Remainder { is_signed: false } => remainder_unsigned(nodes, lhs, rhs),
		BinaryOperator::And => BitAnd::add_into(nodes, lhs, rhs),
		BinaryOperator::Or => BitOr::add_into(nodes, lhs, rhs),
		BinaryOperator::ExclusiveOr => BitXor::add_into(nodes, lhs, rhs),
		BinaryOperator::ShiftLeft => shift_left(nodes, lhs, rhs),
		BinaryOperator::ShiftRight { is_signed: true } => shift_right_signed(nodes, lhs, rhs),
		BinaryOperator::ShiftRight { is_signed: false } => shift_right_unsigned(nodes, lhs, rhs),
		BinaryOperator::RotateLeft => BitLRotate::add_into(nodes, lhs, rhs),
		BinaryOperator::RotateRight => BitRRotate::add_into(nodes, lhs, rhs),
	}
}

/// Lowers a 64-bit integer comparison.
pub fn compare(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, operator: CompareOperator) -> Link {
	let boolean = match operator {
		CompareOperator::Equal => LuaJITEqual::add_into(nodes, lhs, rhs),
		CompareOperator::NotEqual => LuaJITNotEqual::add_into(nodes, lhs, rhs),
		CompareOperator::LessThan { is_signed } => less_than(nodes, lhs, rhs, is_signed),
		CompareOperator::LessThanEqual { is_signed } => less_than_equal(nodes, lhs, rhs, is_signed),
	};

	BooleanToInteger::add_into(nodes, boolean)
}

fn divide_signed(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let minimum = Node::add_i64_into(nodes, i64::MIN);
	let at_minimum = LuaJITEqual::add_into(nodes, lhs, minimum);
	let negative_one = Node::add_i64_into(nodes, -1);
	let at_negative_one = LuaJITEqual::add_into(nodes, rhs, negative_one);
	let overflow = both(nodes, at_minimum, at_negative_one);
	let zero = is_zero(nodes, rhs);
	let trapping = either(nodes, overflow, zero);
	let condition = BooleanToInteger::add_into(nodes, trapping);

	guard_division(nodes, lhs, rhs, condition, LuaJITDivide::add_into)
}

fn divide_unsigned(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let zero = is_zero(nodes, rhs);
	let condition = BooleanToInteger::add_into(nodes, zero);

	guard_division(nodes, lhs, rhs, condition, unsigned_quotient)
}

fn remainder_signed(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let zero = is_zero(nodes, rhs);
	let condition = BooleanToInteger::add_into(nodes, zero);

	guard_division(nodes, lhs, rhs, condition, LuaJITModulo::add_into)
}

fn remainder_unsigned(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let zero = is_zero(nodes, rhs);
	let condition = BooleanToInteger::add_into(nodes, zero);

	guard_division(nodes, lhs, rhs, condition, unsigned_remainder)
}

fn guard_division(
	nodes: &mut Vec<Node>,
	lhs: Link,
	rhs: Link,
	condition: Link,
	value: fn(&mut Vec<Node>, Link, Link) -> Link,
) -> Link {
	let matcher = Match::add_if_into(
		nodes,
		vec![lhs, rhs],
		condition,
		|nodes, arguments| vec![value(nodes, Link(arguments, 0), Link(arguments, 1))],
		|nodes, _arguments| vec![Node::add_trap_into(nodes)],
	);

	Link(matcher, 0)
}

fn unsigned_quotient(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let lhs = CastU64::add_into(nodes, lhs);
	let rhs = CastU64::add_into(nodes, rhs);
	let result = LuaJITDivide::add_into(nodes, lhs, rhs);

	CastI64::add_into(nodes, result)
}

fn unsigned_remainder(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let lhs = CastU64::add_into(nodes, lhs);
	let rhs = CastU64::add_into(nodes, rhs);
	let result = LuaJITModulo::add_into(nodes, lhs, rhs);

	CastI64::add_into(nodes, result)
}

fn is_zero(nodes: &mut Vec<Node>, source: Link) -> Link {
	let zero = Node::add_i64_into(nodes, 0);

	LuaJITEqual::add_into(nodes, source, zero)
}

fn shift_amount(nodes: &mut Vec<Node>, rhs: Link) -> Link {
	let mask = Node::add_i64_into(nodes, SHIFT_MASK);

	BitAnd::add_into(nodes, rhs, mask)
}

fn shift_left(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let rhs = shift_amount(nodes, rhs);

	BitLShift::add_into(nodes, lhs, rhs)
}

fn shift_right_signed(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let rhs = shift_amount(nodes, rhs);

	BitArShift::add_into(nodes, lhs, rhs)
}

fn shift_right_unsigned(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let rhs = shift_amount(nodes, rhs);

	BitRShift::add_into(nodes, lhs, rhs)
}

fn less_than(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, is_signed: bool) -> Link {
	let (lhs, rhs) = comparison_operands(nodes, lhs, rhs, is_signed);

	LuaJITLessThan::add_into(nodes, lhs, rhs)
}

fn less_than_equal(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, is_signed: bool) -> Link {
	let (lhs, rhs) = comparison_operands(nodes, lhs, rhs, is_signed);

	LuaJITLessThanEqual::add_into(nodes, lhs, rhs)
}

fn comparison_operands(
	nodes: &mut Vec<Node>,
	lhs: Link,
	rhs: Link,
	is_signed: bool,
) -> (Link, Link) {
	if is_signed {
		(lhs, rhs)
	} else {
		(CastU64::add_into(nodes, lhs), CastU64::add_into(nodes, rhs))
	}
}

fn count_ones(nodes: &mut Vec<Node>, source: Link) -> Link {
	let one = Node::add_i64_into(nodes, 1);
	let shifted_one = BitRShift::add_into(nodes, source, one);
	let alternating_mask = Node::add_i64_into(nodes, 0x5555_5555_5555_5555);
	let masked = BitAnd::add_into(nodes, shifted_one, alternating_mask);
	let first = LuaJITSubtract::add_into(nodes, source, masked);

	let pair_mask = Node::add_i64_into(nodes, 0x3333_3333_3333_3333);
	let low = BitAnd::add_into(nodes, first, pair_mask);
	let two = Node::add_i64_into(nodes, 2);
	let shifted_two = BitRShift::add_into(nodes, first, two);
	let high = BitAnd::add_into(nodes, shifted_two, pair_mask);
	let second = LuaJITAdd::add_into(nodes, low, high);

	let four = Node::add_i64_into(nodes, 4);
	let shifted_four = BitRShift::add_into(nodes, second, four);
	let third = LuaJITAdd::add_into(nodes, second, shifted_four);
	let nibble_mask = Node::add_i64_into(nodes, 0x0F0F_0F0F_0F0F_0F0F);
	let third = BitAnd::add_into(nodes, third, nibble_mask);

	let fourth = fold_count(nodes, third, 8);
	let fifth = fold_count(nodes, fourth, 16);
	let sixth = fold_count(nodes, fifth, 32);
	let result_mask = Node::add_i64_into(nodes, 0x7F);

	BitAnd::add_into(nodes, sixth, result_mask)
}

fn fold_count(nodes: &mut Vec<Node>, source: Link, amount: i64) -> Link {
	let amount = Node::add_i64_into(nodes, amount);
	let shifted = BitRShift::add_into(nodes, source, amount);

	LuaJITAdd::add_into(nodes, source, shifted)
}

fn leading_zeros(nodes: &mut Vec<Node>, source: Link) -> Link {
	let zero = Node::add_i64_into(nodes, 0);
	let condition = LuaJITEqual::add_into(nodes, source, zero);
	let condition = BooleanToInteger::add_into(nodes, condition);
	let matcher = Match::add_if_into(
		nodes,
		vec![source],
		condition,
		|nodes, arguments| vec![leading_zeros_nonzero(nodes, Link(arguments, 0))],
		|nodes, _arguments| vec![Node::add_i64_into(nodes, 64)],
	);

	Link(matcher, 0)
}

fn leading_zeros_nonzero(nodes: &mut Vec<Node>, source: Link) -> Link {
	let result = Node::add_i64_into(nodes, 0);
	let (source, result) = leading_step(nodes, source, result, 32, 32);
	let (source, result) = leading_step(nodes, source, result, 48, 16);
	let (source, result) = leading_step(nodes, source, result, 56, 8);
	let (source, result) = leading_step(nodes, source, result, 60, 4);
	let (source, result) = leading_step(nodes, source, result, 62, 2);
	let check = Node::add_i64_into(nodes, 63);
	let shifted = BitRShift::add_into(nodes, source, check);
	let zero = Node::add_i64_into(nodes, 0);
	let condition = LuaJITEqual::add_into(nodes, shifted, zero);
	let condition = BooleanToInteger::add_into(nodes, condition);

	boolean_increment(nodes, result, condition)
}

fn leading_step(
	nodes: &mut Vec<Node>,
	source: Link,
	result: Link,
	check: i64,
	amount: i64,
) -> (Link, Link) {
	let check = Node::add_i64_into(nodes, check);
	let shifted = BitRShift::add_into(nodes, source, check);
	let zero = Node::add_i64_into(nodes, 0);
	let condition = LuaJITEqual::add_into(nodes, shifted, zero);
	let condition = BooleanToInteger::add_into(nodes, condition);
	let matcher = Match::add_if_into(
		nodes,
		vec![source, result],
		condition,
		|_nodes, arguments| vec![Link(arguments, 0), Link(arguments, 1)],
		move |nodes, arguments| {
			let amount = Node::add_i64_into(nodes, amount);
			let shifted_source = BitLShift::add_into(nodes, Link(arguments, 0), amount);
			let incremented_result = LuaJITAdd::add_into(nodes, Link(arguments, 1), amount);

			vec![shifted_source, incremented_result]
		},
	);

	(Link(matcher, 0), Link(matcher, 1))
}

fn trailing_zeros(nodes: &mut Vec<Node>, source: Link) -> Link {
	let zero = Node::add_i64_into(nodes, 0);
	let condition = LuaJITEqual::add_into(nodes, source, zero);
	let condition = BooleanToInteger::add_into(nodes, condition);
	let matcher = Match::add_if_into(
		nodes,
		vec![source],
		condition,
		|nodes, arguments| vec![trailing_zeros_nonzero(nodes, Link(arguments, 0))],
		|nodes, _arguments| vec![Node::add_i64_into(nodes, 64)],
	);

	Link(matcher, 0)
}

fn trailing_zeros_nonzero(nodes: &mut Vec<Node>, source: Link) -> Link {
	let result = Node::add_i64_into(nodes, 0);
	let (source, result) = trailing_step(nodes, source, result, 32, 32);
	let (source, result) = trailing_step(nodes, source, result, 48, 16);
	let (source, result) = trailing_step(nodes, source, result, 56, 8);
	let (source, result) = trailing_step(nodes, source, result, 60, 4);
	let (source, result) = trailing_step(nodes, source, result, 62, 2);
	let check = Node::add_i64_into(nodes, 63);
	let shifted = BitLShift::add_into(nodes, source, check);
	let zero = Node::add_i64_into(nodes, 0);
	let condition = LuaJITEqual::add_into(nodes, shifted, zero);
	let condition = BooleanToInteger::add_into(nodes, condition);

	boolean_increment(nodes, result, condition)
}

fn trailing_step(
	nodes: &mut Vec<Node>,
	source: Link,
	result: Link,
	check: i64,
	amount: i64,
) -> (Link, Link) {
	let check = Node::add_i64_into(nodes, check);
	let shifted = BitLShift::add_into(nodes, source, check);
	let zero = Node::add_i64_into(nodes, 0);
	let condition = LuaJITEqual::add_into(nodes, shifted, zero);
	let condition = BooleanToInteger::add_into(nodes, condition);
	let matcher = Match::add_if_into(
		nodes,
		vec![source, result],
		condition,
		|_nodes, arguments| vec![Link(arguments, 0), Link(arguments, 1)],
		move |nodes, arguments| {
			let amount = Node::add_i64_into(nodes, amount);
			let shifted_source = BitRShift::add_into(nodes, Link(arguments, 0), amount);
			let incremented_result = LuaJITAdd::add_into(nodes, Link(arguments, 1), amount);

			vec![shifted_source, incremented_result]
		},
	);

	(Link(matcher, 0), Link(matcher, 1))
}

fn boolean_increment(nodes: &mut Vec<Node>, result: Link, condition: Link) -> Link {
	let matcher = Match::add_if_into(
		nodes,
		vec![result],
		condition,
		|_nodes, arguments| vec![Link(arguments, 0)],
		|nodes, arguments| {
			let one = Node::add_i64_into(nodes, 1);
			let incremented = LuaJITAdd::add_into(nodes, Link(arguments, 0), one);

			vec![incremented]
		},
	);

	Link(matcher, 0)
}
