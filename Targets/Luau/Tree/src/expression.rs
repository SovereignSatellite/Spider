use alloc::{boxed::Box, sync::Arc, vec::Vec};

pub use ir_graph::simple::{
	ExtendType, IntegerBinaryOperator, IntegerCompareOperator, IntegerType, IntegerUnaryOperator,
	LoadType, MemoryNew, NumberBinaryOperator, NumberCompareOperator, NumberType,
	NumberUnaryOperator,
};

use crate::statement::Sequence;

pub struct Function {
	pub arguments: Vec<Name>,
	pub locals: Vec<Name>,
	pub stack: u16,
	pub code: Sequence,
	pub returns: Vec<Expression>,
}

pub struct Scoped {
	pub dependencies: Vec<(Name, Expression)>,
	pub function: Function,
}

pub struct Match {
	pub condition: Expression,
	pub branches: Vec<Expression>,
}

pub struct Import {
	pub environment: Expression,
	pub namespace: Arc<str>,
	pub identifier: Arc<str>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name {
	pub id: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Local {
	Fast { name: Name },
	Slow { offset: u16 },
}

impl Local {
	#[must_use]
	pub const fn into_name(self) -> Name {
		if let Self::Fast { name } = self {
			name
		} else {
			panic!("`Local::Fast expected`, but we got `Local::Slow`")
		}
	}
}

pub struct Call {
	pub function: Expression,
	pub arguments: Vec<Expression>,
}

pub struct BooleanToInteger {
	pub source: Expression,
}

pub struct RefIsNull {
	pub source: Expression,
}

pub struct IntegerUnaryOperation {
	pub source: Expression,
	pub r#type: IntegerType,
	pub operator: IntegerUnaryOperator,
}

impl IntegerUnaryOperation {
	const fn should_be_boolean(&self) -> bool {
		matches!(self.r#type, IntegerType::I32)
	}
}

pub struct IntegerBinaryOperation {
	pub lhs: Expression,
	pub rhs: Expression,
	pub r#type: IntegerType,
	pub operator: IntegerBinaryOperator,
}

impl IntegerBinaryOperation {
	const fn should_be_boolean(&self) -> bool {
		matches!(self.r#type, IntegerType::I32)
	}
}

pub struct IntegerCompareOperation {
	pub lhs: Expression,
	pub rhs: Expression,
	pub r#type: IntegerType,
	pub operator: IntegerCompareOperator,
}

pub struct IntegerNarrow {
	pub source: Expression,
}

pub struct IntegerWiden {
	pub source: Expression,
}

pub struct IntegerExtend {
	pub source: Expression,
	pub r#type: ExtendType,
}

impl IntegerExtend {
	const fn should_be_boolean(&self) -> bool {
		matches!(self.r#type, ExtendType::I32_S8 | ExtendType::I32_S16)
	}
}

pub struct IntegerConvertToNumber {
	pub source: Expression,
	pub signed: bool,
	pub to: NumberType,
	pub from: IntegerType,
}

pub struct IntegerTransmuteToNumber {
	pub source: Expression,
	pub from: IntegerType,
}

pub struct NumberUnaryOperation {
	pub source: Expression,
	pub r#type: NumberType,
	pub operator: NumberUnaryOperator,
}

pub struct NumberBinaryOperation {
	pub lhs: Expression,
	pub rhs: Expression,
	pub r#type: NumberType,
	pub operator: NumberBinaryOperator,
}

pub struct NumberCompareOperation {
	pub lhs: Expression,
	pub rhs: Expression,
	pub r#type: NumberType,
	pub operator: NumberCompareOperator,
}

pub struct NumberNarrow {
	pub source: Expression,
}

pub struct NumberWiden {
	pub source: Expression,
}

pub struct NumberTruncateToInteger {
	pub source: Expression,
	pub signed: bool,
	pub saturate: bool,
	pub to: IntegerType,
	pub from: NumberType,
}

impl NumberTruncateToInteger {
	const fn should_be_boolean(&self) -> bool {
		matches!(self.to, IntegerType::I32)
	}
}

pub struct NumberTransmuteToInteger {
	pub source: Expression,
	pub from: NumberType,
}

impl NumberTransmuteToInteger {
	const fn should_be_boolean(&self) -> bool {
		matches!(self.from, NumberType::F32)
	}
}

pub struct Location {
	pub reference: Expression,
	pub offset: Expression,
}

pub struct GlobalNew {
	pub initializer: Expression,
}

pub struct GlobalGet {
	pub source: Expression,
}

pub struct TableNew {
	pub initializer: Vec<(Expression, u32)>,
	pub minimum: u32,
	pub maximum: u32,
}

pub struct TableGet {
	pub source: Location,
}

pub struct TableSize {
	pub source: Expression,
}

pub struct TableGrow {
	pub destination: Expression,
	pub initializer: Expression,
	pub size: Expression,
}

pub struct MemoryLoad {
	pub source: Location,
	pub r#type: LoadType,
}

impl MemoryLoad {
	const fn should_be_boolean(&self) -> bool {
		matches!(
			self.r#type,
			LoadType::I32_S8
				| LoadType::I32_U8
				| LoadType::I32_S16
				| LoadType::I32_U16
				| LoadType::I32
		)
	}
}

pub struct MemorySize {
	pub source: Expression,
}

pub struct MemoryGrow {
	pub destination: Expression,
	pub size: Expression,
}

pub enum Expression {
	Function(Box<Function>),
	Scoped(Box<Scoped>),
	Match(Box<Match>),
	Import(Box<Import>),

	Trap,
	Null,

	Local(Local),

	I32(i32),
	I64(i64),
	F32(f32),
	F64(f64),

	Call(Box<Call>),

	BooleanToInteger(Box<BooleanToInteger>),
	RefIsNull(Box<RefIsNull>),

	IntegerUnaryOperation(Box<IntegerUnaryOperation>),
	IntegerBinaryOperation(Box<IntegerBinaryOperation>),
	IntegerCompareOperation(Box<IntegerCompareOperation>),
	IntegerNarrow(Box<IntegerNarrow>),
	IntegerWiden(Box<IntegerWiden>),
	IntegerExtend(Box<IntegerExtend>),
	IntegerConvertToNumber(Box<IntegerConvertToNumber>),
	IntegerTransmuteToNumber(Box<IntegerTransmuteToNumber>),

	NumberUnaryOperation(Box<NumberUnaryOperation>),
	NumberBinaryOperation(Box<NumberBinaryOperation>),
	NumberCompareOperation(Box<NumberCompareOperation>),
	NumberNarrow(Box<NumberNarrow>),
	NumberWiden(Box<NumberWiden>),
	NumberTruncateToInteger(Box<NumberTruncateToInteger>),
	NumberTransmuteToInteger(Box<NumberTransmuteToInteger>),

	GlobalNew(Box<GlobalNew>),
	GlobalGet(Box<GlobalGet>),

	TableNew(Box<TableNew>),
	TableGet(Box<TableGet>),
	TableSize(Box<TableSize>),
	TableGrow(Box<TableGrow>),

	MemoryNew(MemoryNew),
	MemoryLoad(Box<MemoryLoad>),
	MemorySize(Box<MemorySize>),
	MemoryGrow(Box<MemoryGrow>),
}

impl Expression {
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
			r#type: IntegerType::I32,
			operator: IntegerCompareOperator::NotEqual,
		};

		Self::IntegerCompareOperation(operation.into())
	}

	#[must_use]
	pub fn into_boolean(self) -> Self {
		match self {
			Self::Match(_)
			| Self::Local(_)
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

			_ => panic!("`Expression` integer expected, we got something else"),
		}
	}
}
