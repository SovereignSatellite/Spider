//! The `Node` enum and its crate-wide dispatch.

use alloc::sync::Arc;

use parking_lot::Mutex;

#[macro_use]
mod macros;

pub mod foreign;
pub mod operation;
pub mod region;

use self::{
	foreign::Foreign,
	operation::{
		Apply, Fence, Identity, IntegerConvertToNumber, IntegerNarrow, IntegerSignExtend,
		IntegerTransmuteToNumber, IntegerWiden, MemoryCopy, MemoryDrop, MemoryFill, MemoryGrow,
		MemoryLoad, MemoryNew, MemorySize, MemoryStore, MutableGet, MutableNew, MutableSet,
		NumberNarrow, NumberTransmuteToInteger, NumberTruncateToInteger, NumberWiden, RefIsNull,
		TableCopy, TableDrop, TableFill, TableGet, TableGrow, TableNew, TableSet, TableSize,
		integer, number,
	},
	region::{Function, Match, Repeat, branch, function, module, repeat},
};

pub use self::region::Region;

use crate::Link;

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
	ModuleArguments(module::Arguments),
	/// The boundary results of a module region.
	ModuleResults(module::Results),
	/// The boundary captures of a function region.
	FunctionCaptures(function::Captures),
	/// The boundary arguments of a function region.
	FunctionArguments(function::Arguments),
	/// The boundary results of a function region.
	FunctionResults(function::Results),
	/// The boundary arguments of a branch region.
	BranchArguments(branch::Arguments),
	/// The boundary results of a branch region.
	BranchResults(branch::Results),
	/// The boundary arguments of a repeat region.
	RepeatArguments(repeat::Arguments),
	/// The boundary results of a repeat region.
	RepeatResults(repeat::Results),

	/// An operation outside the core computation universe.
	Foreign(Box<dyn Foreign>),

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
	IntegerUnaryOperation(integer::UnaryOperation),
	/// An integer binary operation.
	IntegerBinaryOperation(integer::BinaryOperation),
	/// An integer comparison.
	IntegerCompareOperation(integer::CompareOperation),
	/// An integer narrowing from 64-bit to 32-bit.
	IntegerNarrow(IntegerNarrow),
	/// An integer widening from 32-bit to 64-bit.
	IntegerWiden(IntegerWiden),
	/// An integer sign extension.
	IntegerSignExtend(IntegerSignExtend),
	/// An integer-to-floating-point conversion.
	IntegerConvertToNumber(IntegerConvertToNumber),
	/// An integer-to-floating-point bit reinterpretation.
	IntegerTransmuteToNumber(IntegerTransmuteToNumber),

	/// A floating-point unary operation.
	NumberUnaryOperation(number::UnaryOperation),
	/// A floating-point binary operation.
	NumberBinaryOperation(number::BinaryOperation),
	/// A floating-point comparison.
	NumberCompareOperation(number::CompareOperation),
	/// A floating-point narrowing from 64-bit to 32-bit.
	NumberNarrow(NumberNarrow),
	/// A floating-point widening from 32-bit to 64-bit.
	NumberWiden(NumberWiden),
	/// A floating-point-to-integer truncation.
	NumberTruncateToInteger(NumberTruncateToInteger),
	/// A floating-point-to-integer bit reinterpretation.
	NumberTransmuteToInteger(NumberTransmuteToInteger),

	/// A mutable-cell creation.
	MutableNew(MutableNew),
	/// A mutable-cell read.
	MutableGet(MutableGet),
	/// A mutable-cell write.
	MutableSet(MutableSet),

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

macro_rules! for_each_visit {
	($self:ident, $visit:ident, $handler:ident) => {
		match $self {
			Self::Function(arc) => arc.lock().$visit($handler),
			Self::Match(arc) => arc.lock().$visit($handler),
			Self::Repeat(arc) => arc.lock().$visit($handler),

			Self::ModuleArguments(_)
			| Self::FunctionCaptures(_)
			| Self::FunctionArguments(_)
			| Self::BranchArguments(_)
			| Self::RepeatArguments(_)
			| Self::Trap
			| Self::Null
			| Self::I32(_)
			| Self::I64(_)
			| Self::F32(_)
			| Self::F64(_) => {}

			Self::ModuleResults(node) => node.$visit($handler),
			Self::FunctionResults(node) => node.$visit($handler),
			Self::BranchResults(node) => node.$visit($handler),
			Self::RepeatResults(node) => node.$visit($handler),

			Self::Foreign(foreign) => foreign.$visit(&mut $handler),

			Self::Identity(node) => node.$visit($handler),
			Self::Fence(node) => node.$visit($handler),
			Self::Apply(node) => node.$visit($handler),
			Self::RefIsNull(node) => node.$visit($handler),
			Self::IntegerUnaryOperation(node) => node.$visit($handler),
			Self::IntegerBinaryOperation(node) => node.$visit($handler),
			Self::IntegerCompareOperation(node) => node.$visit($handler),
			Self::IntegerNarrow(node) => node.$visit($handler),
			Self::IntegerWiden(node) => node.$visit($handler),
			Self::IntegerSignExtend(node) => node.$visit($handler),
			Self::IntegerConvertToNumber(node) => node.$visit($handler),
			Self::IntegerTransmuteToNumber(node) => node.$visit($handler),
			Self::NumberUnaryOperation(node) => node.$visit($handler),
			Self::NumberBinaryOperation(node) => node.$visit($handler),
			Self::NumberCompareOperation(node) => node.$visit($handler),
			Self::NumberNarrow(node) => node.$visit($handler),
			Self::NumberWiden(node) => node.$visit($handler),
			Self::NumberTruncateToInteger(node) => node.$visit($handler),
			Self::NumberTransmuteToInteger(node) => node.$visit($handler),
			Self::MutableNew(node) => node.$visit($handler),
			Self::MutableGet(node) => node.$visit($handler),
			Self::MutableSet(node) => node.$visit($handler),
			Self::TableNew(node) => node.$visit($handler),
			Self::TableGet(node) => node.$visit($handler),
			Self::TableSet(node) => node.$visit($handler),
			Self::TableSize(node) => node.$visit($handler),
			Self::TableGrow(node) => node.$visit($handler),
			Self::TableFill(node) => node.$visit($handler),
			Self::TableCopy(node) => node.$visit($handler),
			Self::TableDrop(node) => node.$visit($handler),
			Self::MemoryNew(node) => node.$visit($handler),
			Self::MemoryLoad(node) => node.$visit($handler),
			Self::MemoryStore(node) => node.$visit($handler),
			Self::MemorySize(node) => node.$visit($handler),
			Self::MemoryGrow(node) => node.$visit($handler),
			Self::MemoryFill(node) => node.$visit($handler),
			Self::MemoryCopy(node) => node.$visit($handler),
			Self::MemoryDrop(node) => node.$visit($handler),
		}
	};
}

impl Node {
	/// Adds a trap node to the graph.
	pub fn add_trap_into(nodes: &mut Vec<Self>) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());

		nodes.push(Self::Trap);

		Link(id, 0)
	}

	/// Adds a null reference constant node to the graph.
	pub fn add_null_into(nodes: &mut Vec<Self>) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());

		nodes.push(Self::Null);

		Link(id, 0)
	}

	/// Adds a 32-bit integer constant node to the graph.
	pub fn add_i32_into(nodes: &mut Vec<Self>, source: i32) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Self::I32(source);

		nodes.push(node);

		Link(id, 0)
	}

	/// Adds a 64-bit integer constant node to the graph.
	pub fn add_i64_into(nodes: &mut Vec<Self>, source: i64) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Self::I64(source);

		nodes.push(node);

		Link(id, 0)
	}

	/// Adds a 32-bit float constant node to the graph.
	pub fn add_f32_into(nodes: &mut Vec<Self>, source: f32) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Self::F32(source);

		nodes.push(node);

		Link(id, 0)
	}

	/// Adds a 64-bit float constant node to the graph.
	pub fn add_f64_into(nodes: &mut Vec<Self>, source: f64) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Self::F64(source);

		nodes.push(node);

		Link(id, 0)
	}

	/// Returns the number of output ports for this node.
	#[must_use]
	#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
	pub fn result_count(&self) -> u16 {
		match self {
			Self::Function(_)
			| Self::Trap
			| Self::Null
			| Self::I32(_)
			| Self::I64(_)
			| Self::F32(_)
			| Self::F64(_)
			| Self::RefIsNull(_)
			| Self::IntegerUnaryOperation(_)
			| Self::IntegerBinaryOperation(_)
			| Self::IntegerCompareOperation(_)
			| Self::IntegerNarrow(_)
			| Self::IntegerWiden(_)
			| Self::IntegerSignExtend(_)
			| Self::IntegerConvertToNumber(_)
			| Self::IntegerTransmuteToNumber(_)
			| Self::NumberUnaryOperation(_)
			| Self::NumberBinaryOperation(_)
			| Self::NumberCompareOperation(_)
			| Self::NumberNarrow(_)
			| Self::NumberWiden(_)
			| Self::NumberTruncateToInteger(_)
			| Self::NumberTransmuteToInteger(_)
			| Self::MutableNew(_)
			| Self::TableNew(_)
			| Self::MemoryNew(_) => 1,

			Self::Match(arc) => arc.lock().result_count(),
			Self::Repeat(arc) => arc.lock().result_count(),

			Self::ModuleArguments(_) => module::Arguments::RESULT_COUNT,

			Self::ModuleResults(_)
			| Self::FunctionResults(_)
			| Self::BranchResults(_)
			| Self::RepeatResults(_) => 0,

			Self::FunctionCaptures(node) => node.result_count(),
			Self::FunctionArguments(node) => node.result_count(),

			Self::BranchArguments(node) => node.result_count(),

			Self::RepeatArguments(node) => node.result_count(),

			Self::Foreign(foreign) => foreign.result_count(),

			Self::Identity(node) => node.result_count(),
			Self::Fence(node) => node.result_count(),
			Self::Apply(node) => node.result_count(),

			Self::MutableGet(_) => MutableGet::RESULT_COUNT,
			Self::MutableSet(_) => MutableSet::RESULT_COUNT,

			Self::TableGet(_) => TableGet::RESULT_COUNT,
			Self::TableSet(_) => TableSet::RESULT_COUNT,
			Self::TableSize(_) => TableSize::RESULT_COUNT,
			Self::TableGrow(_) => TableGrow::RESULT_COUNT,
			Self::TableFill(_) => TableFill::RESULT_COUNT,
			Self::TableCopy(_) => TableCopy::RESULT_COUNT,
			Self::TableDrop(_) => TableDrop::RESULT_COUNT,

			Self::MemoryLoad(_) => MemoryLoad::RESULT_COUNT,
			Self::MemoryStore(_) => MemoryStore::RESULT_COUNT,
			Self::MemorySize(_) => MemorySize::RESULT_COUNT,
			Self::MemoryGrow(_) => MemoryGrow::RESULT_COUNT,
			Self::MemoryFill(_) => MemoryFill::RESULT_COUNT,
			Self::MemoryCopy(_) => MemoryCopy::RESULT_COUNT,
			Self::MemoryDrop(_) => MemoryDrop::RESULT_COUNT,
		}
	}

	/// Visits each outer link (arguments from the parent region's perspective).
	pub fn for_each_outer<H: FnMut(Link)>(&self, mut handler: H) {
		for_each_visit!(self, for_each_outer, handler);
	}

	/// Mutably visits each outer link (arguments from the parent region's perspective).
	pub fn for_each_mut_outer<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		for_each_visit!(self, for_each_mut_outer, handler);
	}
}
