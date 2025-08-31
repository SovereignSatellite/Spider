use alloc::{boxed::Box, sync::Arc, vec::Vec};

pub use data_flow_graph::mvp::{
	DataNew, ExtendType, IntegerBinaryOperator, IntegerCompareOperator, IntegerType,
	IntegerUnaryOperator, LoadType, MemoryNew, NumberBinaryOperator, NumberCompareOperator,
	NumberType, NumberUnaryOperator,
};

use crate::statement::{FastDefine, Sequence};

pub struct Function {
	pub arguments: Vec<Name>,
	pub code: Sequence,
	pub returns: Vec<Local>,
}

pub struct Scoped {
	pub locals: Vec<FastDefine>,
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
	Slow { table: Name, index: u16 },
}

pub struct Call {
	pub function: Expression,
	pub arguments: Vec<Expression>,
}

pub struct RefIsNull {
	pub source: Expression,
}

pub struct IntegerUnaryOperation {
	pub source: Expression,
	pub r#type: IntegerType,
	pub operator: IntegerUnaryOperator,
}

pub struct IntegerBinaryOperation {
	pub lhs: Expression,
	pub rhs: Expression,
	pub r#type: IntegerType,
	pub operator: IntegerBinaryOperator,
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

pub struct NumberTransmuteToInteger {
	pub source: Expression,
	pub from: NumberType,
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
	pub initializer: Expression,
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

pub struct ElementsNew {
	pub content: Vec<Expression>,
}

pub struct MemoryLoad {
	pub source: Location,
	pub r#type: LoadType,
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

	ElementsNew(Box<ElementsNew>),

	MemoryNew(MemoryNew),
	MemoryLoad(Box<MemoryLoad>),
	MemorySize(Box<MemorySize>),
	MemoryGrow(Box<MemoryGrow>),

	DataNew(DataNew),
}
