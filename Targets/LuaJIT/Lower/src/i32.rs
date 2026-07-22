//! Lowerings for 32-bit integer operations.

use ir_graph::{
	Link, Node,
	operation::integer::{BinaryOperator, CompareOperator, UnaryOperator},
	region::Match,
};
use luajit_foreign::{
	BitAnd, BitArShift, BitLRotate, BitLShift, BitOr, BitRRotate, BitRShift, BitXor,
	BooleanToInteger, CastI64, ForceI32, ForceU32, LuaJITAdd, LuaJITDivide, LuaJITEqual,
	LuaJITLessThan, LuaJITLessThanEqual, LuaJITModulo, LuaJITMultiply, LuaJITNotEqual,
	LuaJITSubtract, MathFloor, MathFmod, MathModf,
};

use crate::boolean::{both, either};

const SIGN_BIT: u32 = 0x8000_0000;
const SHIFT_MASK: u32 = 0x1F;

fn wrap(nodes: &mut Vec<Node>, source: Link) -> Link {
	ForceI32::add_into(nodes, source)
}

/// Lowers a 32-bit integer unary operation.
pub fn unary(nodes: &mut Vec<Node>, source: Link, operator: UnaryOperator) -> Link {
	match operator {
		UnaryOperator::CountOnes => count_ones(nodes, source),
		UnaryOperator::LeadingZeros => leading_zeros(nodes, source),
		UnaryOperator::TrailingZeros => trailing_zeros(nodes, source),
	}
}

/// Lowers a 32-bit integer binary operation.
pub fn binary(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, operator: BinaryOperator) -> Link {
	match operator {
		BinaryOperator::Add => add(nodes, lhs, rhs),
		BinaryOperator::Subtract => subtract(nodes, lhs, rhs),
		BinaryOperator::Multiply => multiply(nodes, lhs, rhs),
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
		BinaryOperator::RotateLeft => rotate_left(nodes, lhs, rhs),
		BinaryOperator::RotateRight => rotate_right(nodes, lhs, rhs),
	}
}

/// Lowers a 32-bit integer comparison.
pub fn compare(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, operator: CompareOperator) -> Link {
	let boolean = match operator {
		CompareOperator::Equal => LuaJITEqual::add_into(nodes, lhs, rhs),
		CompareOperator::NotEqual => LuaJITNotEqual::add_into(nodes, lhs, rhs),
		CompareOperator::LessThan { is_signed } => less_than(nodes, lhs, rhs, is_signed),
		CompareOperator::LessThanEqual { is_signed } => less_than_equal(nodes, lhs, rhs, is_signed),
	};

	BooleanToInteger::add_into(nodes, boolean)
}

fn add(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let sum = LuaJITAdd::add_into(nodes, lhs, rhs);

	wrap(nodes, sum)
}

fn subtract(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let difference = LuaJITSubtract::add_into(nodes, lhs, rhs);

	wrap(nodes, difference)
}

fn multiply(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let lhs = CastI64::add_into(nodes, lhs);
	let rhs = CastI64::add_into(nodes, rhs);
	let product = LuaJITMultiply::add_into(nodes, lhs, rhs);

	wrap(nodes, product)
}

fn divide_signed(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let condition = signed_division_traps(nodes, lhs, rhs);

	guard_division(nodes, lhs, rhs, condition, signed_quotient)
}

fn divide_unsigned(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let condition = divisor_is_zero(nodes, rhs);

	guard_division(nodes, lhs, rhs, condition, unsigned_quotient)
}

fn remainder_signed(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let condition = divisor_is_zero(nodes, rhs);

	guard_division(nodes, lhs, rhs, condition, signed_remainder)
}

fn remainder_unsigned(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let condition = divisor_is_zero(nodes, rhs);

	guard_division(nodes, lhs, rhs, condition, unsigned_remainder)
}

// A divisor that fails its guard takes the on-true branch into a trap; otherwise
// the on-false branch produces the quotient or remainder.
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

fn signed_quotient(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let quotient = LuaJITDivide::add_into(nodes, lhs, rhs);
	let truncated = MathModf::add_into(nodes, quotient);

	wrap(nodes, truncated)
}

fn unsigned_quotient(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let lhs = ForceU32::add_into(nodes, lhs);
	let rhs = ForceU32::add_into(nodes, rhs);
	let quotient = LuaJITDivide::add_into(nodes, lhs, rhs);
	let floored = MathFloor::add_into(nodes, quotient);

	wrap(nodes, floored)
}

fn signed_remainder(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let remainder = MathFmod::add_into(nodes, lhs, rhs);

	wrap(nodes, remainder)
}

fn unsigned_remainder(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let lhs = ForceU32::add_into(nodes, lhs);
	let rhs = ForceU32::add_into(nodes, rhs);
	let modulo = LuaJITModulo::add_into(nodes, lhs, rhs);

	wrap(nodes, modulo)
}

// Signed division overflows only for the minimum dividend over a divisor of
// negative one; both that and a zero divisor trap.
fn signed_division_traps(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let overflow = is_overflow(nodes, lhs, rhs);
	let zero = is_zero(nodes, rhs);
	let trapping = either(nodes, overflow, zero);

	BooleanToInteger::add_into(nodes, trapping)
}

fn is_overflow(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let minimum = Node::add_i32_into(nodes, i32::MIN);
	let at_minimum = LuaJITEqual::add_into(nodes, lhs, minimum);
	let negative_one = Node::add_i32_into(nodes, -1);
	let at_negative_one = LuaJITEqual::add_into(nodes, rhs, negative_one);

	both(nodes, at_minimum, at_negative_one)
}

fn divisor_is_zero(nodes: &mut Vec<Node>, rhs: Link) -> Link {
	let zero = is_zero(nodes, rhs);

	BooleanToInteger::add_into(nodes, zero)
}

fn is_zero(nodes: &mut Vec<Node>, value: Link) -> Link {
	let zero = Node::add_i32_into(nodes, 0);

	LuaJITEqual::add_into(nodes, value, zero)
}

fn shift_left(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let rhs = BitAnd::add_fast_into(nodes, rhs, SHIFT_MASK);

	BitLShift::add_into(nodes, lhs, rhs)
}

fn shift_right_signed(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let rhs = BitAnd::add_fast_into(nodes, rhs, SHIFT_MASK);

	BitArShift::add_into(nodes, lhs, rhs)
}

fn shift_right_unsigned(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let rhs = BitAnd::add_fast_into(nodes, rhs, SHIFT_MASK);

	BitRShift::add_into(nodes, lhs, rhs)
}

fn rotate_left(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let rhs = BitAnd::add_fast_into(nodes, rhs, SHIFT_MASK);

	BitLRotate::add_into(nodes, lhs, rhs)
}

fn rotate_right(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let rhs = BitAnd::add_fast_into(nodes, rhs, SHIFT_MASK);

	BitRRotate::add_into(nodes, lhs, rhs)
}

fn less_than(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, is_signed: bool) -> Link {
	let (lhs, rhs) = order_operands(nodes, lhs, rhs, is_signed);

	LuaJITLessThan::add_into(nodes, lhs, rhs)
}

fn less_than_equal(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, is_signed: bool) -> Link {
	let (lhs, rhs) = order_operands(nodes, lhs, rhs, is_signed);

	LuaJITLessThanEqual::add_into(nodes, lhs, rhs)
}

fn order_operands(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, is_signed: bool) -> (Link, Link) {
	if is_signed {
		(lhs, rhs)
	} else {
		(
			BitXor::add_fast_into(nodes, lhs, SIGN_BIT),
			BitXor::add_fast_into(nodes, rhs, SIGN_BIT),
		)
	}
}

/// Lowers a 32-bit population count into a `bit` SWAR tree.
pub fn count_ones(nodes: &mut Vec<Node>, source: Link) -> Link {
	let one = Node::add_i32_into(nodes, 1);
	let shifted = BitRShift::add_into(nodes, source, one);
	let masked = BitAnd::add_fast_into(nodes, shifted, 0x5555_5555);
	let pairs = LuaJITSubtract::add_into(nodes, source, masked);

	let low_pairs = BitAnd::add_fast_into(nodes, pairs, 0x3333_3333);
	let two = Node::add_i32_into(nodes, 2);
	let shifted_pairs = BitRShift::add_into(nodes, pairs, two);
	let high_pairs = BitAnd::add_fast_into(nodes, shifted_pairs, 0x3333_3333);
	let nibbles = LuaJITAdd::add_into(nodes, low_pairs, high_pairs);

	let four = Node::add_i32_into(nodes, 4);
	let shifted_nibbles = BitRShift::add_into(nodes, nibbles, four);
	let nibble_sums = LuaJITAdd::add_into(nodes, nibbles, shifted_nibbles);
	let bytes = BitAnd::add_fast_into(nodes, nibble_sums, 0x0F0F_0F0F);

	let eight = Node::add_i32_into(nodes, 8);
	let shifted_bytes = BitRShift::add_into(nodes, bytes, eight);
	let byte_sums = LuaJITAdd::add_into(nodes, bytes, shifted_bytes);

	let sixteen = Node::add_i32_into(nodes, 16);
	let shifted_byte_sums = BitRShift::add_into(nodes, byte_sums, sixteen);
	let total = LuaJITAdd::add_into(nodes, byte_sums, shifted_byte_sums);

	BitAnd::add_fast_into(nodes, total, 0x0000_003F)
}

fn leading_zeros(nodes: &mut Vec<Node>, source: Link) -> Link {
	let zero = Node::add_i32_into(nodes, 0);
	let condition = LuaJITEqual::add_into(nodes, source, zero);
	let condition = BooleanToInteger::add_into(nodes, condition);
	let matcher = Match::add_if_into(
		nodes,
		vec![source],
		condition,
		|nodes, arguments| vec![leading_zeros_nonzero(nodes, Link(arguments, 0))],
		|nodes, _arguments| vec![Node::add_i32_into(nodes, 32)],
	);

	Link(matcher, 0)
}

fn leading_zeros_nonzero(nodes: &mut Vec<Node>, source: Link) -> Link {
	let result = Node::add_i32_into(nodes, 0);
	let (source, result) = leading_step(nodes, source, result, 16, 16);
	let (source, result) = leading_step(nodes, source, result, 24, 8);
	let (source, result) = leading_step(nodes, source, result, 28, 4);
	let (source, result) = leading_step(nodes, source, result, 30, 2);
	let shifted = BitRShift::add_fast_into(nodes, source, 31);
	let zero = Node::add_i32_into(nodes, 0);
	let condition = LuaJITEqual::add_into(nodes, shifted, zero);
	let condition = BooleanToInteger::add_into(nodes, condition);

	boolean_increment(nodes, result, condition)
}

fn leading_step(
	nodes: &mut Vec<Node>,
	source: Link,
	result: Link,
	check: u32,
	amount: u32,
) -> (Link, Link) {
	let shifted = BitRShift::add_fast_into(nodes, source, check);
	let zero = Node::add_i32_into(nodes, 0);
	let condition = LuaJITEqual::add_into(nodes, shifted, zero);
	let condition = BooleanToInteger::add_into(nodes, condition);
	let matcher = Match::add_if_into(
		nodes,
		vec![source, result],
		condition,
		|_nodes, arguments| vec![Link(arguments, 0), Link(arguments, 1)],
		|nodes, arguments| {
			let shifted_source = BitLShift::add_fast_into(nodes, Link(arguments, 0), amount);
			let amount = Node::add_i32_into(nodes, i32::try_from(amount).unwrap());
			let incremented_result = LuaJITAdd::add_into(nodes, Link(arguments, 1), amount);

			vec![shifted_source, incremented_result]
		},
	);

	(Link(matcher, 0), Link(matcher, 1))
}

fn trailing_zeros(nodes: &mut Vec<Node>, source: Link) -> Link {
	let zero = Node::add_i32_into(nodes, 0);
	let condition = LuaJITEqual::add_into(nodes, source, zero);
	let condition = BooleanToInteger::add_into(nodes, condition);
	let matcher = Match::add_if_into(
		nodes,
		vec![source],
		condition,
		|nodes, arguments| vec![trailing_zeros_nonzero(nodes, Link(arguments, 0))],
		|nodes, _arguments| vec![Node::add_i32_into(nodes, 32)],
	);

	Link(matcher, 0)
}

fn trailing_zeros_nonzero(nodes: &mut Vec<Node>, source: Link) -> Link {
	let result = Node::add_i32_into(nodes, 0);
	let (source, result) = trailing_step(nodes, source, result, 16, 16);
	let (source, result) = trailing_step(nodes, source, result, 24, 8);
	let (source, result) = trailing_step(nodes, source, result, 28, 4);
	let (source, result) = trailing_step(nodes, source, result, 30, 2);
	let shifted = BitLShift::add_fast_into(nodes, source, 31);
	let zero = Node::add_i32_into(nodes, 0);
	let condition = LuaJITEqual::add_into(nodes, shifted, zero);
	let condition = BooleanToInteger::add_into(nodes, condition);

	boolean_increment(nodes, result, condition)
}

fn trailing_step(
	nodes: &mut Vec<Node>,
	source: Link,
	result: Link,
	check: u32,
	amount: u32,
) -> (Link, Link) {
	let shifted = BitLShift::add_fast_into(nodes, source, check);
	let zero = Node::add_i32_into(nodes, 0);
	let condition = LuaJITEqual::add_into(nodes, shifted, zero);
	let condition = BooleanToInteger::add_into(nodes, condition);
	let matcher = Match::add_if_into(
		nodes,
		vec![source, result],
		condition,
		|_nodes, arguments| vec![Link(arguments, 0), Link(arguments, 1)],
		|nodes, arguments| {
			let shifted_source = BitRShift::add_fast_into(nodes, Link(arguments, 0), amount);
			let amount = Node::add_i32_into(nodes, i32::try_from(amount).unwrap());
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
			let one = Node::add_i32_into(nodes, 1);
			let incremented = LuaJITAdd::add_into(nodes, Link(arguments, 0), one);

			vec![incremented]
		},
	);

	Link(matcher, 0)
}
