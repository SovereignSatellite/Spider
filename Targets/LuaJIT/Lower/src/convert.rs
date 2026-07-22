//! Lowerings for width and representation conversions.

use ir_graph::{
	Link, Node,
	operation::{ExtendType, integer, number},
};
use luajit_foreign::{
	BitArShift, BitLShift, CastI64, CastU64, ForceI32, ForceU32, FromBitsF32, FromBitsF64,
	IntoBitsF32, IntoBitsF64,
};

/// Lowers a 64-bit-to-32-bit narrowing to the signed 32-bit carrier.
pub fn narrow_i64(nodes: &mut Vec<Node>, source: Link) -> Link {
	ForceI32::add_into(nodes, source)
}

/// Lowers a 32-bit-to-64-bit widening by zero-extending before the native cast.
pub fn widen_i32(nodes: &mut Vec<Node>, source: Link) -> Link {
	let source = ForceU32::add_into(nodes, source);

	CastI64::add_into(nodes, source)
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
	let raised = BitLShift::add_fast_into(nodes, source, shift);

	BitArShift::add_fast_into(nodes, raised, shift)
}

fn extend_long(nodes: &mut Vec<Node>, source: Link, shift: u32) -> Link {
	let word = ForceI32::add_into(nodes, source);
	let word = if shift == 0 {
		word
	} else {
		extend_word(nodes, word, shift)
	};

	CastI64::add_into(nodes, word)
}

/// Lowers an integer-to-floating-point conversion.
pub fn convert_to_number(
	nodes: &mut Vec<Node>,
	source: Link,
	is_signed: bool,
	to: number::Type,
	from: integer::Type,
) -> Link {
	let native = native_integer(nodes, source, is_signed, from);

	match to {
		number::Type::F32 => IntoBitsF32::add_into(nodes, native),
		number::Type::F64 => IntoBitsF64::add_into(nodes, native),
	}
}

fn native_integer(
	nodes: &mut Vec<Node>,
	source: Link,
	is_signed: bool,
	from: integer::Type,
) -> Link {
	match (from, is_signed) {
		(integer::Type::I32 | integer::Type::I64, true) => source,
		(integer::Type::I32, false) => ForceU32::add_into(nodes, source),
		(integer::Type::I64, false) => CastU64::add_into(nodes, source),
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

/// Lowers a 32-bit-to-64-bit float widening by decoding and re-encoding the bit pattern.
pub fn widen_f32(nodes: &mut Vec<Node>, source: Link) -> Link {
	let native = FromBitsF32::add_into(nodes, source);

	IntoBitsF64::add_into(nodes, native)
}

/// Lowers a 64-bit-to-32-bit float narrowing by decoding and re-encoding the bit pattern.
pub fn narrow_f64(nodes: &mut Vec<Node>, source: Link) -> Link {
	let native = FromBitsF64::add_into(nodes, source);

	IntoBitsF32::add_into(nodes, native)
}
