//! Expression types for the `LuaJIT` tree representation.

use alloc::{boxed::Box, sync::Arc, vec::Vec};

pub use ir_graph::simple::{
	ExtendType, IntegerBinaryOperator, IntegerCompareOperator, IntegerType, IntegerUnaryOperator,
	LoadType, MemoryNew, NumberBinaryOperator, NumberCompareOperator, NumberType,
	NumberUnaryOperator,
};

use crate::statement::Sequence;

/// A function definition expression.
pub struct Function {
	/// The argument expressions.
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

/// A scoped function expression with captured dependencies.
pub struct Scoped {
	/// The captured dependencies.
	pub dependencies: Vec<(Name, Expression)>,
	/// The function expression.
	pub function: Function,
}

/// An external import expression.
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
			panic!("`Local::Fast expected`, but we got `Local::Slow`")
		}
	}
}

/// A function call expression.
pub struct Call {
	/// The function expression.
	pub function: Expression,
	/// The argument expressions.
	pub arguments: Vec<Expression>,
}

/// A boolean-to-integer conversion expression.
pub struct BooleanToInteger {
	/// The source expression.
	pub source: Expression,
}

/// A reference null check expression.
pub struct RefIsNull {
	/// The source expression.
	pub source: Expression,
}

/// An integer unary operation expression.
pub struct IntegerUnaryOperation {
	/// The source expression.
	pub source: Expression,
	/// The integer type.
	pub kind: IntegerType,
	/// The operator.
	pub operator: IntegerUnaryOperator,
}

impl IntegerUnaryOperation {
	const fn should_be_boolean(&self) -> bool {
		matches!(self.kind, IntegerType::I32)
	}
}

/// An integer binary operation expression.
pub struct IntegerBinaryOperation {
	/// The left-hand operand.
	pub lhs: Expression,
	/// The right-hand operand.
	pub rhs: Expression,
	/// The integer type.
	pub kind: IntegerType,
	/// The operator.
	pub operator: IntegerBinaryOperator,
}

impl IntegerBinaryOperation {
	const fn should_be_boolean(&self) -> bool {
		matches!(self.kind, IntegerType::I32)
	}
}

/// An integer comparison expression.
pub struct IntegerCompareOperation {
	/// The left-hand operand.
	pub lhs: Expression,
	/// The right-hand operand.
	pub rhs: Expression,
	/// The integer type.
	pub kind: IntegerType,
	/// The operator.
	pub operator: IntegerCompareOperator,
}

/// An integer narrowing expression.
pub struct IntegerNarrow {
	/// The source expression.
	pub source: Expression,
}

/// An integer widening expression.
pub struct IntegerWiden {
	/// The source expression.
	pub source: Expression,
}

/// An integer sign-extension expression.
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

/// An integer-to-floating-point conversion expression.
pub struct IntegerConvertToNumber {
	/// The source expression.
	pub source: Expression,
	/// Whether the value is signed.
	pub signed: bool,
	/// The target type.
	pub to: NumberType,
	/// The source type.
	pub from: IntegerType,
}

/// An integer-to-floating-point reinterpretation expression.
pub struct IntegerTransmuteToNumber {
	/// The source expression.
	pub source: Expression,
	/// The source type.
	pub from: IntegerType,
}

/// A floating-point unary operation expression.
pub struct NumberUnaryOperation {
	/// The source expression.
	pub source: Expression,
	/// The floating-point type.
	pub kind: NumberType,
	/// The operator.
	pub operator: NumberUnaryOperator,
}

/// A floating-point binary operation expression.
pub struct NumberBinaryOperation {
	/// The left-hand operand.
	pub lhs: Expression,
	/// The right-hand operand.
	pub rhs: Expression,
	/// The floating-point type.
	pub kind: NumberType,
	/// The operator.
	pub operator: NumberBinaryOperator,
}

/// A floating-point comparison expression.
pub struct NumberCompareOperation {
	/// The left-hand operand.
	pub lhs: Expression,
	/// The right-hand operand.
	pub rhs: Expression,
	/// The floating-point type.
	pub kind: NumberType,
	/// The operator.
	pub operator: NumberCompareOperator,
}

/// A floating-point narrowing expression.
pub struct NumberNarrow {
	/// The source expression.
	pub source: Expression,
}

/// A floating-point widening expression.
pub struct NumberWiden {
	/// The source expression.
	pub source: Expression,
}

/// A floating-point-to-integer truncation expression.
pub struct NumberTruncateToInteger {
	/// The source expression.
	pub source: Expression,
	/// Whether the value is signed.
	pub signed: bool,
	/// Whether to use saturating semantics.
	pub saturate: bool,
	/// The target type.
	pub to: IntegerType,
	/// The source type.
	pub from: NumberType,
}

impl NumberTruncateToInteger {
	const fn should_be_boolean(&self) -> bool {
		matches!(self.to, IntegerType::I32)
	}
}

/// A floating-point-to-integer reinterpretation expression.
pub struct NumberTransmuteToInteger {
	/// The source expression.
	pub source: Expression,
	/// The source type.
	pub from: NumberType,
}

impl NumberTransmuteToInteger {
	const fn should_be_boolean(&self) -> bool {
		matches!(self.from, NumberType::F32)
	}
}

/// A memory or table location.
pub struct Location {
	/// The base reference expression.
	pub reference: Expression,
	/// The offset expression.
	pub offset: Expression,
}

/// A global variable creation expression.
pub struct GlobalNew {
	/// The initial value expression.
	pub initializer: Expression,
}

/// A global variable read expression.
pub struct GlobalGet {
	/// The source expression.
	pub source: Expression,
}

/// A table creation expression.
pub struct TableNew {
	/// The initial value expression.
	pub initializer: Vec<(Expression, u32)>,
	/// The minimum element count.
	pub minimum: u32,
	/// The maximum element count.
	pub maximum: u32,
}

/// A table element read expression.
pub struct TableGet {
	/// The source expression.
	pub source: Location,
}

/// A table size query expression.
pub struct TableSize {
	/// The source expression.
	pub source: Expression,
}

/// A table grow expression.
pub struct TableGrow {
	/// The destination expression.
	pub destination: Expression,
	/// The initial value expression.
	pub initializer: Expression,
	/// The size expression.
	pub size: Expression,
}

/// A memory load expression.
pub struct MemoryLoad {
	/// The source expression.
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

/// A memory size query expression.
pub struct MemorySize {
	/// The source expression.
	pub source: Expression,
}

/// A memory grow expression.
pub struct MemoryGrow {
	/// The destination expression.
	pub destination: Expression,
	/// The size expression.
	pub size: Expression,
}

/// An expression in the tree.
pub enum Expression {
	/// A function definition expression.
	Function(Box<Function>),
	/// A scoped function expression with captured dependencies.
	Scoped(Box<Scoped>),
	/// An external import expression.
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

	/// A function call expression.
	Call(Box<Call>),

	/// A boolean-to-integer conversion expression.
	BooleanToInteger(Box<BooleanToInteger>),
	/// A reference null check expression.
	RefIsNull(Box<RefIsNull>),

	/// An integer unary operation expression.
	IntegerUnaryOperation(Box<IntegerUnaryOperation>),
	/// An integer binary operation expression.
	IntegerBinaryOperation(Box<IntegerBinaryOperation>),
	/// An integer comparison expression.
	IntegerCompareOperation(Box<IntegerCompareOperation>),
	/// An integer narrowing expression.
	IntegerNarrow(Box<IntegerNarrow>),
	/// An integer widening expression.
	IntegerWiden(Box<IntegerWiden>),
	/// An integer sign-extension expression.
	IntegerExtend(Box<IntegerExtend>),
	/// An integer-to-floating-point conversion expression.
	IntegerConvertToNumber(Box<IntegerConvertToNumber>),
	/// An integer-to-floating-point reinterpretation expression.
	IntegerTransmuteToNumber(Box<IntegerTransmuteToNumber>),

	/// A floating-point unary operation expression.
	NumberUnaryOperation(Box<NumberUnaryOperation>),
	/// A floating-point binary operation expression.
	NumberBinaryOperation(Box<NumberBinaryOperation>),
	/// A floating-point comparison expression.
	NumberCompareOperation(Box<NumberCompareOperation>),
	/// A floating-point narrowing expression.
	NumberNarrow(Box<NumberNarrow>),
	/// A floating-point widening expression.
	NumberWiden(Box<NumberWiden>),
	/// A floating-point-to-integer truncation expression.
	NumberTruncateToInteger(Box<NumberTruncateToInteger>),
	/// A floating-point-to-integer reinterpretation expression.
	NumberTransmuteToInteger(Box<NumberTransmuteToInteger>),

	/// A global variable creation expression.
	GlobalNew(Box<GlobalNew>),
	/// A global variable read expression.
	GlobalGet(Box<GlobalGet>),

	/// A table creation expression.
	TableNew(Box<TableNew>),
	/// A table element read expression.
	TableGet(Box<TableGet>),
	/// A table size query expression.
	TableSize(Box<TableSize>),
	/// A table grow expression.
	TableGrow(Box<TableGrow>),

	/// A memory creation expression.
	MemoryNew(MemoryNew),
	/// A memory load expression.
	MemoryLoad(Box<MemoryLoad>),
	/// A memory size query expression.
	MemorySize(Box<MemorySize>),
	/// A memory grow expression.
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
			panic!("`Expression::Local` expected, we got something else")
		}
	}

	fn into_boolean_unchecked(self) -> Self {
		let operation = IntegerCompareOperation {
			lhs: self,
			rhs: Self::I32(0),
			kind: IntegerType::I32,
			operator: IntegerCompareOperator::NotEqual,
		};

		Self::IntegerCompareOperation(operation.into())
	}

	/// Converts this expression into a boolean expression.
	///
	/// # Panics
	///
	/// Panics if the expression cannot be converted to a boolean;
	/// if this happens, it is a bug.
	#[must_use]
	pub fn into_boolean(self) -> Self {
		match self {
			Self::Local(_)
			| Self::I32(_)
			| Self::Call(_)
			| Self::IntegerNarrow(_)
			| Self::GlobalGet(_)
			| Self::TableSize(_)
			| Self::TableGrow(_)
			| Self::MemorySize(_)
			| Self::MemoryGrow(_) => self.into_boolean_unchecked(),

			Self::Trap
			| Self::RefIsNull(_)
			| Self::IntegerCompareOperation(_)
			| Self::NumberCompareOperation(_) => self,

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
				panic!("`Expression` integer expected, we got something else")
			}
		}
	}
}
