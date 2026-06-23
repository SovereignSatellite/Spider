//! Lowerings for 32-bit integer operations.

use ir_graph::{
	Link, Node,
	operation::integer::{BinaryOperator, CompareOperator, UnaryOperator},
	region::Match,
};
use luau_foreign::{
	Bit32And, Bit32ArShift, Bit32CountLz, Bit32CountRz, Bit32LRotate, Bit32LShift, Bit32Or,
	Bit32RRotate, Bit32RShift, Bit32Xor, BooleanToInteger, LuauAdd, LuauAnd, LuauDivide, LuauEqual,
	LuauFloorDivide, LuauLessThan, LuauLessThanEqual, LuauModulo, LuauMultiply, LuauNotEqual,
	LuauOr, LuauSubtract, MathFmod, MathModf,
};

const SIGN_BIT: u32 = 0x8000_0000;
const SHIFT_MASK: u32 = 0x1F;
const HALF_WIDTH: u32 = 16;
const HALF_MASK: u32 = 0xFFFF;
const SMALL_PRODUCT: i32 = 0x0800_0000;
const MINIMUM_SIGNED_BITS: f64 = 2_147_483_648.0;
const NEGATIVE_ONE_BITS: f64 = 4_294_967_295.0;

/// Reinterprets an unsigned 32-bit word as its signed value.
pub fn to_signed(nodes: &mut Vec<Node>, source: Link) -> Link {
	let flipped = Bit32Xor::add_fast_into(nodes, source, SIGN_BIT);

	LuauSubtract::add_fast_into(nodes, flipped, SIGN_BIT)
}

fn wrap(nodes: &mut Vec<Node>, source: Link) -> Link {
	Bit32Or::add_fast_into(nodes, source, 0)
}

/// Lowers a 32-bit integer unary operation.
pub fn unary(nodes: &mut Vec<Node>, source: Link, operator: UnaryOperator) -> Link {
	match operator {
		UnaryOperator::CountOnes => count_ones(nodes, source),
		UnaryOperator::LeadingZeros => Bit32CountLz::add_into(nodes, source),
		UnaryOperator::TrailingZeros => Bit32CountRz::add_into(nodes, source),
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
		BinaryOperator::And => Bit32And::add_into(nodes, lhs, rhs),
		BinaryOperator::Or => Bit32Or::add_into(nodes, lhs, rhs),
		BinaryOperator::ExclusiveOr => Bit32Xor::add_into(nodes, lhs, rhs),
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
		CompareOperator::Equal => LuauEqual::add_into(nodes, lhs, rhs),
		CompareOperator::NotEqual => LuauNotEqual::add_into(nodes, lhs, rhs),
		CompareOperator::LessThan { is_signed } => less_than(nodes, lhs, rhs, is_signed),
		CompareOperator::GreaterThan { is_signed } => less_than(nodes, rhs, lhs, is_signed),
		CompareOperator::LessThanEqual { is_signed } => less_than_equal(nodes, lhs, rhs, is_signed),
		CompareOperator::GreaterThanEqual { is_signed } => {
			less_than_equal(nodes, rhs, lhs, is_signed)
		}
	};

	BooleanToInteger::add_into(nodes, boolean)
}

fn add(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let sum = LuauAdd::add_into(nodes, lhs, rhs);

	wrap(nodes, sum)
}

fn subtract(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let difference = LuauSubtract::add_into(nodes, lhs, rhs);

	wrap(nodes, difference)
}

// A direct `lhs * rhs` is exact only while the product stays within the f64
// mantissa; once the operands grow, multiply the 16-bit halves and recombine.
fn multiply(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let condition = is_small_product(nodes, lhs, rhs);
	let matcher = Match::add_if_into(
		nodes,
		vec![lhs, rhs],
		condition,
		|nodes, arguments| {
			vec![multiply_halves(
				nodes,
				Link(arguments, 0),
				Link(arguments, 1),
			)]
		},
		|nodes, arguments| {
			vec![multiply_direct(
				nodes,
				Link(arguments, 0),
				Link(arguments, 1),
			)]
		},
	);

	Link(matcher, 0)
}

fn is_small_product(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let sum = LuauAdd::add_into(nodes, lhs, rhs);
	let threshold = Node::add_i32_into(nodes, SMALL_PRODUCT);
	let small = LuauLessThan::add_into(nodes, sum, threshold);

	BooleanToInteger::add_into(nodes, small)
}

fn multiply_direct(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let product = LuauMultiply::add_into(nodes, lhs, rhs);

	wrap(nodes, product)
}

fn multiply_halves(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let high_lhs = Bit32RShift::add_fast_into(nodes, lhs, HALF_WIDTH);
	let low_lhs = Bit32And::add_fast_into(nodes, lhs, HALF_MASK);
	let high_rhs = Bit32RShift::add_fast_into(nodes, rhs, HALF_WIDTH);
	let low_rhs = Bit32And::add_fast_into(nodes, rhs, HALF_MASK);

	let low = LuauMultiply::add_into(nodes, low_lhs, low_rhs);
	let cross = cross_terms(nodes, high_lhs, low_lhs, high_rhs, low_rhs);
	let shifted = Bit32LShift::add_fast_into(nodes, cross, HALF_WIDTH);
	let combined = LuauAdd::add_into(nodes, low, shifted);

	wrap(nodes, combined)
}

fn cross_terms(
	nodes: &mut Vec<Node>,
	high_lhs: Link,
	low_lhs: Link,
	high_rhs: Link,
	low_rhs: Link,
) -> Link {
	let first = LuauMultiply::add_into(nodes, high_lhs, low_rhs);
	let second = LuauMultiply::add_into(nodes, low_lhs, high_rhs);

	LuauAdd::add_into(nodes, first, second)
}

fn divide_signed(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let condition = signed_division_traps(nodes, lhs, rhs);

	guard_division(nodes, lhs, rhs, condition, signed_quotient)
}

fn divide_unsigned(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let condition = divisor_is_zero(nodes, rhs);

	guard_division(nodes, lhs, rhs, condition, LuauFloorDivide::add_into)
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
	let lhs = to_signed(nodes, lhs);
	let rhs = to_signed(nodes, rhs);
	let quotient = LuauDivide::add_into(nodes, lhs, rhs);
	let truncated = MathModf::add_into(nodes, quotient);

	wrap(nodes, truncated)
}

fn signed_remainder(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let lhs = to_signed(nodes, lhs);
	let rhs = to_signed(nodes, rhs);
	let remainder = MathFmod::add_into(nodes, lhs, rhs);

	wrap(nodes, remainder)
}

fn unsigned_remainder(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let modulo = LuauModulo::add_into(nodes, lhs, rhs);

	wrap(nodes, modulo)
}

// Signed division overflows only for the minimum dividend over a divisor of
// negative one; both that and a zero divisor trap.
fn signed_division_traps(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let overflow = is_overflow(nodes, lhs, rhs);
	let zero = is_zero(nodes, rhs);
	let trapping = LuauOr::add_into(nodes, overflow, zero);

	BooleanToInteger::add_into(nodes, trapping)
}

fn is_overflow(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let minimum = Node::add_f64_into(nodes, MINIMUM_SIGNED_BITS);
	let at_minimum = LuauEqual::add_into(nodes, lhs, minimum);
	let negative_one = Node::add_f64_into(nodes, NEGATIVE_ONE_BITS);
	let at_negative_one = LuauEqual::add_into(nodes, rhs, negative_one);

	LuauAnd::add_into(nodes, at_minimum, at_negative_one)
}

fn divisor_is_zero(nodes: &mut Vec<Node>, rhs: Link) -> Link {
	let zero = is_zero(nodes, rhs);

	BooleanToInteger::add_into(nodes, zero)
}

fn is_zero(nodes: &mut Vec<Node>, value: Link) -> Link {
	let zero = Node::add_i32_into(nodes, 0);

	LuauEqual::add_into(nodes, value, zero)
}

fn shift_left(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let rhs = Bit32And::add_fast_into(nodes, rhs, SHIFT_MASK);

	Bit32LShift::add_into(nodes, lhs, rhs)
}

fn shift_right_signed(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let rhs = Bit32And::add_fast_into(nodes, rhs, SHIFT_MASK);

	Bit32ArShift::add_into(nodes, lhs, rhs)
}

fn shift_right_unsigned(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let rhs = Bit32And::add_fast_into(nodes, rhs, SHIFT_MASK);

	Bit32RShift::add_into(nodes, lhs, rhs)
}

fn rotate_left(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let rhs = Bit32And::add_fast_into(nodes, rhs, SHIFT_MASK);

	Bit32LRotate::add_into(nodes, lhs, rhs)
}

fn rotate_right(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let rhs = Bit32And::add_fast_into(nodes, rhs, SHIFT_MASK);

	Bit32RRotate::add_into(nodes, lhs, rhs)
}

fn less_than(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, is_signed: bool) -> Link {
	let (lhs, rhs) = order_operands(nodes, lhs, rhs, is_signed);

	LuauLessThan::add_into(nodes, lhs, rhs)
}

fn less_than_equal(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, is_signed: bool) -> Link {
	let (lhs, rhs) = order_operands(nodes, lhs, rhs, is_signed);

	LuauLessThanEqual::add_into(nodes, lhs, rhs)
}

fn order_operands(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, is_signed: bool) -> (Link, Link) {
	if is_signed {
		(to_signed(nodes, lhs), to_signed(nodes, rhs))
	} else {
		(lhs, rhs)
	}
}

/// Lowers a 32-bit population count into a `bit32` SWAR tree.
pub fn count_ones(nodes: &mut Vec<Node>, source: Link) -> Link {
	let one = Node::add_i32_into(nodes, 1);
	let shifted = Bit32RShift::add_into(nodes, source, one);
	let masked = Bit32And::add_fast_into(nodes, shifted, 0x5555_5555);
	let pairs = LuauSubtract::add_into(nodes, source, masked);

	let low_pairs = Bit32And::add_fast_into(nodes, pairs, 0x3333_3333);
	let two = Node::add_i32_into(nodes, 2);
	let shifted_pairs = Bit32RShift::add_into(nodes, pairs, two);
	let high_pairs = Bit32And::add_fast_into(nodes, shifted_pairs, 0x3333_3333);
	let nibbles = LuauAdd::add_into(nodes, low_pairs, high_pairs);

	let four = Node::add_i32_into(nodes, 4);
	let shifted_nibbles = Bit32RShift::add_into(nodes, nibbles, four);
	let nibble_sums = LuauAdd::add_into(nodes, nibbles, shifted_nibbles);
	let bytes = Bit32And::add_fast_into(nodes, nibble_sums, 0x0F0F_0F0F);

	let eight = Node::add_i32_into(nodes, 8);
	let shifted_bytes = Bit32RShift::add_into(nodes, bytes, eight);
	let byte_sums = LuauAdd::add_into(nodes, bytes, shifted_bytes);

	let sixteen = Node::add_i32_into(nodes, 16);
	let shifted_byte_sums = Bit32RShift::add_into(nodes, byte_sums, sixteen);
	let total = LuauAdd::add_into(nodes, byte_sums, shifted_byte_sums);

	Bit32And::add_fast_into(nodes, total, 0x0000_003F)
}
