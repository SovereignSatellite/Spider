use alloc::sync::Arc;

use parking_lot::Mutex;

use self::{
	control::{
		BranchArguments, BranchResults, Function, FunctionArguments, FunctionCaptures,
		FunctionResults, Import, Match, ModuleArguments, ModuleResults, Repeat, RepeatArguments,
		RepeatResults,
	},
	simple::{
		Apply, Fence, GlobalGet, GlobalNew, GlobalSet, Host, Identity, IntegerBinaryOperation,
		IntegerCompareOperation, IntegerConvertToNumber, IntegerExtend, IntegerNarrow,
		IntegerTransmuteToNumber, IntegerUnaryOperation, IntegerWiden, MemoryCopy, MemoryDrop,
		MemoryFill, MemoryGrow, MemoryLoad, MemoryNew, MemorySize, MemoryStore,
		NumberBinaryOperation, NumberCompareOperation, NumberNarrow, NumberTransmuteToInteger,
		NumberTruncateToInteger, NumberUnaryOperation, NumberWiden, RefIsNull, TableCopy,
		TableDrop, TableFill, TableGet, TableGrow, TableNew, TableSet, TableSize,
	},
};

pub use self::control::Region;

mod sinks;
mod sources;

pub mod control;
pub mod simple;

/// A node in the data flow graph.
#[derive(Default)]
pub enum Node {
	/// A function region.
	Function(Arc<Mutex<Function>>),
	/// A match (conditional) region.
	Match(Arc<Mutex<Match>>),
	/// A repeat (loop) region.
	Repeat(Arc<Mutex<Repeat>>),

	/// The boundary arguments of a module region.
	ModuleArguments(ModuleArguments),
	/// The boundary results of a module region.
	ModuleResults(ModuleResults),
	/// The boundary captures of a function region.
	FunctionCaptures(FunctionCaptures),
	/// The boundary arguments of a function region.
	FunctionArguments(FunctionArguments),
	/// The boundary results of a function region.
	FunctionResults(FunctionResults),
	/// The boundary arguments of a branch region.
	BranchArguments(BranchArguments),
	/// The boundary results of a branch region.
	BranchResults(BranchResults),
	/// The boundary arguments of a repeat region.
	RepeatArguments(RepeatArguments),
	/// The boundary results of a repeat region.
	RepeatResults(RepeatResults),

	/// An external import.
	Import(Box<Import>),
	/// A host-defined operation.
	Host(Box<dyn Host>),

	/// An unreachable trap.
	#[default]
	Trap,
	/// A null reference constant.
	Null,
	/// A 32-bit integer constant.
	I32(i32),
	/// A 64-bit integer constant.
	I64(i64),
	/// A 32-bit float constant.
	F32(f32),
	/// A 64-bit float constant.
	F64(f64),

	/// An identity pass-through.
	Identity(Identity),
	/// An ordering fence.
	Fence(Fence),

	/// A function application.
	Apply(Apply),

	/// A reference null check.
	RefIsNull(RefIsNull),

	/// An integer unary operation.
	IntegerUnaryOperation(IntegerUnaryOperation),
	/// An integer binary operation.
	IntegerBinaryOperation(IntegerBinaryOperation),
	/// An integer comparison.
	IntegerCompareOperation(IntegerCompareOperation),
	/// An integer narrowing from 64-bit to 32-bit.
	IntegerNarrow(IntegerNarrow),
	/// An integer widening from 32-bit to 64-bit.
	IntegerWiden(IntegerWiden),
	/// An integer sign extension.
	IntegerExtend(IntegerExtend),
	/// An integer-to-floating-point conversion.
	IntegerConvertToNumber(IntegerConvertToNumber),
	/// An integer-to-floating-point bit reinterpretation.
	IntegerTransmuteToNumber(IntegerTransmuteToNumber),

	/// A floating-point unary operation.
	NumberUnaryOperation(NumberUnaryOperation),
	/// A floating-point binary operation.
	NumberBinaryOperation(NumberBinaryOperation),
	/// A floating-point comparison.
	NumberCompareOperation(NumberCompareOperation),
	/// A floating-point narrowing from 64-bit to 32-bit.
	NumberNarrow(NumberNarrow),
	/// A floating-point widening from 32-bit to 64-bit.
	NumberWiden(NumberWiden),
	/// A floating-point-to-integer truncation.
	NumberTruncateToInteger(NumberTruncateToInteger),
	/// A floating-point-to-integer bit reinterpretation.
	NumberTransmuteToInteger(NumberTransmuteToInteger),

	/// A global variable creation.
	GlobalNew(GlobalNew),
	/// A global variable read.
	GlobalGet(GlobalGet),
	/// A global variable write.
	GlobalSet(GlobalSet),

	/// A table creation.
	TableNew(TableNew),
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
	/// A table drop.
	TableDrop(TableDrop),

	/// A memory creation.
	MemoryNew(MemoryNew),
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
	/// A memory drop.
	MemoryDrop(MemoryDrop),
}
