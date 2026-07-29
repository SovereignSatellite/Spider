//! Models `LuaJIT` operators and `bit`/`math`/FFI calls as foreign nodes
//! that lowering composes and the builder emits inline.

#![no_std]

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};

use ir_graph::{Link, Node, foreign::Foreign};

#[macro_use]
mod macros;

define_binary_operation!(BitAnd, "Bit.And");
define_binary_operation!(BitOr, "Bit.Or");
define_binary_operation!(BitXor, "Bit.Xor");
define_binary_operation!(BitLShift, "Bit.LShift");
define_binary_operation!(BitRShift, "Bit.RShift");
define_binary_operation!(BitArShift, "Bit.ArShift");
define_binary_operation!(BitLRotate, "Bit.LRotate");
define_binary_operation!(BitRRotate, "Bit.RRotate");

define_unary_operation!(ForceI32, "Bit.ForceI32");
define_unary_operation!(ForceU32, "Bit.ForceU32");

define_unary_operation!(MathAbs, "Math.Abs");
define_unary_operation!(MathSqrt, "Math.Sqrt");
define_unary_operation!(MathCeil, "Math.Ceil");
define_unary_operation!(MathFloor, "Math.Floor");
define_unary_operation!(MathModf, "Math.Modf");

define_binary_operation!(MathMin, "Math.Min");
define_binary_operation!(MathMax, "Math.Max");
define_binary_operation!(MathFmod, "Math.Fmod");

define_binary_operation!(LuaJITAdd, "LuaJIT.Add");
define_binary_operation!(LuaJITSubtract, "LuaJIT.Subtract");
define_binary_operation!(LuaJITMultiply, "LuaJIT.Multiply");
define_binary_operation!(LuaJITDivide, "LuaJIT.Divide");
define_binary_operation!(LuaJITModulo, "LuaJIT.Modulo");

define_unary_operation!(LuaJITNegate, "LuaJIT.Negate");

define_binary_operation!(LuaJITEqual, "LuaJIT.Equal");
define_binary_operation!(LuaJITNotEqual, "LuaJIT.NotEqual");
define_binary_operation!(LuaJITLessThan, "LuaJIT.LessThan");
define_binary_operation!(LuaJITLessThanEqual, "LuaJIT.LessThanEqual");

define_unary_operation!(BooleanToInteger, "LuaJIT.BooleanToInteger");
define_unary_operation!(CastAnyPointer, "FFI.CastAnyPointer");
define_unary_operation!(CastI64, "FFI.CastI64");
define_unary_operation!(CastU64, "FFI.CastU64");
define_unary_operation!(CastU8Pointer, "FFI.CastU8Pointer");
define_unary_operation!(FromBitsF32, "FFI.FromBitsF32");
define_unary_operation!(IntoBitsF32, "FFI.IntoBitsF32");
define_unary_operation!(FromBitsF64, "FFI.FromBitsF64");
define_unary_operation!(IntoBitsF64, "FFI.IntoBitsF64");

define_unary_operation!(TableLength, "Table.Length");

define_unary_operation!(NativeSquareRootF32, "NativeF32.SquareRoot");
define_binary_operation!(NativeAddF32, "NativeF32.Add");
define_binary_operation!(NativeSubtractF32, "NativeF32.Subtract");
define_binary_operation!(NativeMultiplyF32, "NativeF32.Multiply");
define_binary_operation!(NativeDivideF32, "NativeF32.Divide");

define_unary_operation!(MemoryData, "Memory.Data");
define_unary_operation!(MemorySize, "Memory.Size");

/// A pointer field read.
#[derive(Clone, Copy)]
pub struct PointerLoad {
	/// The pointer being read.
	pub pointer: Link,
	/// The field being read.
	pub field: &'static str,
}

impl PointerLoad {
	/// Adds the operation to the graph and returns its result link.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, pointer: Link, field: &'static str) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};

		nodes.push(Node::Foreign(Box::new(Self { pointer, field })));

		Link(id, 0)
	}
}

impl Foreign for PointerLoad {
	fn identifier(&self) -> &'static str {
		"Pointer.Load"
	}

	fn result_count(&self) -> u16 {
		1
	}

	fn duplicate(&self) -> Box<dyn Foreign> {
		Box::new(*self)
	}

	fn for_each_outer(&self, handler: &mut dyn FnMut(Link)) {
		handler(self.pointer);
	}

	fn for_each_mut_outer(&mut self, handler: &mut dyn FnMut(&mut Link)) {
		handler(&mut self.pointer);
	}
}

/// A pointer field write.
#[derive(Clone, Copy)]
pub struct PointerStore {
	/// The state reference being forwarded.
	pub reference: Link,
	/// The pointer being written.
	pub pointer: Link,
	/// The field being written.
	pub field: &'static str,
	/// The value being written.
	pub value: Link,
}

impl PointerStore {
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds the operation to the graph and returns its state token link.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(
		nodes: &mut Vec<Node>,
		reference: Link,
		pointer: Link,
		field: &'static str,
		value: Link,
	) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};

		nodes.push(Node::Foreign(Box::new(Self {
			reference,
			pointer,
			field,
			value,
		})));

		Link(id, Self::STATE_PORT)
	}
}

impl Foreign for PointerStore {
	fn identifier(&self) -> &'static str {
		"Pointer.Store"
	}

	fn result_count(&self) -> u16 {
		1
	}

	fn duplicate(&self) -> Box<dyn Foreign> {
		Box::new(*self)
	}

	fn for_each_outer(&self, handler: &mut dyn FnMut(Link)) {
		handler(self.reference);
		handler(self.pointer);
		handler(self.value);
	}

	fn for_each_mut_outer(&mut self, handler: &mut dyn FnMut(&mut Link)) {
		handler(&mut self.reference);
		handler(&mut self.pointer);
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
	#[must_use = "inserted nodes without live consumers are dead"]
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

	fn duplicate(&self) -> Box<dyn Foreign> {
		Box::new(*self)
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
	#[must_use = "inserted nodes without live consumers are dead"]
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

	fn duplicate(&self) -> Box<dyn Foreign> {
		Box::new(*self)
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
