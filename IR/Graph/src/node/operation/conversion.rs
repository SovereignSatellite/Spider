//! Width and representation conversions between integers and floats.
//!
//! Each operation does exactly one thing (narrow storage, widen storage,
//! replicate sign bit, convert between integer/float, bit-reinterpret).
//! WebAssembly's fused extension opcodes compose these primitives.

#![expect(
	unused_variables,
	reason = "macro-generated visitors may not consume every field"
)]

use crate::{Link, Node};

use super::{integer, number};

/// An integer narrowing node from 64-bit to 32-bit.
#[derive(Clone, Copy)]
pub struct IntegerNarrow {
	/// The source value.
	pub source: Link,
}

impl IntegerNarrow {
	/// Adds an integer narrowing node to the graph.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::IntegerNarrow(Self { source });

		nodes.push(node);

		Link(id, 0)
	}

	handle_sources!((source, link));
}

/// An integer widening node from 32-bit to 64-bit.
#[derive(Clone, Copy)]
pub struct IntegerWiden {
	/// The source value.
	pub source: Link,
}

impl IntegerWiden {
	/// Adds an integer widening node to the graph.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::IntegerWiden(Self { source });

		nodes.push(node);

		Link(id, 0)
	}

	handle_sources!((source, link));
}

/// Source and target type pairs for integer sign extension.
#[expect(
	non_camel_case_types,
	reason = "variants encode source/target type pairs"
)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum ExtendType {
	/// Extends a signed 8-bit value to a 32-bit integer.
	I32_S8,
	/// Extends a signed 16-bit value to a 32-bit integer.
	I32_S16,

	/// Extends a signed 8-bit value to a 64-bit integer.
	I64_S8,
	/// Extends a signed 16-bit value to a 64-bit integer.
	I64_S16,
	/// Extends a signed 32-bit value to a 64-bit integer.
	I64_S32,
}

/// An integer sign-extension node.
#[derive(Clone, Copy)]
pub struct IntegerSignExtend {
	/// The source value.
	pub source: Link,
	/// The extension type pair.
	pub kind: ExtendType,
}

impl IntegerSignExtend {
	/// Adds an integer sign-extension node to the graph.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, source: Link, kind: ExtendType) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::IntegerSignExtend(Self { source, kind });

		nodes.push(node);

		Link(id, 0)
	}

	handle_sources!((source, link), (kind, ignore));
}

/// An integer-to-floating-point conversion node.
#[derive(Clone, Copy)]
pub struct IntegerConvertToNumber {
	/// The source value.
	pub source: Link,
	/// Whether the source integer is signed.
	pub is_signed: bool,
	/// The target floating-point type.
	pub to: number::Type,
	/// The source integer type.
	pub from: integer::Type,
}

impl IntegerConvertToNumber {
	/// Adds an integer-to-floating-point conversion node to the graph.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(
		nodes: &mut Vec<Node>,
		source: Link,
		is_signed: bool,
		to: number::Type,
		from: integer::Type,
	) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::IntegerConvertToNumber(Self {
			source,
			is_signed,
			to,
			from,
		});

		nodes.push(node);

		Link(id, 0)
	}

	handle_sources!(
		(source, link),
		(is_signed, ignore),
		(to, ignore),
		(from, ignore)
	);
}

/// An integer-to-floating-point bit reinterpretation node.
#[derive(Clone, Copy)]
pub struct IntegerTransmuteToNumber {
	/// The source value.
	pub source: Link,
	/// The source integer type.
	pub from: integer::Type,
}

impl IntegerTransmuteToNumber {
	/// Adds an integer-to-floating-point reinterpretation node to the graph.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, source: Link, from: integer::Type) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::IntegerTransmuteToNumber(Self { source, from });

		nodes.push(node);

		Link(id, 0)
	}

	handle_sources!((source, link), (from, ignore));
}

/// A floating-point narrowing node from 64-bit to 32-bit.
#[derive(Clone, Copy)]
pub struct NumberNarrow {
	/// The source value.
	pub source: Link,
}

impl NumberNarrow {
	/// Adds a floating-point narrowing node to the graph.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::NumberNarrow(Self { source });

		nodes.push(node);

		Link(id, 0)
	}

	handle_sources!((source, link));
}

/// A floating-point widening node from 32-bit to 64-bit.
#[derive(Clone, Copy)]
pub struct NumberWiden {
	/// The source value.
	pub source: Link,
}

impl NumberWiden {
	/// Adds a floating-point widening node to the graph.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::NumberWiden(Self { source });

		nodes.push(node);

		Link(id, 0)
	}

	handle_sources!((source, link));
}

/// A floating-point-to-integer truncation node.
#[derive(Clone, Copy)]
pub struct NumberTruncateToInteger {
	/// The source value.
	pub source: Link,
	/// Whether the target integer is signed.
	pub is_signed: bool,
	/// True for saturating conversion; false for trapping on out-of-range.
	pub is_saturating: bool,
	/// The target integer type.
	pub to: integer::Type,
	/// The source floating-point type.
	pub from: number::Type,
}

impl NumberTruncateToInteger {
	/// Adds a floating-point-to-integer truncation node to the graph.
	#[must_use = "inserted nodes without live consumers are dead"]
	#[expect(
		clippy::too_many_arguments,
		reason = "conversion constructor requires source, signedness, saturation, and type pair"
	)]
	pub fn add_into(
		nodes: &mut Vec<Node>,
		source: Link,
		is_signed: bool,
		is_saturating: bool,
		to: integer::Type,
		from: number::Type,
	) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::NumberTruncateToInteger(Self {
			source,
			is_signed,
			is_saturating,
			to,
			from,
		});

		nodes.push(node);

		Link(id, 0)
	}

	handle_sources!(
		(source, link),
		(is_signed, ignore),
		(is_saturating, ignore),
		(to, ignore),
		(from, ignore)
	);
}

/// A floating-point-to-integer bit reinterpretation node.
#[derive(Clone, Copy)]
pub struct NumberTransmuteToInteger {
	/// The source value.
	pub source: Link,
	/// The source floating-point type.
	pub from: number::Type,
}

impl NumberTransmuteToInteger {
	/// Adds a floating-point-to-integer reinterpretation node to the graph.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, source: Link, from: number::Type) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::NumberTransmuteToInteger(Self { source, from });

		nodes.push(node);

		Link(id, 0)
	}

	handle_sources!((source, link), (from, ignore));
}
