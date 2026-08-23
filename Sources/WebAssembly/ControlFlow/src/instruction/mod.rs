//! Represent WebAssembly instructions between control-flow construction and RVSDG lowering.

pub use self::{
	control::{Call, LocalBranch, LocalSet, ReservedLocal},
	memory::{
		DataDrop, MemoryCopy, MemoryFill, MemoryGrow, MemoryInit, MemoryLoad, MemorySize,
		MemoryStore,
	},
	numeric::{
		F32Constant, F64Constant, I32Constant, I64Constant, IntegerBinaryOperation,
		IntegerCompareOperation, IntegerConvertToNumber, IntegerExtend, IntegerNarrow,
		IntegerTransmuteToNumber, IntegerUnaryOperation, IntegerWiden, NumberBinaryOperation,
		NumberCompareOperation, NumberNarrow, NumberTransmuteToInteger, NumberTruncateToInteger,
		NumberUnaryOperation, NumberWiden,
	},
	reference::{GlobalGet, GlobalSet, RefFunction, RefIsNull, RefNull, Reference, ReferenceType},
	table::{
		ElementsDrop, TableCopy, TableFill, TableGet, TableGrow, TableInit, TableSet, TableSize,
	},
};

mod control;
mod memory;
mod numeric;
mod reference;
mod table;

/// A memory or table location specified by an external resource index and an offset register.
#[derive(Clone, Copy, Debug)]
pub struct Location {
	/// The external resource index.
	pub reference: u16,
	/// The offset register.
	pub offset: u16,
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

impl Instruction {
	/// Visits each external entity reference this instruction touches.
	#[expect(
		clippy::too_many_lines,
		reason = "exhaustive match over instruction variants"
	)]
	pub fn for_each_reference<Handler: FnMut(Reference)>(self, mut handler: Handler) {
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
				handler(Reference {
					kind: ReferenceType::Function,
					id: function,
				});
			}
			Self::GlobalGet(GlobalGet { source, .. }) => {
				handler(Reference {
					kind: ReferenceType::Global,
					id: source,
				});
			}
			Self::GlobalSet(GlobalSet { destination, .. }) => {
				handler(Reference {
					kind: ReferenceType::Global,
					id: destination,
				});
			}
			Self::TableGet(TableGet { source, .. }) => {
				handler(Reference {
					kind: ReferenceType::Table,
					id: source.reference,
				});
			}
			Self::TableSet(TableSet { destination, .. })
			| Self::TableFill(TableFill { destination, .. }) => {
				handler(Reference {
					kind: ReferenceType::Table,
					id: destination.reference,
				});
			}
			Self::TableSize(TableSize { table, .. }) | Self::TableGrow(TableGrow { table, .. }) => {
				handler(Reference {
					kind: ReferenceType::Table,
					id: table,
				});
			}
			Self::TableCopy(TableCopy {
				destination,
				source,
				..
			}) => {
				handler(Reference {
					kind: ReferenceType::Table,
					id: destination.reference,
				});
				handler(Reference {
					kind: ReferenceType::Table,
					id: source.reference,
				});
			}
			Self::TableInit(TableInit {
				destination,
				source,
				..
			}) => {
				handler(Reference {
					kind: ReferenceType::Table,
					id: destination.reference,
				});
				handler(Reference {
					kind: ReferenceType::Elements,
					id: source.reference,
				});
			}
			Self::ElementsDrop(ElementsDrop { source }) => {
				handler(Reference {
					kind: ReferenceType::Elements,
					id: source,
				});
			}
			Self::MemoryLoad(MemoryLoad { source, .. }) => {
				handler(Reference {
					kind: ReferenceType::Memory,
					id: source.reference,
				});
			}
			Self::MemoryStore(MemoryStore { destination, .. })
			| Self::MemoryFill(MemoryFill { destination, .. }) => {
				handler(Reference {
					kind: ReferenceType::Memory,
					id: destination.reference,
				});
			}
			Self::MemorySize(MemorySize { memory, .. })
			| Self::MemoryGrow(MemoryGrow { memory, .. }) => {
				handler(Reference {
					kind: ReferenceType::Memory,
					id: memory,
				});
			}
			Self::MemoryCopy(MemoryCopy {
				destination,
				source,
				..
			}) => {
				handler(Reference {
					kind: ReferenceType::Memory,
					id: destination.reference,
				});
				handler(Reference {
					kind: ReferenceType::Memory,
					id: source.reference,
				});
			}
			Self::MemoryInit(MemoryInit {
				destination,
				source,
				..
			}) => {
				handler(Reference {
					kind: ReferenceType::Memory,
					id: destination.reference,
				});
				handler(Reference {
					kind: ReferenceType::Data,
					id: source.reference,
				});
			}
			Self::DataDrop(DataDrop { source }) => {
				handler(Reference {
					kind: ReferenceType::Data,
					id: source,
				});
			}
		}
	}
}
