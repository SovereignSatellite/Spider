//! Lowerings for 64-bit integer operations on the two-word representation.

use ir_graph::{
	Link, Node,
	operation::integer::{BinaryOperator, CompareOperator, UnaryOperator},
	region::Match,
};
use luau_foreign::{
	Bit32And, Bit32ArShift, Bit32CountLz, Bit32CountRz, Bit32LShift, Bit32Or, Bit32RShift,
	Bit32Xor, BooleanToInteger, FlipMostSignificant, FromBitsI64, IntoBitsI64, LuauAdd, LuauAnd,
	LuauEqual, LuauLessThan, LuauLessThanEqual, LuauNotEqual, LuauOr, LuauSubtract,
};

use crate::i32 as lower_i32;

const WORD_BITS: i32 = 32;
const SHIFT_MASK: u32 = 0x3F;
const WORD_MODULUS: f64 = 4_294_967_296.0;

/// Lowers a 64-bit integer unary operation.
pub fn unary(nodes: &mut Vec<Node>, source: Link, operator: UnaryOperator) -> Link {
	match operator {
		UnaryOperator::CountOnes => count_ones(nodes, source),
		UnaryOperator::LeadingZeros => leading_zeros(nodes, source),
		UnaryOperator::TrailingZeros => trailing_zeros(nodes, source),
	}
}

/// Lowers a 64-bit integer binary operation, if it is trivial.
pub fn binary(
	nodes: &mut Vec<Node>,
	lhs: Link,
	rhs: Link,
	operator: BinaryOperator,
) -> Option<Link> {
	Some(match operator {
		BinaryOperator::Add => add(nodes, lhs, rhs),
		BinaryOperator::Subtract => subtract(nodes, lhs, rhs),
		BinaryOperator::Multiply
		| BinaryOperator::Divide { .. }
		| BinaryOperator::Remainder { .. } => return None,
		BinaryOperator::And => bitwise(nodes, lhs, rhs, Bit32And::add_into),
		BinaryOperator::Or => bitwise(nodes, lhs, rhs, Bit32Or::add_into),
		BinaryOperator::ExclusiveOr => bitwise(nodes, lhs, rhs, Bit32Xor::add_into),
		BinaryOperator::ShiftLeft => shift_left(nodes, lhs, rhs),
		BinaryOperator::ShiftRight { is_signed: true } => shift_right_signed(nodes, lhs, rhs),
		BinaryOperator::ShiftRight { is_signed: false } => shift_right_unsigned(nodes, lhs, rhs),
		BinaryOperator::RotateLeft => rotate_left(nodes, lhs, rhs),
		BinaryOperator::RotateRight => rotate_right(nodes, lhs, rhs),
	})
}

/// Lowers a 64-bit integer comparison.
pub fn compare(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, operator: CompareOperator) -> Link {
	let boolean = match operator {
		CompareOperator::Equal => equal(nodes, lhs, rhs),
		CompareOperator::NotEqual => not_equal(nodes, lhs, rhs),
		CompareOperator::LessThan { is_signed } => less_than(nodes, lhs, rhs, is_signed),
		CompareOperator::GreaterThan { is_signed } => less_than(nodes, rhs, lhs, is_signed),
		CompareOperator::LessThanEqual { is_signed } => less_than_equal(nodes, lhs, rhs, is_signed),
		CompareOperator::GreaterThanEqual { is_signed } => {
			less_than_equal(nodes, rhs, lhs, is_signed)
		}
	};

	BooleanToInteger::add_into(nodes, boolean)
}

fn split_both(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> (Link, Link, Link, Link) {
	let (low_lhs, high_lhs) = FromBitsI64::add_into(nodes, lhs);
	let (low_rhs, high_rhs) = FromBitsI64::add_into(nodes, rhs);

	(low_lhs, high_lhs, low_rhs, high_rhs)
}

fn bitwise(
	nodes: &mut Vec<Node>,
	lhs: Link,
	rhs: Link,
	combine: fn(&mut Vec<Node>, Link, Link) -> Link,
) -> Link {
	let (low_lhs, high_lhs, low_rhs, high_rhs) = split_both(nodes, lhs, rhs);

	let low = combine(nodes, low_lhs, low_rhs);
	let high = combine(nodes, high_lhs, high_rhs);

	IntoBitsI64::add_into(nodes, low, high)
}

fn carry_overflow(nodes: &mut Vec<Node>, word: Link) -> Link {
	let modulus = Node::add_f64_into(nodes, WORD_MODULUS);
	let overflowed = LuauLessThanEqual::add_into(nodes, modulus, word);

	BooleanToInteger::add_into(nodes, overflowed)
}

fn borrow_underflow(nodes: &mut Vec<Node>, word: Link) -> Link {
	let zero = Node::add_i32_into(nodes, 0);
	let underflowed = LuauLessThan::add_into(nodes, word, zero);

	BooleanToInteger::add_into(nodes, underflowed)
}

fn drop_modulus(nodes: &mut Vec<Node>, word: Link) -> Link {
	let modulus = Node::add_f64_into(nodes, WORD_MODULUS);

	LuauSubtract::add_into(nodes, word, modulus)
}

fn raise_modulus(nodes: &mut Vec<Node>, word: Link) -> Link {
	let modulus = Node::add_f64_into(nodes, WORD_MODULUS);

	LuauAdd::add_into(nodes, word, modulus)
}

fn increment(nodes: &mut Vec<Node>, word: Link) -> Link {
	let one = Node::add_i32_into(nodes, 1);

	LuauAdd::add_into(nodes, word, one)
}

fn decrement(nodes: &mut Vec<Node>, word: Link) -> Link {
	let one = Node::add_i32_into(nodes, 1);

	LuauSubtract::add_into(nodes, word, one)
}

struct Propagation {
	combine: fn(&mut Vec<Node>, Link, Link) -> Link,
	overflow: fn(&mut Vec<Node>, Link) -> Link,
	adjust_word: fn(&mut Vec<Node>, Link) -> Link,
	adjust_carry: fn(&mut Vec<Node>, Link) -> Link,
}

fn propagate(nodes: &mut Vec<Node>, low: Link, high: Link, policy: &Propagation) -> (Link, Link) {
	let adjust_word = policy.adjust_word;
	let adjust_carry = policy.adjust_carry;
	let condition = (policy.overflow)(nodes, low);
	let matcher = Match::add_if_into(
		nodes,
		vec![low, high],
		condition,
		|_nodes, arguments| vec![Link(arguments, 0), Link(arguments, 1)],
		|nodes, arguments| {
			vec![
				adjust_word(nodes, Link(arguments, 0)),
				adjust_carry(nodes, Link(arguments, 1)),
			]
		},
	);

	(Link(matcher, 0), Link(matcher, 1))
}

fn wrap(nodes: &mut Vec<Node>, high: Link, policy: &Propagation) -> Link {
	let adjust_word = policy.adjust_word;
	let condition = (policy.overflow)(nodes, high);
	let matcher = Match::add_if_into(
		nodes,
		vec![high],
		condition,
		|_nodes, arguments| vec![Link(arguments, 0)],
		|nodes, arguments| vec![adjust_word(nodes, Link(arguments, 0))],
	);

	Link(matcher, 0)
}

fn carrying(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, policy: &Propagation) -> Link {
	let (low_lhs, high_lhs, low_rhs, high_rhs) = split_both(nodes, lhs, rhs);

	let low = (policy.combine)(nodes, low_lhs, low_rhs);
	let high = (policy.combine)(nodes, high_lhs, high_rhs);

	let (low, high) = propagate(nodes, low, high, policy);
	let high = wrap(nodes, high, policy);

	IntoBitsI64::add_into(nodes, low, high)
}

fn add(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let policy = Propagation {
		combine: LuauAdd::add_into,
		overflow: carry_overflow,
		adjust_word: drop_modulus,
		adjust_carry: increment,
	};

	carrying(nodes, lhs, rhs, &policy)
}

fn subtract(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let policy = Propagation {
		combine: LuauSubtract::add_into,
		overflow: borrow_underflow,
		adjust_word: raise_modulus,
		adjust_carry: decrement,
	};

	carrying(nodes, lhs, rhs, &policy)
}

fn at_least_word(nodes: &mut Vec<Node>, amount: Link) -> Link {
	let word_bits = Node::add_i32_into(nodes, WORD_BITS);
	let crosses = LuauLessThanEqual::add_into(nodes, word_bits, amount);

	BooleanToInteger::add_into(nodes, crosses)
}

fn word_complement(nodes: &mut Vec<Node>, amount: Link) -> Link {
	let word_bits = Node::add_i32_into(nodes, WORD_BITS);

	LuauSubtract::add_into(nodes, word_bits, amount)
}

fn excess_amount(nodes: &mut Vec<Node>, amount: Link) -> Link {
	let word_bits = Node::add_i32_into(nodes, WORD_BITS);

	LuauSubtract::add_into(nodes, amount, word_bits)
}

fn fill_sign(nodes: &mut Vec<Node>, high: Link) -> Link {
	let top = Node::add_i32_into(nodes, WORD_BITS - 1);

	Bit32ArShift::add_into(nodes, high, top)
}

fn shift_left_near(nodes: &mut Vec<Node>, low: Link, high: Link, amount: Link) -> Link {
	let shifted_low = Bit32LShift::add_into(nodes, low, amount);
	let complement = word_complement(nodes, amount);
	let carried = Bit32RShift::add_into(nodes, low, complement);
	let shifted_high = Bit32LShift::add_into(nodes, high, amount);
	let merged_high = Bit32Or::add_into(nodes, shifted_high, carried);

	IntoBitsI64::add_into(nodes, shifted_low, merged_high)
}

fn shift_left_far(nodes: &mut Vec<Node>, low: Link, _high: Link, amount: Link) -> Link {
	let excess = excess_amount(nodes, amount);
	let shifted = Bit32LShift::add_into(nodes, low, excess);
	let zero = Node::add_i32_into(nodes, 0);

	IntoBitsI64::add_into(nodes, zero, shifted)
}

// The low word of a within-word right shift is identical for the logical and
// arithmetic variants; only the high word differs (`Bit32RShift` vs `Bit32ArShift`).
fn merge_low_on_right_shift(nodes: &mut Vec<Node>, low: Link, high: Link, amount: Link) -> Link {
	let shifted_low = Bit32RShift::add_into(nodes, low, amount);
	let complement = word_complement(nodes, amount);
	let carried = Bit32LShift::add_into(nodes, high, complement);

	Bit32Or::add_into(nodes, shifted_low, carried)
}

fn shift_right_unsigned_near(nodes: &mut Vec<Node>, low: Link, high: Link, amount: Link) -> Link {
	let merged_low = merge_low_on_right_shift(nodes, low, high, amount);
	let shifted_high = Bit32RShift::add_into(nodes, high, amount);

	IntoBitsI64::add_into(nodes, merged_low, shifted_high)
}

fn shift_right_unsigned_far(nodes: &mut Vec<Node>, _low: Link, high: Link, amount: Link) -> Link {
	let excess = excess_amount(nodes, amount);
	let shifted = Bit32RShift::add_into(nodes, high, excess);
	let zero = Node::add_i32_into(nodes, 0);

	IntoBitsI64::add_into(nodes, shifted, zero)
}

fn shift_right_signed_near(nodes: &mut Vec<Node>, low: Link, high: Link, amount: Link) -> Link {
	let merged_low = merge_low_on_right_shift(nodes, low, high, amount);
	let shifted_high = Bit32ArShift::add_into(nodes, high, amount);

	IntoBitsI64::add_into(nodes, merged_low, shifted_high)
}

fn shift_right_signed_far(nodes: &mut Vec<Node>, _low: Link, high: Link, amount: Link) -> Link {
	let excess = excess_amount(nodes, amount);
	let shifted = Bit32ArShift::add_into(nodes, high, excess);
	let sign = fill_sign(nodes, high);

	IntoBitsI64::add_into(nodes, shifted, sign)
}

fn variable_shift(
	nodes: &mut Vec<Node>,
	lhs: Link,
	rhs: Link,
	near: fn(&mut Vec<Node>, Link, Link, Link) -> Link,
	far: fn(&mut Vec<Node>, Link, Link, Link) -> Link,
) -> Link {
	let (low_rhs, _) = FromBitsI64::add_into(nodes, rhs);
	let amount = Bit32And::add_fast_into(nodes, low_rhs, SHIFT_MASK);
	let (low, high) = FromBitsI64::add_into(nodes, lhs);

	let condition = at_least_word(nodes, amount);
	let matcher = Match::add_if_into(
		nodes,
		vec![low, high, amount],
		condition,
		|nodes, arguments| {
			vec![near(
				nodes,
				Link(arguments, 0),
				Link(arguments, 1),
				Link(arguments, 2),
			)]
		},
		|nodes, arguments| {
			vec![far(
				nodes,
				Link(arguments, 0),
				Link(arguments, 1),
				Link(arguments, 2),
			)]
		},
	);

	Link(matcher, 0)
}

fn shift_left(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	variable_shift(nodes, lhs, rhs, shift_left_near, shift_left_far)
}

fn shift_right_unsigned(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	variable_shift(
		nodes,
		lhs,
		rhs,
		shift_right_unsigned_near,
		shift_right_unsigned_far,
	)
}

fn shift_right_signed(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	variable_shift(
		nodes,
		lhs,
		rhs,
		shift_right_signed_near,
		shift_right_signed_far,
	)
}

fn complement_amount(nodes: &mut Vec<Node>, rhs: Link) -> Link {
	let full = Node::add_i64_into(nodes, 64);

	subtract(nodes, full, rhs)
}

fn rotate_left(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let forward = shift_left(nodes, lhs, rhs);
	let complement = complement_amount(nodes, rhs);
	let backward = shift_right_unsigned(nodes, lhs, complement);

	bitwise(nodes, forward, backward, Bit32Or::add_into)
}

fn rotate_right(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let forward = shift_right_unsigned(nodes, lhs, rhs);
	let complement = complement_amount(nodes, rhs);
	let backward = shift_left(nodes, lhs, complement);

	bitwise(nodes, forward, backward, Bit32Or::add_into)
}

fn count_ones(nodes: &mut Vec<Node>, source: Link) -> Link {
	let (low, high) = FromBitsI64::add_into(nodes, source);

	let low = lower_i32::count_ones(nodes, low);
	let high = lower_i32::count_ones(nodes, high);
	let total = LuauAdd::add_into(nodes, low, high);

	let zero = Node::add_i32_into(nodes, 0);

	IntoBitsI64::add_into(nodes, total, zero)
}

fn count_zeros(
	nodes: &mut Vec<Node>,
	tested: Link,
	other: Link,
	count: fn(&mut Vec<Node>, Link) -> Link,
) -> Link {
	let condition = is_zero(nodes, tested);
	let matcher = Match::add_if_into(
		nodes,
		vec![tested, other],
		condition,
		|nodes, arguments| vec![count(nodes, Link(arguments, 0))],
		|nodes, arguments| {
			let counted = count(nodes, Link(arguments, 1));
			let word_bits = Node::add_i32_into(nodes, WORD_BITS);

			vec![LuauAdd::add_into(nodes, counted, word_bits)]
		},
	);

	let zero = Node::add_i32_into(nodes, 0);

	IntoBitsI64::add_into(nodes, Link(matcher, 0), zero)
}

fn leading_zeros(nodes: &mut Vec<Node>, source: Link) -> Link {
	let (low, high) = FromBitsI64::add_into(nodes, source);

	count_zeros(nodes, high, low, Bit32CountLz::add_into)
}

fn trailing_zeros(nodes: &mut Vec<Node>, source: Link) -> Link {
	let (low, high) = FromBitsI64::add_into(nodes, source);

	count_zeros(nodes, low, high, Bit32CountRz::add_into)
}

fn is_zero(nodes: &mut Vec<Node>, word: Link) -> Link {
	let zero = Node::add_i32_into(nodes, 0);
	let empty = LuauEqual::add_into(nodes, word, zero);

	BooleanToInteger::add_into(nodes, empty)
}

fn equal(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let (low_lhs, high_lhs, low_rhs, high_rhs) = split_both(nodes, lhs, rhs);

	let low = LuauEqual::add_into(nodes, low_lhs, low_rhs);
	let high = LuauEqual::add_into(nodes, high_lhs, high_rhs);

	LuauAnd::add_into(nodes, low, high)
}

fn not_equal(nodes: &mut Vec<Node>, lhs: Link, rhs: Link) -> Link {
	let (low_lhs, high_lhs, low_rhs, high_rhs) = split_both(nodes, lhs, rhs);

	let low = LuauNotEqual::add_into(nodes, low_lhs, low_rhs);
	let high = LuauNotEqual::add_into(nodes, high_lhs, high_rhs);

	LuauOr::add_into(nodes, low, high)
}

fn flip_signed(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, is_signed: bool) -> (Link, Link) {
	if is_signed {
		let lhs = FlipMostSignificant::add_into(nodes, lhs);
		let rhs = FlipMostSignificant::add_into(nodes, rhs);

		(lhs, rhs)
	} else {
		(lhs, rhs)
	}
}

fn order(
	nodes: &mut Vec<Node>,
	lhs: Link,
	rhs: Link,
	is_signed: bool,
	low_order: fn(&mut Vec<Node>, Link, Link) -> Link,
) -> Link {
	let (lhs, rhs) = flip_signed(nodes, lhs, rhs, is_signed);

	let (low_lhs, high_lhs, low_rhs, high_rhs) = split_both(nodes, lhs, rhs);

	let high_less = LuauLessThan::add_into(nodes, high_lhs, high_rhs);
	let high_equal = LuauEqual::add_into(nodes, high_lhs, high_rhs);
	let low = low_order(nodes, low_lhs, low_rhs);
	let tail = LuauAnd::add_into(nodes, high_equal, low);

	LuauOr::add_into(nodes, high_less, tail)
}

fn less_than(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, is_signed: bool) -> Link {
	order(nodes, lhs, rhs, is_signed, LuauLessThan::add_into)
}

fn less_than_equal(nodes: &mut Vec<Node>, lhs: Link, rhs: Link, is_signed: bool) -> Link {
	order(nodes, lhs, rhs, is_signed, LuauLessThanEqual::add_into)
}
