//! Simple operation node types.

use alloc::{sync::Arc, vec::Vec};
use list::resizable::Resizable;

use crate::Link;

/// Trait for node types that host links and identifiers.
pub trait Host {
	/// Returns the name of this node type.
	fn identifier(&self) -> &'static str;

	/// Calls `handler` for each identifier.
	fn for_each_id(&self, handler: &mut dyn FnMut(u32)) {
		let _ = handler;
	}

	/// Calls `handler` for each mutable identifier.
	fn for_each_mut_id(&mut self, handler: &mut dyn FnMut(&mut u32)) {
		let _ = handler;
	}

	/// Calls `handler` for each argument link.
	fn for_each_argument(&self, handler: &mut dyn FnMut(Link)) {
		let _ = handler;
	}

	/// Calls `handler` for each mutable argument link.
	fn for_each_mut_argument(&mut self, handler: &mut dyn FnMut(&mut Link)) {
		let _ = handler;
	}
}

/// A node that passes through its sources unchanged.
pub struct Identity {
	/// The source links.
	pub sources: Resizable<Link, 4>,
}

/// A fence node that orders its sources.
pub struct Fence {
	/// The ordered source links.
	pub sources: Resizable<Link, 4>,
}

/// A function application node.
pub struct Apply {
	/// The link to the function being applied.
	pub function: Link,
	/// The argument links passed to the function.
	pub arguments: Vec<Link>,
	/// The number of results produced.
	pub results: u16,
}

/// A reference null check node.
#[derive(Clone, Copy)]
pub struct RefIsNull {
	/// The link to the reference being checked.
	pub source: Link,
}

/// Integer types.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum IntegerType {
	/// A 32-bit integer.
	I32,
	/// A 64-bit integer.
	I64,
}

/// Unary operators for integers.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum IntegerUnaryOperator {
	/// Population count.
	CountOnes,
	/// Count of leading zero bits.
	LeadingZeroes,
	/// Count of trailing zero bits.
	TrailingZeroes,
}

/// An integer unary operation node.
#[derive(Clone, Copy)]
pub struct IntegerUnaryOperation {
	/// The link to the source value.
	pub source: Link,
	/// The integer type of the operation.
	pub kind: IntegerType,
	/// The operator to apply.
	pub operator: IntegerUnaryOperator,
}

/// Binary operators for integer arithmetic.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum IntegerBinaryOperator {
	/// Addition.
	Add,
	/// Subtraction.
	Subtract,
	/// Multiplication.
	Multiply,
	/// Division.
	Divide {
		/// Whether the operands are treated as signed.
		signed: bool,
	},
	/// Remainder.
	Remainder {
		/// Whether the operands are treated as signed.
		signed: bool,
	},
	/// Bitwise AND.
	And,
	/// Bitwise OR.
	Or,
	/// Bitwise exclusive OR.
	ExclusiveOr,
	/// Left shift.
	ShiftLeft,
	/// Right shift.
	ShiftRight {
		/// Whether the shift is arithmetic.
		signed: bool,
	},
	/// Left rotation.
	RotateLeft,
	/// Right rotation.
	RotateRight,
}

/// An integer binary operation node.
#[derive(Clone, Copy)]
pub struct IntegerBinaryOperation {
	/// The left-hand operand.
	pub lhs: Link,
	/// The right-hand operand.
	pub rhs: Link,
	/// The integer type of the operation.
	pub kind: IntegerType,
	/// The operator to apply.
	pub operator: IntegerBinaryOperator,
}

/// Comparison operators for integers.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum IntegerCompareOperator {
	/// Equality.
	Equal,
	/// Inequality.
	NotEqual,
	/// Less than.
	LessThan {
		/// Whether the comparison is signed.
		signed: bool,
	},
	/// Greater than.
	GreaterThan {
		/// Whether the comparison is signed.
		signed: bool,
	},
	/// Less than or equal.
	LessThanEqual {
		/// Whether the comparison is signed.
		signed: bool,
	},
	/// Greater than or equal.
	GreaterThanEqual {
		/// Whether the comparison is signed.
		signed: bool,
	},
}

/// An integer comparison operation node.
#[derive(Clone, Copy)]
pub struct IntegerCompareOperation {
	/// The left-hand operand.
	pub lhs: Link,
	/// The right-hand operand.
	pub rhs: Link,
	/// The integer type of the operands.
	pub kind: IntegerType,
	/// The comparison operator.
	pub operator: IntegerCompareOperator,
}

/// An integer narrowing node from 64-bit to 32-bit.
#[derive(Clone, Copy)]
pub struct IntegerNarrow {
	/// The link to the source value.
	pub source: Link,
}

/// An integer widening node from 32-bit to 64-bit.
#[derive(Clone, Copy)]
pub struct IntegerWiden {
	/// The link to the source value.
	pub source: Link,
}

/// Source and target type pairs for integer sign extension.
#[expect(
	non_camel_case_types,
	reason = "variants encode source/target type pairs"
)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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
pub struct IntegerExtend {
	/// The link to the source value.
	pub source: Link,
	/// The extension type pair.
	pub kind: ExtendType,
}

/// An integer-to-floating-point conversion node.
#[derive(Clone, Copy)]
pub struct IntegerConvertToNumber {
	/// The link to the source value.
	pub source: Link,
	/// Whether the source integer is signed.
	pub signed: bool,
	/// The target floating-point type.
	pub to: NumberType,
	/// The source integer type.
	pub from: IntegerType,
}

/// An integer-to-floating-point bit reinterpretation node.
#[derive(Clone, Copy)]
pub struct IntegerTransmuteToNumber {
	/// The link to the source value.
	pub source: Link,
	/// The source integer type.
	pub from: IntegerType,
}

/// Floating-point types.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum NumberType {
	/// A 32-bit float.
	F32,
	/// A 64-bit float.
	F64,
}

/// Unary operators for floating-point values.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum NumberUnaryOperator {
	/// Absolute value.
	Absolute,
	/// Negation.
	Negate,
	/// Square root.
	SquareRoot,
	/// Rounds toward positive infinity.
	RoundUp,
	/// Rounds toward negative infinity.
	RoundDown,
	/// Truncates toward zero.
	Truncate,
	/// Rounds to the nearest integer.
	Nearest,
}

/// A floating-point unary operation node.
#[derive(Clone, Copy)]
pub struct NumberUnaryOperation {
	/// The link to the source value.
	pub source: Link,
	/// The floating-point type of the operation.
	pub kind: NumberType,
	/// The operator to apply.
	pub operator: NumberUnaryOperator,
}

/// Binary operators for floating-point arithmetic.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum NumberBinaryOperator {
	/// Addition.
	Add,
	/// Subtraction.
	Subtract,
	/// Multiplication.
	Multiply,
	/// Division.
	Divide,
	/// Minimum.
	Minimum,
	/// Maximum.
	Maximum,
	/// Copies the sign of one operand to the other.
	CopySign,
}

/// A floating-point binary operation node.
#[derive(Clone, Copy)]
pub struct NumberBinaryOperation {
	/// The left-hand operand.
	pub lhs: Link,
	/// The right-hand operand.
	pub rhs: Link,
	/// The floating-point type of the operation.
	pub kind: NumberType,
	/// The operator to apply.
	pub operator: NumberBinaryOperator,
}

/// Comparison operators for floating-point values.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum NumberCompareOperator {
	/// Equality.
	Equal,
	/// Inequality.
	NotEqual,
	/// Less than.
	LessThan,
	/// Greater than.
	GreaterThan,
	/// Less than or equal.
	LessThanEqual,
	/// Greater than or equal.
	GreaterThanEqual,
}

/// A floating-point comparison operation node.
#[derive(Clone, Copy)]
pub struct NumberCompareOperation {
	/// The left-hand operand.
	pub lhs: Link,
	/// The right-hand operand.
	pub rhs: Link,
	/// The floating-point type of the operands.
	pub kind: NumberType,
	/// The comparison operator.
	pub operator: NumberCompareOperator,
}

/// A floating-point-to-integer truncation node.
#[derive(Clone, Copy)]
pub struct NumberTruncateToInteger {
	/// The link to the source value.
	pub source: Link,
	/// Whether the target integer is signed.
	pub signed: bool,
	/// Whether to use saturating semantics.
	pub saturate: bool,
	/// The target integer type.
	pub to: IntegerType,
	/// The source floating-point type.
	pub from: NumberType,
}

/// A floating-point-to-integer bit reinterpretation node.
#[derive(Clone, Copy)]
pub struct NumberTransmuteToInteger {
	/// The link to the source value.
	pub source: Link,
	/// The source floating-point type.
	pub from: NumberType,
}

/// A floating-point narrowing node from 64-bit to 32-bit.
#[derive(Clone, Copy)]
pub struct NumberNarrow {
	/// The link to the source value.
	pub source: Link,
}

/// A floating-point widening node from 32-bit to 64-bit.
#[derive(Clone, Copy)]
pub struct NumberWiden {
	/// The link to the source value.
	pub source: Link,
}

/// A memory location specified by a base reference and an offset.
#[derive(Clone, Copy)]
pub struct Location {
	/// The base reference.
	pub reference: Link,
	/// The offset from the base reference.
	pub offset: Link,
}

/// A global variable creation node.
#[derive(Clone, Copy)]
pub struct GlobalNew {
	/// The link to the initial value.
	pub initializer: Link,
}

/// A global variable read node.
#[derive(Clone, Copy)]
pub struct GlobalGet {
	/// The link to the global variable.
	pub source: Link,
}

/// A global variable write node.
#[derive(Clone, Copy)]
pub struct GlobalSet {
	/// The link to the global variable.
	pub destination: Link,
	/// The link to the value being stored.
	pub source: Link,
}

/// A table creation node.
pub struct TableNew {
	/// The initial elements and their offsets.
	pub initializer: Vec<(Link, u32)>,
	/// The minimum number of elements.
	pub minimum: u32,
	/// The maximum number of elements.
	pub maximum: u32,
}

/// A table element read node.
#[derive(Clone, Copy)]
pub struct TableGet {
	/// The source location to read from.
	pub source: Location,
}

/// A table element write node.
#[derive(Clone, Copy)]
pub struct TableSet {
	/// The destination location.
	pub destination: Location,
	/// The link to the value being stored.
	pub source: Link,
}

/// A table size query node.
#[derive(Clone, Copy)]
pub struct TableSize {
	/// The link to the table being queried.
	pub source: Link,
}

/// A table grow node.
#[derive(Clone, Copy)]
pub struct TableGrow {
	/// The link to the table being grown.
	pub destination: Link,
	/// The link to the initial value for new elements.
	pub initializer: Link,
	/// The number of elements to grow by.
	pub size: Link,
}

/// A table fill node.
#[derive(Clone, Copy)]
pub struct TableFill {
	/// The destination location.
	pub destination: Location,
	/// The link to the fill value.
	pub source: Link,
	/// The number of elements to fill.
	pub size: Link,
}

/// A table copy node.
#[derive(Clone, Copy)]
pub struct TableCopy {
	/// The destination location.
	pub destination: Location,
	/// The source location.
	pub source: Location,
	/// The number of elements to copy.
	pub size: Link,
}

/// A table drop node.
#[derive(Clone, Copy)]
pub struct TableDrop {
	/// The link to the table being dropped.
	pub source: Link,
}

/// A memory creation node.
#[derive(Clone)]
pub struct MemoryNew {
	/// The initial data segments and their offsets.
	pub initializer: Vec<(Arc<[u8]>, u32)>,
	/// The minimum number of pages.
	pub minimum: u32,
	/// The maximum number of pages.
	pub maximum: u32,
}

/// Source and target type pairs for memory loads.
#[expect(
	non_camel_case_types,
	reason = "variants encode source/target type pairs"
)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum LoadType {
	/// Loads a signed 8-bit value into a 32-bit integer.
	I32_S8,
	/// Loads an unsigned 8-bit value into a 32-bit integer.
	I32_U8,
	/// Loads a signed 16-bit value into a 32-bit integer.
	I32_S16,
	/// Loads an unsigned 16-bit value into a 32-bit integer.
	I32_U16,
	/// Loads a 32-bit integer.
	I32,

	/// Loads a signed 8-bit value into a 64-bit integer.
	I64_S8,
	/// Loads an unsigned 8-bit value into a 64-bit integer.
	I64_U8,
	/// Loads a signed 16-bit value into a 64-bit integer.
	I64_S16,
	/// Loads an unsigned 16-bit value into a 64-bit integer.
	I64_U16,
	/// Loads a signed 32-bit value into a 64-bit integer.
	I64_S32,
	/// Loads an unsigned 32-bit value into a 64-bit integer.
	I64_U32,
	/// Loads a 64-bit integer.
	I64,

	/// Loads a 32-bit float.
	F32,
	/// Loads a 64-bit float.
	F64,
}

/// A memory load node.
#[derive(Clone, Copy)]
pub struct MemoryLoad {
	/// The source location to load from.
	pub source: Location,
	/// The load type.
	pub kind: LoadType,
}

/// Source and target type pairs for memory stores.
#[expect(
	non_camel_case_types,
	reason = "variants encode source/target type pairs"
)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum StoreType {
	/// Stores the low 8 bits of a 32-bit integer.
	I32_I8,
	/// Stores the low 16 bits of a 32-bit integer.
	I32_I16,
	/// Stores a 32-bit integer.
	I32,

	/// Stores the low 8 bits of a 64-bit integer.
	I64_I8,
	/// Stores the low 16 bits of a 64-bit integer.
	I64_I16,
	/// Stores the low 32 bits of a 64-bit integer.
	I64_I32,
	/// Stores a 64-bit integer.
	I64,

	/// Stores a 32-bit float.
	F32,
	/// Stores a 64-bit float.
	F64,
}

/// A memory store node.
#[derive(Clone, Copy)]
pub struct MemoryStore {
	/// The destination location.
	pub destination: Location,
	/// The link to the value being stored.
	pub source: Link,
	/// The store type.
	pub kind: StoreType,
}

/// A memory size query node.
#[derive(Clone, Copy)]
pub struct MemorySize {
	/// The link to the memory being queried.
	pub source: Link,
}

/// A memory grow node.
#[derive(Clone, Copy)]
pub struct MemoryGrow {
	/// The link to the memory being grown.
	pub destination: Link,
	/// The number of pages to grow by.
	pub size: Link,
}

/// A memory fill node.
#[derive(Clone, Copy)]
pub struct MemoryFill {
	/// The destination location.
	pub destination: Location,
	/// The byte value to fill with.
	pub byte: Link,
	/// The number of bytes to fill.
	pub size: Link,
}

/// A memory copy node.
#[derive(Clone, Copy)]
pub struct MemoryCopy {
	/// The destination location.
	pub destination: Location,
	/// The source location.
	pub source: Location,
	/// The number of bytes to copy.
	pub size: Link,
}

/// A memory drop node.
#[derive(Clone, Copy)]
pub struct MemoryDrop {
	/// The link to the memory being dropped.
	pub source: Link,
}
