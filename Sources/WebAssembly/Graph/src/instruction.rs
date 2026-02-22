pub use ir_graph::simple::{
	ExtendType, IntegerBinaryOperator, IntegerCompareOperator, IntegerType, IntegerUnaryOperator,
	LoadType, NumberBinaryOperator, NumberCompareOperator, NumberType, NumberUnaryOperator,
	StoreType,
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Name {
	A,
	B,
	C,
	D,
}

impl Name {
	pub const COUNT: u16 = Self::D as u16 + 1;
}

#[derive(Clone, Copy, Debug)]
pub struct LocalSet {
	pub destination: u16,
	pub source: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct LocalBranch {
	pub source: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct I32Constant {
	pub destination: u16,
	pub data: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct I64Constant {
	pub destination: u16,
	pub data: i64,
}

#[derive(Clone, Copy, Debug)]
pub struct F32Constant {
	pub destination: u16,
	pub data: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct F64Constant {
	pub destination: u16,
	pub data: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Call {
	pub destinations: (u16, u16),
	pub sources: (u16, u16),
	pub function: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct RefIsNull {
	pub destination: u16,
	pub source: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct RefNull {
	pub destination: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct RefFunction {
	pub destination: u16,
	pub function: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct IntegerUnaryOperation {
	pub destination: u16,
	pub source: u16,

	pub r#type: IntegerType,
	pub operator: IntegerUnaryOperator,
}

#[derive(Clone, Copy, Debug)]
pub struct IntegerBinaryOperation {
	pub destination: u16,
	pub lhs: u16,
	pub rhs: u16,

	pub r#type: IntegerType,
	pub operator: IntegerBinaryOperator,
}

#[derive(Clone, Copy, Debug)]
pub struct IntegerCompareOperation {
	pub destination: u16,
	pub lhs: u16,
	pub rhs: u16,

	pub r#type: IntegerType,
	pub operator: IntegerCompareOperator,
}

#[derive(Clone, Copy, Debug)]
pub struct IntegerNarrow {
	pub destination: u16,
	pub source: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct IntegerWiden {
	pub destination: u16,
	pub source: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct IntegerExtend {
	pub destination: u16,
	pub source: u16,

	pub r#type: ExtendType,
}

#[derive(Clone, Copy, Debug)]
pub struct IntegerConvertToNumber {
	pub destination: u16,
	pub source: u16,

	pub signed: bool,
	pub to: NumberType,
	pub from: IntegerType,
}

#[derive(Clone, Copy, Debug)]
pub struct IntegerTransmuteToNumber {
	pub destination: u16,
	pub source: u16,

	pub from: IntegerType,
}

#[derive(Clone, Copy, Debug)]
pub struct NumberUnaryOperation {
	pub destination: u16,
	pub source: u16,

	pub r#type: NumberType,
	pub operator: NumberUnaryOperator,
}

#[derive(Clone, Copy, Debug)]
pub struct NumberBinaryOperation {
	pub destination: u16,
	pub lhs: u16,
	pub rhs: u16,

	pub r#type: NumberType,
	pub operator: NumberBinaryOperator,
}

#[derive(Clone, Copy, Debug)]
pub struct NumberCompareOperation {
	pub destination: u16,
	pub lhs: u16,
	pub rhs: u16,

	pub r#type: NumberType,
	pub operator: NumberCompareOperator,
}

#[derive(Clone, Copy, Debug)]
pub struct NumberTruncateToInteger {
	pub destination: u16,
	pub source: u16,

	pub signed: bool,
	pub saturate: bool,
	pub to: IntegerType,
	pub from: NumberType,
}

#[derive(Clone, Copy, Debug)]
pub struct NumberTransmuteToInteger {
	pub destination: u16,
	pub source: u16,

	pub from: NumberType,
}

#[derive(Clone, Copy, Debug)]
pub struct NumberNarrow {
	pub destination: u16,
	pub source: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct NumberWiden {
	pub destination: u16,
	pub source: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct GlobalGet {
	pub destination: u16,
	pub source: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct GlobalSet {
	pub destination: u16,
	pub source: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct Location {
	pub reference: u16,
	pub offset: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct TableGet {
	pub destination: u16,
	pub source: Location,
}

#[derive(Clone, Copy, Debug)]
pub struct TableSet {
	pub destination: Location,
	pub source: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct TableSize {
	pub destination: u16,
	pub table: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct TableGrow {
	pub destination: u16,
	pub table: u16,
	pub size: u16,
	pub initializer: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct TableFill {
	pub destination: Location,
	pub source: u16,
	pub size: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct TableCopy {
	pub destination: Location,
	pub source: Location,
	pub size: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct TableInit {
	pub destination: Location,
	pub source: Location,
	pub size: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct ElementsDrop {
	pub source: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryLoad {
	pub destination: u16,
	pub source: Location,
	pub r#type: LoadType,
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryStore {
	pub destination: Location,
	pub source: u16,
	pub r#type: StoreType,
}

#[derive(Clone, Copy, Debug)]
pub struct MemorySize {
	pub destination: u16,
	pub memory: u16,
}

impl MemorySize {
	pub const PAGE_SIZE: usize = 0x1_0000;
	pub const PAGE_LIMIT: usize = 0xFFFF;
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryGrow {
	pub destination: u16,
	pub memory: u16,
	pub size: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryFill {
	pub destination: Location,
	pub byte: u16,
	pub size: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryCopy {
	pub destination: Location,
	pub source: Location,
	pub size: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryInit {
	pub destination: Location,
	pub source: Location,
	pub size: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct DataDrop {
	pub source: u16,
}

#[derive(Clone, Copy, Debug)]
pub enum Instruction {
	LocalSet(LocalSet),
	LocalBranch(LocalBranch),

	I32Constant(I32Constant),
	I64Constant(I64Constant),
	F32Constant(F32Constant),
	F64Constant(F64Constant),

	RefIsNull(RefIsNull),
	RefNull(RefNull),
	RefFunction(RefFunction),

	Call(Call),

	Unreachable,

	IntegerUnaryOperation(IntegerUnaryOperation),
	IntegerBinaryOperation(IntegerBinaryOperation),
	IntegerCompareOperation(IntegerCompareOperation),
	IntegerNarrow(IntegerNarrow),
	IntegerWiden(IntegerWiden),
	IntegerExtend(IntegerExtend),
	IntegerConvertToNumber(IntegerConvertToNumber),
	IntegerTransmuteToNumber(IntegerTransmuteToNumber),

	NumberUnaryOperation(NumberUnaryOperation),
	NumberBinaryOperation(NumberBinaryOperation),
	NumberCompareOperation(NumberCompareOperation),
	NumberNarrow(NumberNarrow),
	NumberWiden(NumberWiden),
	NumberTruncateToInteger(NumberTruncateToInteger),
	NumberTransmuteToInteger(NumberTransmuteToInteger),

	GlobalGet(GlobalGet),
	GlobalSet(GlobalSet),

	TableGet(TableGet),
	TableSet(TableSet),
	TableSize(TableSize),
	TableGrow(TableGrow),
	TableFill(TableFill),
	TableCopy(TableCopy),
	TableInit(TableInit),

	ElementsDrop(ElementsDrop),

	MemoryLoad(MemoryLoad),
	MemoryStore(MemoryStore),
	MemorySize(MemorySize),
	MemoryGrow(MemoryGrow),
	MemoryFill(MemoryFill),
	MemoryCopy(MemoryCopy),
	MemoryInit(MemoryInit),

	DataDrop(DataDrop),
}
