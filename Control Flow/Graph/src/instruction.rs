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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum IntegerType {
	I32,
	I64,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum IntegerUnaryOperator {
	CountOnes,
	LeadingZeroes,
	TrailingZeroes,
}

#[derive(Clone, Copy, Debug)]
pub struct IntegerUnaryOperation {
	pub destination: u16,
	pub source: u16,

	pub r#type: IntegerType,
	pub operator: IntegerUnaryOperator,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum IntegerBinaryOperator {
	Add,
	Subtract,
	Multiply,
	Divide { signed: bool },
	Remainder { signed: bool },
	And,
	Or,
	ExclusiveOr,
	ShiftLeft,
	ShiftRight { signed: bool },
	RotateLeft,
	RotateRight,
}

#[derive(Clone, Copy, Debug)]
pub struct IntegerBinaryOperation {
	pub destination: u16,
	pub lhs: u16,
	pub rhs: u16,

	pub r#type: IntegerType,
	pub operator: IntegerBinaryOperator,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum IntegerCompareOperator {
	Equal,
	NotEqual,
	LessThan { signed: bool },
	GreaterThan { signed: bool },
	LessThanEqual { signed: bool },
	GreaterThanEqual { signed: bool },
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

#[expect(non_camel_case_types)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ExtendType {
	I32_S8,
	I32_S16,

	I64_S8,
	I64_S16,
	I64_S32,
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum NumberType {
	F32,
	F64,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum NumberUnaryOperator {
	Absolute,
	Negate,
	SquareRoot,
	RoundUp,
	RoundDown,
	Truncate,
	Nearest,
}

#[derive(Clone, Copy, Debug)]
pub struct NumberUnaryOperation {
	pub destination: u16,
	pub source: u16,

	pub r#type: NumberType,
	pub operator: NumberUnaryOperator,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum NumberBinaryOperator {
	Add,
	Subtract,
	Multiply,
	Divide,
	Minimum,
	Maximum,
	CopySign,
}

#[derive(Clone, Copy, Debug)]
pub struct NumberBinaryOperation {
	pub destination: u16,
	pub lhs: u16,
	pub rhs: u16,

	pub r#type: NumberType,
	pub operator: NumberBinaryOperator,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum NumberCompareOperator {
	Equal,
	NotEqual,
	LessThan,
	GreaterThan,
	LessThanEqual,
	GreaterThanEqual,
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
	pub reference: u16,
	pub destination: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct TableGrow {
	pub reference: u16,
	pub destination: u16,
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

#[expect(non_camel_case_types)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum LoadType {
	I32_S8,
	I32_U8,
	I32_S16,
	I32_U16,
	I32,

	I64_S8,
	I64_U8,
	I64_S16,
	I64_U16,
	I64_S32,
	I64_U32,
	I64,

	F32,
	F64,
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryLoad {
	pub destination: u16,
	pub source: Location,
	pub r#type: LoadType,
}

#[expect(non_camel_case_types)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum StoreType {
	I32_I8,
	I32_I16,
	I32,

	I64_I8,
	I64_I16,
	I64_I32,
	I64,

	F32,
	F64,
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryStore {
	pub destination: Location,
	pub source: u16,
	pub r#type: StoreType,
}

#[derive(Clone, Copy, Debug)]
pub struct MemorySize {
	pub reference: u16,
	pub destination: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryGrow {
	pub reference: u16,
	pub destination: u16,
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
