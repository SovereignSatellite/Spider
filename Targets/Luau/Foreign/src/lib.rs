//! Primitive Luau foreign operation nodes.
//!
//! Each operation maps onto a single Luau host primitive (a `bit32`/`math`
//! library call, a raw operator, or a runtime transmute helper). Target
//! lowerings construct trees of these nodes; the Luau builder emits each one
//! inline instead of as a runtime call.

#![no_std]

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};

use ir_graph::{Link, Node, foreign::Foreign};

#[macro_use]
mod macros;

define_binary_operation!(Bit32And, "Bit32.And");
define_binary_operation!(Bit32Or, "Bit32.Or");
define_binary_operation!(Bit32Xor, "Bit32.Xor");
define_binary_operation!(Bit32LShift, "Bit32.LShift");
define_binary_operation!(Bit32RShift, "Bit32.RShift");
define_binary_operation!(Bit32ArShift, "Bit32.ArShift");
define_binary_operation!(Bit32LRotate, "Bit32.LRotate");
define_binary_operation!(Bit32RRotate, "Bit32.RRotate");

define_unary_operation!(Bit32CountLz, "Bit32.CountLz");
define_unary_operation!(Bit32CountRz, "Bit32.CountRz");

define_unary_operation!(MathAbs, "Math.Abs");
define_unary_operation!(MathSqrt, "Math.Sqrt");
define_unary_operation!(MathCeil, "Math.Ceil");
define_unary_operation!(MathFloor, "Math.Floor");
define_unary_operation!(MathModf, "Math.Modf");

define_binary_operation!(MathMin, "Math.Min");
define_binary_operation!(MathMax, "Math.Max");
define_binary_operation!(MathFmod, "Math.Fmod");

define_binary_operation!(LuauAdd, "Luau.Add");
define_binary_operation!(LuauSubtract, "Luau.Subtract");
define_binary_operation!(LuauMultiply, "Luau.Multiply");
define_binary_operation!(LuauDivide, "Luau.Divide");
define_binary_operation!(LuauFloorDivide, "Luau.FloorDivide");
define_binary_operation!(LuauModulo, "Luau.Modulo");

define_unary_operation!(LuauNegate, "Luau.Negate");

define_binary_operation!(LuauEqual, "Luau.Equal");
define_binary_operation!(LuauNotEqual, "Luau.NotEqual");
define_binary_operation!(LuauLessThan, "Luau.LessThan");
define_binary_operation!(LuauLessThanEqual, "Luau.LessThanEqual");

define_unary_operation!(BooleanToInteger, "Luau.BooleanToInteger");
define_unary_operation!(FlipMostSignificant, "Luau.FlipMostSignificant");
define_unary_operation!(FromBitsF32, "Transmute.FromBitsF32");
define_unary_operation!(IntoBitsF32, "Transmute.IntoBitsF32");
define_unary_operation!(IsPositive, "Transmute.IsPositive");

define_binary_operation!(IntoBitsI64, "Luau.IntoBitsI64");

define_unary_operation!(TableLength, "Table.Length");

define_unary_operation!(VectorCreate, "Vector.Create");
define_unary_operation!(VectorX, "Vector.X");

/// The `from_bits_i64` Luau foreign operation, splitting a packed `i64` value
/// into its low and high 32-bit words.
#[derive(Clone, Copy)]
pub struct FromBitsI64 {
	/// The packed source operand.
	pub source: Link,
}

impl FromBitsI64 {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The low-word result port index.
	pub const LOW_PORT: u16 = 0;
	/// The high-word result port index.
	pub const HIGH_PORT: u16 = 1;

	/// Adds the operation to the graph and returns its low- and high-word links.
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> (Link, Link) {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};

		nodes.push(Node::Foreign(Box::new(Self { source })));

		(Link(id, Self::LOW_PORT), Link(id, Self::HIGH_PORT))
	}
}

impl Foreign for FromBitsI64 {
	fn identifier(&self) -> &'static str {
		"Luau.FromBitsI64"
	}

	fn result_count(&self) -> u16 {
		Self::RESULT_COUNT
	}

	fn for_each_outer(&self, handler: &mut dyn FnMut(Link)) {
		handler(self.source);
	}

	fn for_each_mut_outer(&mut self, handler: &mut dyn FnMut(&mut Link)) {
		handler(&mut self.source);
	}
}

/// A `buffer.read*` foreign operation, reading one named-width value from a
/// buffer at a byte offset.
#[derive(Clone, Copy)]
pub struct BufferLoad {
	/// The runtime read primitive's name.
	pub name: &'static str,
	/// The source buffer.
	pub buffer: Link,
	/// The byte offset.
	pub offset: Link,
}

impl BufferLoad {
	/// Adds the operation to the graph and returns its result link.
	pub fn add_into(nodes: &mut Vec<Node>, name: &'static str, buffer: Link, offset: Link) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};

		nodes.push(Node::Foreign(Box::new(Self {
			name,
			buffer,
			offset,
		})));

		Link(id, 0)
	}
}

impl Foreign for BufferLoad {
	fn identifier(&self) -> &'static str {
		self.name
	}

	fn result_count(&self) -> u16 {
		1
	}

	fn for_each_outer(&self, handler: &mut dyn FnMut(Link)) {
		handler(self.buffer);
		handler(self.offset);
	}

	fn for_each_mut_outer(&mut self, handler: &mut dyn FnMut(&mut Link)) {
		handler(&mut self.buffer);
		handler(&mut self.offset);
	}
}

/// A `buffer.write*` foreign operation, writing one named-width value to a
/// memory reference at a byte offset.
#[derive(Clone, Copy)]
pub struct BufferStore {
	/// The runtime write primitive's name.
	pub name: &'static str,
	/// The destination memory reference.
	pub reference: Link,
	/// The byte offset.
	pub offset: Link,
	/// The value being written.
	pub value: Link,
}

impl BufferStore {
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds the operation to the graph and returns its state token link.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		name: &'static str,
		reference: Link,
		offset: Link,
		value: Link,
	) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};

		nodes.push(Node::Foreign(Box::new(Self {
			name,
			reference,
			offset,
			value,
		})));

		Link(id, Self::STATE_PORT)
	}
}

impl Foreign for BufferStore {
	fn identifier(&self) -> &'static str {
		self.name
	}

	fn result_count(&self) -> u16 {
		1
	}

	fn for_each_outer(&self, handler: &mut dyn FnMut(Link)) {
		handler(self.reference);
		handler(self.offset);
		handler(self.value);
	}

	fn for_each_mut_outer(&mut self, handler: &mut dyn FnMut(&mut Link)) {
		handler(&mut self.reference);
		handler(&mut self.offset);
		handler(&mut self.value);
	}
}

/// A `table[offset]` foreign read, indexing a table by a dynamic offset.
#[derive(Clone, Copy)]
pub struct TableLoad {
	/// The source table.
	pub reference: Link,
	/// The element offset.
	pub offset: Link,
}

impl TableLoad {
	/// Adds the operation to the graph and returns its result link.
	pub fn add_into(nodes: &mut Vec<Node>, reference: Link, offset: Link) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};

		nodes.push(Node::Foreign(Box::new(Self { reference, offset })));

		Link(id, 0)
	}
}

impl Foreign for TableLoad {
	fn identifier(&self) -> &'static str {
		"Table.Load"
	}

	fn result_count(&self) -> u16 {
		1
	}

	fn for_each_outer(&self, handler: &mut dyn FnMut(Link)) {
		handler(self.reference);
		handler(self.offset);
	}

	fn for_each_mut_outer(&mut self, handler: &mut dyn FnMut(&mut Link)) {
		handler(&mut self.reference);
		handler(&mut self.offset);
	}
}

/// A `table[offset] = value` foreign write, indexing a table by a dynamic offset.
#[derive(Clone, Copy)]
pub struct TableStore {
	/// The destination table.
	pub reference: Link,
	/// The element offset.
	pub offset: Link,
	/// The value being stored.
	pub value: Link,
}

impl TableStore {
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds the operation to the graph and returns its state token link.
	pub fn add_into(nodes: &mut Vec<Node>, reference: Link, offset: Link, value: Link) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};

		nodes.push(Node::Foreign(Box::new(Self {
			reference,
			offset,
			value,
		})));

		Link(id, Self::STATE_PORT)
	}
}

impl Foreign for TableStore {
	fn identifier(&self) -> &'static str {
		"Table.Store"
	}

	fn result_count(&self) -> u16 {
		1
	}

	fn for_each_outer(&self, handler: &mut dyn FnMut(Link)) {
		handler(self.reference);
		handler(self.offset);
		handler(self.value);
	}

	fn for_each_mut_outer(&mut self, handler: &mut dyn FnMut(&mut Link)) {
		handler(&mut self.reference);
		handler(&mut self.offset);
		handler(&mut self.value);
	}
}
