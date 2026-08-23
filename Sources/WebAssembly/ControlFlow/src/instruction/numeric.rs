use ir_graph::operation::{ExtendType, integer, number};

/// A 32-bit integer constant assignment.
#[derive(Clone, Copy, Debug)]
pub struct I32Constant {
	/// The destination register.
	pub destination: u16,
	/// The constant value.
	pub data: i32,
}

/// A 64-bit integer constant assignment.
#[derive(Clone, Copy, Debug)]
pub struct I64Constant {
	/// The destination register.
	pub destination: u16,
	/// The constant value.
	pub data: i64,
}

/// A 32-bit float constant assignment.
#[derive(Clone, Copy, Debug)]
pub struct F32Constant {
	/// The destination register.
	pub destination: u16,
	/// The constant value.
	pub data: f32,
}

/// A 64-bit float constant assignment.
#[derive(Clone, Copy, Debug)]
pub struct F64Constant {
	/// The destination register.
	pub destination: u16,
	/// The constant value.
	pub data: f64,
}

/// An integer unary operation instruction.
#[derive(Clone, Copy, Debug)]
pub struct IntegerUnaryOperation {
	/// The destination register.
	pub destination: u16,
	/// The source register.
	pub source: u16,

	/// The integer type.
	pub kind: integer::Type,
	/// The operator.
	pub operator: integer::UnaryOperator,
}

/// An integer binary operation instruction.
#[derive(Clone, Copy, Debug)]
pub struct IntegerBinaryOperation {
	/// The destination register.
	pub destination: u16,
	/// The left-hand operand register.
	pub lhs: u16,
	/// The right-hand operand register.
	pub rhs: u16,

	/// The integer type.
	pub kind: integer::Type,
	/// The operator.
	pub operator: integer::BinaryOperator,
}

/// An integer comparison instruction.
#[derive(Clone, Copy, Debug)]
pub struct IntegerCompareOperation {
	/// The destination register.
	pub destination: u16,
	/// The left-hand operand register.
	pub lhs: u16,
	/// The right-hand operand register.
	pub rhs: u16,

	/// The integer type.
	pub kind: integer::Type,
	/// The comparison operator.
	pub operator: integer::CompareOperator,
}

/// An integer narrowing instruction from 64-bit to 32-bit.
#[derive(Clone, Copy, Debug)]
pub struct IntegerNarrow {
	/// The destination register.
	pub destination: u16,
	/// The source register.
	pub source: u16,
}

/// An integer widening instruction from 32-bit to 64-bit.
#[derive(Clone, Copy, Debug)]
pub struct IntegerWiden {
	/// The destination register.
	pub destination: u16,
	/// The source register.
	pub source: u16,
}

/// An integer sign-extension instruction.
#[derive(Clone, Copy, Debug)]
pub struct IntegerExtend {
	/// The destination register.
	pub destination: u16,
	/// The source register.
	pub source: u16,

	/// The extension type pair.
	pub kind: ExtendType,
}

/// An integer-to-floating-point conversion instruction.
#[derive(Clone, Copy, Debug)]
pub struct IntegerConvertToNumber {
	/// The destination register.
	pub destination: u16,
	/// The source register.
	pub source: u16,

	/// Whether the source integer is signed.
	pub is_signed: bool,
	/// The target floating-point type.
	pub to: number::Type,
	/// The source integer type.
	pub from: integer::Type,
}

/// An integer-to-floating-point bit reinterpretation instruction.
#[derive(Clone, Copy, Debug)]
pub struct IntegerTransmuteToNumber {
	/// The destination register.
	pub destination: u16,
	/// The source register.
	pub source: u16,

	/// The source integer type.
	pub from: integer::Type,
}

/// A floating-point unary operation instruction.
#[derive(Clone, Copy, Debug)]
pub struct NumberUnaryOperation {
	/// The destination register.
	pub destination: u16,
	/// The source register.
	pub source: u16,

	/// The floating-point type.
	pub kind: number::Type,
	/// The operator.
	pub operator: number::UnaryOperator,
}

/// A floating-point binary operation instruction.
#[derive(Clone, Copy, Debug)]
pub struct NumberBinaryOperation {
	/// The destination register.
	pub destination: u16,
	/// The left-hand operand register.
	pub lhs: u16,
	/// The right-hand operand register.
	pub rhs: u16,

	/// The floating-point type.
	pub kind: number::Type,
	/// The operator.
	pub operator: number::BinaryOperator,
}

/// A floating-point comparison instruction.
#[derive(Clone, Copy, Debug)]
pub struct NumberCompareOperation {
	/// The destination register.
	pub destination: u16,
	/// The left-hand operand register.
	pub lhs: u16,
	/// The right-hand operand register.
	pub rhs: u16,

	/// The floating-point type.
	pub kind: number::Type,
	/// The comparison operator.
	pub operator: number::CompareOperator,
}

/// A floating-point-to-integer truncation instruction.
#[derive(Clone, Copy, Debug)]
pub struct NumberTruncateToInteger {
	/// The destination register.
	pub destination: u16,
	/// The source register.
	pub source: u16,

	/// Whether the target integer is signed.
	pub is_signed: bool,
	/// Whether the conversion saturates instead of trapping.
	pub is_saturating: bool,
	/// The target integer type.
	pub to: integer::Type,
	/// The source floating-point type.
	pub from: number::Type,
}

/// A floating-point-to-integer bit reinterpretation instruction.
#[derive(Clone, Copy, Debug)]
pub struct NumberTransmuteToInteger {
	/// The destination register.
	pub destination: u16,
	/// The source register.
	pub source: u16,

	/// The source floating-point type.
	pub from: number::Type,
}

/// A floating-point narrowing instruction from 64-bit to 32-bit.
#[derive(Clone, Copy, Debug)]
pub struct NumberNarrow {
	/// The destination register.
	pub destination: u16,
	/// The source register.
	pub source: u16,
}

/// A floating-point widening instruction from 32-bit to 64-bit.
#[derive(Clone, Copy, Debug)]
pub struct NumberWiden {
	/// The destination register.
	pub destination: u16,
	/// The source register.
	pub source: u16,
}
