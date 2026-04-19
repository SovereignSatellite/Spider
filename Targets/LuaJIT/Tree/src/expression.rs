//! Expression types for the `LuaJIT` tree.

use alloc::{boxed::Box, sync::Arc, vec::Vec};

use super::statement::Sequence;

pub use ir_graph::operation::{ExtendType, LoadType, MemoryNew, integer, number};

/// A function definition.
pub struct Function {
	/// The argument names.
	pub arguments: Vec<Name>,
	/// The local variable names.
	pub locals: Vec<Name>,
	/// The stack size.
	pub stack: u16,
	/// The function body.
	pub code: Sequence,
	/// The return expressions.
	pub returns: Vec<Expression>,
}

/// A scoped function with captured dependencies.
pub struct Scoped {
	/// The captured dependencies.
	pub dependencies: Vec<(Name, Expression)>,
	/// The function definition.
	pub function: Function,
}

/// An external import.
pub struct Import {
	/// The environment expression.
	pub environment: Expression,
	/// The import namespace.
	pub namespace: Arc<str>,
	/// The import name.
	pub identifier: Arc<str>,
}

/// A variable name identifier.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name {
	/// The name identifier.
	pub id: u32,
}

/// A local variable reference.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Local {
	/// A fast register variable.
	Fast {
		/// The variable name.
		name: Name,
	},
	/// A slow stack variable.
	Slow {
		/// The stack offset.
		offset: u16,
	},
}

impl Local {
	/// Returns the inner variable name.
	///
	/// # Panics
	///
	/// Panics if this is not a `Fast` local; if this happens, it is a bug.
	#[must_use]
	pub const fn into_name(self) -> Name {
		if let Self::Fast { name } = self {
			name
		} else {
			unreachable!()
		}
	}
}

/// A function call.
pub struct Call {
	/// The function expression.
	pub function: Expression,
	/// The argument expressions.
	pub arguments: Vec<Expression>,
}

/// A boolean-to-integer conversion.
pub struct BooleanToInteger {
	/// The source expression.
	pub source: Expression,
}

/// A reference null check.
pub struct RefIsNull {
	/// The source expression.
	pub source: Expression,
}

/// An integer unary operation.
pub struct IntegerUnaryOperation {
	/// The source expression.
	pub source: Expression,
	/// The integer type.
	pub kind: integer::Type,
	/// The operator.
	pub operator: integer::UnaryOperator,
}

impl IntegerUnaryOperation {
	const fn should_be_boolean(&self) -> bool {
		matches!(self.kind, integer::Type::I32)
	}
}

/// An integer binary operation.
pub struct IntegerBinaryOperation {
	/// The left-hand operand.
	pub lhs: Expression,
	/// The right-hand operand.
	pub rhs: Expression,
	/// The integer type.
	pub kind: integer::Type,
	/// The operator.
	pub operator: integer::BinaryOperator,
}

impl IntegerBinaryOperation {
	const fn should_be_boolean(&self) -> bool {
		matches!(self.kind, integer::Type::I32)
	}
}

/// An integer comparison.
pub struct IntegerCompareOperation {
	/// The left-hand operand.
	pub lhs: Expression,
	/// The right-hand operand.
	pub rhs: Expression,
	/// The integer type.
	pub kind: integer::Type,
	/// The comparison operator.
	pub operator: integer::CompareOperator,
}

/// An integer narrowing.
pub struct IntegerNarrow {
	/// The source expression.
	pub source: Expression,
}

/// An integer widening.
pub struct IntegerWiden {
	/// The source expression.
	pub source: Expression,
}

/// An integer sign extension.
pub struct IntegerExtend {
	/// The source expression.
	pub source: Expression,
	/// The extension type pair.
	pub kind: ExtendType,
}

impl IntegerExtend {
	const fn should_be_boolean(&self) -> bool {
		matches!(self.kind, ExtendType::I32_S8 | ExtendType::I32_S16)
	}
}

/// An integer-to-floating-point conversion.
pub struct IntegerConvertToNumber {
	/// The source expression.
	pub source: Expression,
	/// Whether the source integer is signed.
	pub signed: bool,
	/// The target floating-point type.
	pub to: number::Type,
	/// The source integer type.
	pub from: integer::Type,
}

/// An integer-to-floating-point reinterpretation.
pub struct IntegerTransmuteToNumber {
	/// The source expression.
	pub source: Expression,
	/// The source integer type.
	pub from: integer::Type,
}

/// A floating-point unary operation.
pub struct NumberUnaryOperation {
	/// The source expression.
	pub source: Expression,
	/// The floating-point type.
	pub kind: number::Type,
	/// The operator.
	pub operator: number::UnaryOperator,
}

/// A floating-point binary operation.
pub struct NumberBinaryOperation {
	/// The left-hand operand.
	pub lhs: Expression,
	/// The right-hand operand.
	pub rhs: Expression,
	/// The floating-point type.
	pub kind: number::Type,
	/// The operator.
	pub operator: number::BinaryOperator,
}

/// A floating-point comparison.
pub struct NumberCompareOperation {
	/// The left-hand operand.
	pub lhs: Expression,
	/// The right-hand operand.
	pub rhs: Expression,
	/// The floating-point type.
	pub kind: number::Type,
	/// The comparison operator.
	pub operator: number::CompareOperator,
}

/// A floating-point narrowing.
pub struct NumberNarrow {
	/// The source expression.
	pub source: Expression,
}

/// A floating-point widening.
pub struct NumberWiden {
	/// The source expression.
	pub source: Expression,
}

/// A floating-point-to-integer truncation.
pub struct NumberTruncateToInteger {
	/// The source expression.
	pub source: Expression,
	/// Whether the target integer is signed.
	pub signed: bool,
	/// Whether to use saturating semantics.
	pub saturate: bool,
	/// The target integer type.
	pub to: integer::Type,
	/// The source floating-point type.
	pub from: number::Type,
}

impl NumberTruncateToInteger {
	const fn should_be_boolean(&self) -> bool {
		matches!(self.to, integer::Type::I32)
	}
}

/// A floating-point-to-integer reinterpretation.
pub struct NumberTransmuteToInteger {
	/// The source expression.
	pub source: Expression,
	/// The source floating-point type.
	pub from: number::Type,
}

impl NumberTransmuteToInteger {
	const fn should_be_boolean(&self) -> bool {
		matches!(self.from, number::Type::F32)
	}
}

/// A memory or table location.
pub struct Location {
	/// The base reference expression.
	pub reference: Expression,
	/// The offset expression.
	pub offset: Expression,
}

/// A global variable creation.
pub struct GlobalNew {
	/// The initial value expression.
	pub initializer: Expression,
}

/// A global variable read.
pub struct GlobalGet {
	/// The source expression.
	pub source: Expression,
}

/// A table creation.
pub struct TableNew {
	/// The initial elements and their offsets.
	pub initializer: Vec<(Expression, u32)>,
	/// The minimum element count.
	pub minimum: u32,
	/// The maximum element count.
	pub maximum: u32,
}

/// A table element read.
pub struct TableGet {
	/// The source location.
	pub source: Location,
}

/// A table size query.
pub struct TableSize {
	/// The source expression.
	pub source: Expression,
}

/// A table grow.
pub struct TableGrow {
	/// The destination expression.
	pub destination: Expression,
	/// The initial value expression.
	pub initializer: Expression,
	/// The size expression.
	pub size: Expression,
}

/// A memory load.
pub struct MemoryLoad {
	/// The source location.
	pub source: Location,
	/// The load type.
	pub kind: LoadType,
}

impl MemoryLoad {
	const fn should_be_boolean(&self) -> bool {
		matches!(
			self.kind,
			LoadType::I32_S8
				| LoadType::I32_U8
				| LoadType::I32_S16
				| LoadType::I32_U16
				| LoadType::I32
		)
	}
}

/// A memory size query.
pub struct MemorySize {
	/// The source expression.
	pub source: Expression,
}

/// A memory grow.
pub struct MemoryGrow {
	/// The destination expression.
	pub destination: Expression,
	/// The size expression.
	pub size: Expression,
}

/// An expression node.
pub enum Expression {
	/// A function definition.
	Function(Box<Function>),
	/// A scoped function with captured dependencies.
	Scoped(Box<Scoped>),
	/// An external import.
	Import(Box<Import>),

	/// An unreachable trap.
	Trap,
	/// A null reference constant.
	Null,

	/// A local variable reference.
	Local(Local),

	/// A 32-bit integer constant.
	I32(i32),
	/// A 64-bit integer constant.
	I64(i64),
	/// A 32-bit float constant.
	F32(f32),
	/// A 64-bit float constant.
	F64(f64),

	/// A function call.
	Call(Box<Call>),

	/// A boolean-to-integer conversion.
	BooleanToInteger(Box<BooleanToInteger>),
	/// A reference null check.
	RefIsNull(Box<RefIsNull>),

	/// An integer unary operation.
	IntegerUnaryOperation(Box<IntegerUnaryOperation>),
	/// An integer binary operation.
	IntegerBinaryOperation(Box<IntegerBinaryOperation>),
	/// An integer comparison.
	IntegerCompareOperation(Box<IntegerCompareOperation>),
	/// An integer narrowing.
	IntegerNarrow(Box<IntegerNarrow>),
	/// An integer widening.
	IntegerWiden(Box<IntegerWiden>),
	/// An integer sign extension.
	IntegerExtend(Box<IntegerExtend>),
	/// An integer-to-floating-point conversion.
	IntegerConvertToNumber(Box<IntegerConvertToNumber>),
	/// An integer-to-floating-point reinterpretation.
	IntegerTransmuteToNumber(Box<IntegerTransmuteToNumber>),

	/// A floating-point unary operation.
	NumberUnaryOperation(Box<NumberUnaryOperation>),
	/// A floating-point binary operation.
	NumberBinaryOperation(Box<NumberBinaryOperation>),
	/// A floating-point comparison.
	NumberCompareOperation(Box<NumberCompareOperation>),
	/// A floating-point narrowing.
	NumberNarrow(Box<NumberNarrow>),
	/// A floating-point widening.
	NumberWiden(Box<NumberWiden>),
	/// A floating-point-to-integer truncation.
	NumberTruncateToInteger(Box<NumberTruncateToInteger>),
	/// A floating-point-to-integer reinterpretation.
	NumberTransmuteToInteger(Box<NumberTransmuteToInteger>),

	/// A global variable creation.
	GlobalNew(Box<GlobalNew>),
	/// A global variable read.
	GlobalGet(Box<GlobalGet>),

	/// A table creation.
	TableNew(Box<TableNew>),
	/// A table element read.
	TableGet(Box<TableGet>),
	/// A table size query.
	TableSize(Box<TableSize>),
	/// A table grow.
	TableGrow(Box<TableGrow>),

	/// A memory creation.
	MemoryNew(MemoryNew),
	/// A memory load.
	MemoryLoad(Box<MemoryLoad>),
	/// A memory size query.
	MemorySize(Box<MemorySize>),
	/// A memory grow.
	MemoryGrow(Box<MemoryGrow>),
}

impl Expression {
	/// Returns the inner local variable reference.
	///
	/// # Panics
	///
	/// Panics if this is not a `Local` expression; if this happens, it is a bug.
	#[must_use]
	pub const fn into_local(&self) -> Local {
		if let Self::Local(local) = *self {
			local
		} else {
			unreachable!()
		}
	}

	fn into_boolean_unchecked(self) -> Self {
		let operation = IntegerCompareOperation {
			lhs: self,
			rhs: Self::I32(0),
			kind: integer::Type::I32,
			operator: integer::CompareOperator::NotEqual,
		};

		Self::IntegerCompareOperation(operation.into())
	}

	/// Converts this expression into a boolean.
	///
	/// # Panics
	///
	/// Panics if the expression cannot be converted to a boolean;
	/// if this happens, it is a bug.
	#[must_use]
	#[expect(
		clippy::too_many_lines,
		reason = "exhaustive match over expression variants"
	)]
	pub fn into_boolean(self) -> Self {
		match self {
			Self::Trap
			| Self::RefIsNull(_)
			| Self::IntegerCompareOperation(_)
			| Self::NumberCompareOperation(_) => self,

			Self::Local(_)
			| Self::I32(_)
			| Self::Call(_)
			| Self::IntegerNarrow(_)
			| Self::GlobalGet(_)
			| Self::TableSize(_)
			| Self::TableGrow(_)
			| Self::MemorySize(_)
			| Self::MemoryGrow(_) => self.into_boolean_unchecked(),

			Self::BooleanToInteger(boolean_to_integer) => boolean_to_integer.source,
			Self::IntegerUnaryOperation(ref integer_unary_operation)
				if integer_unary_operation.should_be_boolean() =>
			{
				self.into_boolean_unchecked()
			}
			Self::IntegerBinaryOperation(ref integer_binary_operation)
				if integer_binary_operation.should_be_boolean() =>
			{
				self.into_boolean_unchecked()
			}
			Self::IntegerExtend(ref integer_extend) if integer_extend.should_be_boolean() => {
				self.into_boolean_unchecked()
			}
			Self::NumberTruncateToInteger(ref number_truncate_to_integer)
				if number_truncate_to_integer.should_be_boolean() =>
			{
				self.into_boolean_unchecked()
			}
			Self::NumberTransmuteToInteger(ref number_transmute_to_integer)
				if number_transmute_to_integer.should_be_boolean() =>
			{
				self.into_boolean_unchecked()
			}
			Self::MemoryLoad(ref memory_load) if memory_load.should_be_boolean() => {
				self.into_boolean_unchecked()
			}

			Self::Function(_)
			| Self::Scoped(_)
			| Self::Import(_)
			| Self::Null
			| Self::I64(_)
			| Self::F32(_)
			| Self::F64(_)
			| Self::IntegerUnaryOperation(_)
			| Self::IntegerBinaryOperation(_)
			| Self::IntegerWiden(_)
			| Self::IntegerExtend(_)
			| Self::IntegerConvertToNumber(_)
			| Self::IntegerTransmuteToNumber(_)
			| Self::NumberUnaryOperation(_)
			| Self::NumberBinaryOperation(_)
			| Self::NumberNarrow(_)
			| Self::NumberWiden(_)
			| Self::NumberTruncateToInteger(_)
			| Self::NumberTransmuteToInteger(_)
			| Self::GlobalNew(_)
			| Self::TableNew(_)
			| Self::TableGet(_)
			| Self::MemoryNew(_)
			| Self::MemoryLoad(_) => {
				unreachable!("integer `Expression` expected")
			}
		}
	}
}
