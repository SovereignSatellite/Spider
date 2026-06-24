//! Lowerings for width and representation conversions.

use ir_graph::{
	Link, Node,
	operation::{ExtendType, integer, number},
};
use luau_foreign::{
	Bit32ArShift, Bit32LShift, Bit32Xor, FromBitsF32, FromBitsI64, IntoBitsF32, IntoBitsI64,
	LuauAdd, LuauMultiply, LuauSubtract,
};

use crate::i32 as lower_i32;

const SIGN_BIT: u32 = 0x8000_0000;
const SIGN_VALUE: f64 = 2_147_483_648.0;
const WORD_MODULUS: f64 = 4_294_967_296.0;
const HIGH_WORD: u32 = 31;

/// Lowers a 64-bit-to-32-bit narrowing to the low word.
pub fn narrow_i64(nodes: &mut Vec<Node>, source: Link) -> Link {
	let (low, _) = FromBitsI64::add_into(nodes, source);

	low
}

/// Lowers a 32-bit-to-64-bit widening by zero-extending into the high word.
pub fn widen_i32(nodes: &mut Vec<Node>, source: Link) -> Link {
	let high = Node::add_i32_into(nodes, 0);

	IntoBitsI64::add_into(nodes, source, high)
}

/// Lowers a sign-extension to its target width.
pub fn sign_extend(nodes: &mut Vec<Node>, source: Link, kind: ExtendType) -> Link {
	match kind {
		ExtendType::I32_S8 => extend_word(nodes, source, 24),
		ExtendType::I32_S16 => extend_word(nodes, source, 16),
		ExtendType::I64_S8 => extend_long(nodes, source, 24),
		ExtendType::I64_S16 => extend_long(nodes, source, 16),
		ExtendType::I64_S32 => extend_long(nodes, source, 0),
	}
}

// Shifting the field up to the sign bit, then arithmetically back down, replicates
// its sign across the upper bits.
fn extend_word(nodes: &mut Vec<Node>, source: Link, shift: u32) -> Link {
	let raised = Bit32LShift::add_fast_into(nodes, source, shift);

	Bit32ArShift::add_fast_into(nodes, raised, shift)
}

fn extend_long(nodes: &mut Vec<Node>, source: Link, shift: u32) -> Link {
	let (word, _) = FromBitsI64::add_into(nodes, source);
	let low = if shift == 0 {
		word
	} else {
		extend_word(nodes, word, shift)
	};
	let high = Bit32ArShift::add_fast_into(nodes, low, HIGH_WORD);

	IntoBitsI64::add_into(nodes, low, high)
}

/// Lowers an integer-to-floating-point conversion.
pub fn convert_to_number(
	nodes: &mut Vec<Node>,
	source: Link,
	is_signed: bool,
	to: number::Type,
	from: integer::Type,
) -> Link {
	match (to, from) {
		(number::Type::F32, integer::Type::I32) => convert_i32_to_f32(nodes, source, is_signed),
		(number::Type::F32, integer::Type::I64) => convert_long_to_f32(nodes, source, is_signed),
		(number::Type::F64, integer::Type::I32) => convert_i32_to_f64(nodes, source, is_signed),
		(number::Type::F64, integer::Type::I64) => convert_long_to_f64(nodes, source, is_signed),
	}
}

/// Lowers an integer-to-floating-point reinterpretation as identity.
pub const fn transmute_to_number(source: Link, from: integer::Type) -> Link {
	match from {
		integer::Type::I32 | integer::Type::I64 => source,
	}
}

/// Lowers a floating-point-to-integer reinterpretation as identity.
pub const fn transmute_to_integer(source: Link, from: number::Type) -> Link {
	match from {
		number::Type::F32 | number::Type::F64 => source,
	}
}

/// Lowers a 32-bit-to-64-bit float widening by decoding the bit pattern.
pub fn widen_f32(nodes: &mut Vec<Node>, source: Link) -> Link {
	FromBitsF32::add_into(nodes, source)
}

/// Lowers a 64-bit-to-32-bit float narrowing by re-encoding to the bit pattern.
pub fn narrow_f64(nodes: &mut Vec<Node>, source: Link) -> Link {
	IntoBitsF32::add_into(nodes, source)
}

fn convert_i32_to_f64(nodes: &mut Vec<Node>, source: Link, is_signed: bool) -> Link {
	if is_signed {
		lower_i32::to_signed(nodes, source)
	} else {
		source
	}
}

fn convert_i32_to_f32(nodes: &mut Vec<Node>, source: Link, is_signed: bool) -> Link {
	let value = if is_signed {
		reinterpret_signed(nodes, source)
	} else {
		source
	};

	IntoBitsF32::add_into(nodes, value)
}

fn reinterpret_signed(nodes: &mut Vec<Node>, source: Link) -> Link {
	let flipped = Bit32Xor::add_fast_into(nodes, source, SIGN_BIT);
	let bias = Node::add_f64_into(nodes, SIGN_VALUE);

	LuauSubtract::add_into(nodes, flipped, bias)
}

// A 64-bit integer is its low word plus its high word scaled by 2^32; the high
// word is zero- or sign-reinterpreted like an i32. f64 round-to-nearest is
// symmetric, so this branchless form matches the runtime's negate-convert-negate
// path for negative inputs (and reproduces its double-round when narrowed to f32).
fn convert_long_to_f64(nodes: &mut Vec<Node>, source: Link, is_signed: bool) -> Link {
	let (low, high) = FromBitsI64::add_into(nodes, source);
	let high = convert_i32_to_f64(nodes, high, is_signed);
	let scale = Node::add_f64_into(nodes, WORD_MODULUS);
	let scaled = LuauMultiply::add_into(nodes, high, scale);

	LuauAdd::add_into(nodes, low, scaled)
}

fn convert_long_to_f32(nodes: &mut Vec<Node>, source: Link, is_signed: bool) -> Link {
	let value = convert_long_to_f64(nodes, source, is_signed);

	IntoBitsF32::add_into(nodes, value)
}
