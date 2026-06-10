pub use ir_graph::operation::{ExtendType, LoadType, StoreType, integer, number};

/// Reserved local-variable slots used as branch conditions and assignment scratch.
#[derive(PartialEq, Eq, Clone, Copy)]
#[expect(
	clippy::min_ident_chars,
	reason = "single-letter variable names are the semantic domain"
)]
pub enum Name {
	/// Variable A.
	A,
	/// Variable B.
	B,
	/// Variable C.
	C,
	/// Variable D.
	D,
}

impl Name {
	/// The number of named variables.
	pub const COUNT: u16 = Self::D as u16 + 1;
}

/// A local variable assignment.
#[derive(Clone, Copy, Debug)]
pub struct LocalSet {
	/// The destination register.
	pub destination: u16,
	/// The source register.
	pub source: u16,
}

/// A conditional branch on a local variable.
#[derive(Clone, Copy, Debug)]
pub struct LocalBranch {
	/// The source register.
	pub source: u16,
}

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

/// A function call instruction.
#[derive(Clone, Copy, Debug)]
pub struct Call {
	/// The destination register range.
	pub destinations: (u16, u16),
	/// The source register range.
	pub sources: (u16, u16),
	/// The function register.
	pub function: u16,
}

/// A reference null check instruction.
#[derive(Clone, Copy, Debug)]
pub struct RefIsNull {
	/// The destination register.
	pub destination: u16,
	/// The source register.
	pub source: u16,
}

/// A null reference constant assignment.
#[derive(Clone, Copy, Debug)]
pub struct RefNull {
	/// The destination register.
	pub destination: u16,
}

/// A function reference constant assignment.
#[derive(Clone, Copy, Debug)]
pub struct RefFunction {
	/// The destination register.
	pub destination: u16,
	/// The function register.
	pub function: u16,
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
	/// True for saturating conversion; false for trapping on out-of-range.
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

/// A global variable read instruction.
#[derive(Clone, Copy, Debug)]
pub struct GlobalGet {
	/// The destination register.
	pub destination: u16,
	/// The global register.
	pub source: u16,
}

/// A global variable write instruction.
#[derive(Clone, Copy, Debug)]
pub struct GlobalSet {
	/// The global register.
	pub destination: u16,
	/// The source register.
	pub source: u16,
}

/// A memory location specified by a base register and an offset register.
#[derive(Clone, Copy, Debug)]
pub struct Location {
	/// The base register.
	pub reference: u16,
	/// The offset register.
	pub offset: u16,
}

/// A table element read instruction.
#[derive(Clone, Copy, Debug)]
pub struct TableGet {
	/// The destination register.
	pub destination: u16,
	/// The source location.
	pub source: Location,
}

/// A table element write instruction.
#[derive(Clone, Copy, Debug)]
pub struct TableSet {
	/// The destination location.
	pub destination: Location,
	/// The source register.
	pub source: u16,
}

/// A table size query instruction.
#[derive(Clone, Copy, Debug)]
pub struct TableSize {
	/// The destination register.
	pub destination: u16,
	/// The table register.
	pub table: u16,
}

/// A table grow instruction.
#[derive(Clone, Copy, Debug)]
pub struct TableGrow {
	/// The destination register.
	pub destination: u16,
	/// The table register.
	pub table: u16,
	/// The growth size register.
	pub size: u16,
	/// The initial value register.
	pub initializer: u16,
}

/// A table fill instruction.
#[derive(Clone, Copy, Debug)]
pub struct TableFill {
	/// The destination location.
	pub destination: Location,
	/// The fill value register.
	pub source: u16,
	/// The number of elements register.
	pub size: u16,
}

/// A table copy instruction.
#[derive(Clone, Copy, Debug)]
pub struct TableCopy {
	/// The destination location.
	pub destination: Location,
	/// The source location.
	pub source: Location,
	/// The number of elements register.
	pub size: u16,
}

/// A table initialization instruction.
#[derive(Clone, Copy, Debug)]
pub struct TableInit {
	/// The destination location.
	pub destination: Location,
	/// The source location.
	pub source: Location,
	/// The number of elements register.
	pub size: u16,
}

/// An element segment drop instruction.
#[derive(Clone, Copy, Debug)]
pub struct ElementsDrop {
	/// The element segment register.
	pub source: u16,
}

/// A memory load instruction.
#[derive(Clone, Copy, Debug)]
pub struct MemoryLoad {
	/// The destination register.
	pub destination: u16,
	/// The source location.
	pub source: Location,
	/// The load type.
	pub kind: LoadType,
}

/// A memory store instruction.
#[derive(Clone, Copy, Debug)]
pub struct MemoryStore {
	/// The destination location.
	pub destination: Location,
	/// The source register.
	pub source: u16,
	/// The store type.
	pub kind: StoreType,
}

/// A memory size query instruction.
#[derive(Clone, Copy, Debug)]
pub struct MemorySize {
	/// The destination register.
	pub destination: u16,
	/// The memory register.
	pub memory: u16,
}

impl MemorySize {
	/// The size of a memory page in bytes.
	pub const PAGE_SIZE: usize = 0x1_0000;
	/// The maximum number of memory pages.
	pub const PAGE_LIMIT: usize = 0xFFFF;
}

/// A memory grow instruction.
#[derive(Clone, Copy, Debug)]
pub struct MemoryGrow {
	/// The destination register.
	pub destination: u16,
	/// The memory register.
	pub memory: u16,
	/// The growth size register.
	pub size: u16,
}

/// A memory fill instruction.
#[derive(Clone, Copy, Debug)]
pub struct MemoryFill {
	/// The destination location.
	pub destination: Location,
	/// The fill byte register.
	pub byte: u16,
	/// The fill size register.
	pub size: u16,
}

/// A memory copy instruction.
#[derive(Clone, Copy, Debug)]
pub struct MemoryCopy {
	/// The destination location.
	pub destination: Location,
	/// The source location.
	pub source: Location,
	/// The copy size register.
	pub size: u16,
}

/// A memory initialization instruction.
#[derive(Clone, Copy, Debug)]
pub struct MemoryInit {
	/// The destination location.
	pub destination: Location,
	/// The source location.
	pub source: Location,
	/// The initialization size register.
	pub size: u16,
}

/// A data segment drop instruction.
#[derive(Clone, Copy, Debug)]
pub struct DataDrop {
	/// The data segment register.
	pub source: u16,
}

/// An instruction in the control flow graph.
#[derive(Clone, Copy, Debug)]
pub enum Instruction {
	/// A local variable assignment.
	LocalSet(LocalSet),
	/// A conditional branch.
	LocalBranch(LocalBranch),

	/// A 32-bit integer constant.
	I32Constant(I32Constant),
	/// A 64-bit integer constant.
	I64Constant(I64Constant),
	/// A 32-bit float constant.
	F32Constant(F32Constant),
	/// A 64-bit float constant.
	F64Constant(F64Constant),

	/// A reference null check.
	RefIsNull(RefIsNull),
	/// A null reference constant.
	RefNull(RefNull),
	/// A function reference constant.
	RefFunction(RefFunction),

	/// A function call.
	Call(Call),

	/// An unreachable trap.
	Unreachable,

	/// An integer unary operation.
	IntegerUnaryOperation(IntegerUnaryOperation),
	/// An integer binary operation.
	IntegerBinaryOperation(IntegerBinaryOperation),
	/// An integer comparison.
	IntegerCompareOperation(IntegerCompareOperation),
	/// An integer narrowing.
	IntegerNarrow(IntegerNarrow),
	/// An integer widening.
	IntegerWiden(IntegerWiden),
	/// An integer sign extension.
	IntegerExtend(IntegerExtend),
	/// An integer-to-floating-point conversion.
	IntegerConvertToNumber(IntegerConvertToNumber),
	/// An integer-to-floating-point reinterpretation.
	IntegerTransmuteToNumber(IntegerTransmuteToNumber),

	/// A floating-point unary operation.
	NumberUnaryOperation(NumberUnaryOperation),
	/// A floating-point binary operation.
	NumberBinaryOperation(NumberBinaryOperation),
	/// A floating-point comparison.
	NumberCompareOperation(NumberCompareOperation),
	/// A floating-point narrowing.
	NumberNarrow(NumberNarrow),
	/// A floating-point widening.
	NumberWiden(NumberWiden),
	/// A floating-point-to-integer truncation.
	NumberTruncateToInteger(NumberTruncateToInteger),
	/// A floating-point-to-integer reinterpretation.
	NumberTransmuteToInteger(NumberTransmuteToInteger),

	/// A global variable read.
	GlobalGet(GlobalGet),
	/// A global variable write.
	GlobalSet(GlobalSet),

	/// A table element read.
	TableGet(TableGet),
	/// A table element write.
	TableSet(TableSet),
	/// A table size query.
	TableSize(TableSize),
	/// A table grow.
	TableGrow(TableGrow),
	/// A table fill.
	TableFill(TableFill),
	/// A table copy.
	TableCopy(TableCopy),
	/// A table initialization.
	TableInit(TableInit),

	/// An element segment drop.
	ElementsDrop(ElementsDrop),

	/// A memory load.
	MemoryLoad(MemoryLoad),
	/// A memory store.
	MemoryStore(MemoryStore),
	/// A memory size query.
	MemorySize(MemorySize),
	/// A memory grow.
	MemoryGrow(MemoryGrow),
	/// A memory fill.
	MemoryFill(MemoryFill),
	/// A memory copy.
	MemoryCopy(MemoryCopy),
	/// A memory initialization.
	MemoryInit(MemoryInit),

	/// A data segment drop.
	DataDrop(DataDrop),
}

/// The type of an external reference.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ReferenceType {
	/// A function reference.
	Function,

	/// A global variable reference.
	Global,
	/// A table reference.
	Table,
	/// An element segment reference.
	Elements,
	/// A linear memory reference.
	Memory,
	/// A data segment reference.
	Data,
}

impl ReferenceType {
	/// Returns whether this reference type is mutable.
	#[must_use]
	pub const fn is_mutable(self) -> bool {
		matches!(
			self,
			Self::Global | Self::Table | Self::Elements | Self::Memory | Self::Data
		)
	}
}

/// An external reference used by an instruction.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Reference {
	/// The reference type.
	pub kind: ReferenceType,
	/// The reference index.
	pub id: u16,
}

impl Reference {
	const fn function(id: u16) -> Self {
		Self {
			kind: ReferenceType::Function,
			id,
		}
	}

	const fn global(id: u16) -> Self {
		Self {
			kind: ReferenceType::Global,
			id,
		}
	}

	const fn table(id: u16) -> Self {
		Self {
			kind: ReferenceType::Table,
			id,
		}
	}

	const fn elements(id: u16) -> Self {
		Self {
			kind: ReferenceType::Elements,
			id,
		}
	}

	const fn memory(id: u16) -> Self {
		Self {
			kind: ReferenceType::Memory,
			id,
		}
	}

	const fn data(id: u16) -> Self {
		Self {
			kind: ReferenceType::Data,
			id,
		}
	}
}

impl Instruction {
	/// Visits each external entity reference this instruction touches.
	#[expect(
		clippy::too_many_lines,
		reason = "exhaustive match over instruction variants"
	)]
	pub fn for_each_reference<H: FnMut(Reference)>(self, mut handler: H) {
		match self {
			Self::LocalSet(_)
			| Self::LocalBranch(_)
			| Self::I32Constant(_)
			| Self::I64Constant(_)
			| Self::F32Constant(_)
			| Self::F64Constant(_)
			| Self::RefIsNull(_)
			| Self::RefNull(_)
			| Self::Call(_)
			| Self::Unreachable
			| Self::IntegerUnaryOperation(_)
			| Self::IntegerBinaryOperation(_)
			| Self::IntegerCompareOperation(_)
			| Self::IntegerNarrow(_)
			| Self::IntegerWiden(_)
			| Self::IntegerExtend(_)
			| Self::IntegerConvertToNumber(_)
			| Self::IntegerTransmuteToNumber(_)
			| Self::NumberUnaryOperation(_)
			| Self::NumberBinaryOperation(_)
			| Self::NumberCompareOperation(_)
			| Self::NumberNarrow(_)
			| Self::NumberWiden(_)
			| Self::NumberTruncateToInteger(_)
			| Self::NumberTransmuteToInteger(_) => {}

			Self::RefFunction(RefFunction { function, .. }) => {
				handler(Reference::function(function));
			}
			Self::GlobalGet(GlobalGet { source, .. }) => handler(Reference::global(source)),
			Self::GlobalSet(GlobalSet { destination, .. }) => {
				handler(Reference::global(destination));
			}
			Self::TableGet(TableGet { source, .. }) => handler(Reference::table(source.reference)),
			Self::TableSet(TableSet { destination, .. })
			| Self::TableFill(TableFill { destination, .. }) => {
				handler(Reference::table(destination.reference));
			}
			Self::TableSize(TableSize { table, .. }) | Self::TableGrow(TableGrow { table, .. }) => {
				handler(Reference::table(table));
			}
			Self::TableCopy(TableCopy {
				destination,
				source,
				..
			}) => {
				handler(Reference::table(destination.reference));
				handler(Reference::table(source.reference));
			}
			Self::TableInit(TableInit {
				destination,
				source,
				..
			}) => {
				handler(Reference::table(destination.reference));
				handler(Reference::elements(source.reference));
			}
			Self::ElementsDrop(ElementsDrop { source }) => handler(Reference::elements(source)),
			Self::MemoryLoad(MemoryLoad { source, .. }) => {
				handler(Reference::memory(source.reference));
			}
			Self::MemoryStore(MemoryStore { destination, .. })
			| Self::MemoryFill(MemoryFill { destination, .. }) => {
				handler(Reference::memory(destination.reference));
			}
			Self::MemorySize(MemorySize { memory, .. })
			| Self::MemoryGrow(MemoryGrow { memory, .. }) => handler(Reference::memory(memory)),
			Self::MemoryCopy(MemoryCopy {
				destination,
				source,
				..
			}) => {
				handler(Reference::memory(destination.reference));
				handler(Reference::memory(source.reference));
			}
			Self::MemoryInit(MemoryInit {
				destination,
				source,
				..
			}) => {
				handler(Reference::memory(destination.reference));
				handler(Reference::data(source.reference));
			}
			Self::DataDrop(DataDrop { source }) => handler(Reference::data(source)),
		}
	}
}
