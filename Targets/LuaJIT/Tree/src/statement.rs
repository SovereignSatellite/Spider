//! Statement types for the `LuaJIT` tree.

use alloc::{boxed::Box, sync::Arc, vec::Vec};

use super::expression::{Expression, Local, Location};

pub use ir_graph::simple::StoreType;

/// A sequence of statements.
pub struct Sequence {
	/// The statement list.
	pub list: Vec<Statement>,
}

/// A conditional match statement.
pub struct Match {
	/// The branch sequences.
	pub branches: Vec<Sequence>,
	/// The condition expression.
	pub condition: Expression,
}

/// A repeat loop.
pub struct Repeat {
	/// The loop body.
	pub code: Sequence,
	/// The loop condition expression.
	pub condition: Expression,
}

/// A local variable assignment.
pub struct Assign {
	/// The destination local.
	pub destination: Local,
	/// The source expression.
	pub source: Expression,
}

/// A swap-all operation on locals.
pub struct SwapAll {
	/// The locals to swap.
	pub locals: Vec<Local>,
}

/// A function call statement.
pub struct Call {
	/// The function expression.
	pub function: Expression,
	/// The result locals.
	pub results: Vec<Local>,
	/// The argument expressions.
	pub arguments: Vec<Expression>,
}

/// A global variable write.
pub struct GlobalSet {
	/// The destination expression.
	pub destination: Expression,
	/// The source expression.
	pub source: Expression,
}

/// A table element write.
pub struct TableSet {
	/// The destination location.
	pub destination: Location,
	/// The source expression.
	pub source: Expression,
}

/// A table fill operation.
pub struct TableFill {
	/// The destination location.
	pub destination: Location,
	/// The source expression.
	pub source: Expression,
	/// The size expression.
	pub size: Expression,
}

/// A table copy operation.
pub struct TableCopy {
	/// The destination location.
	pub destination: Location,
	/// The source location.
	pub source: Location,
	/// The size expression.
	pub size: Expression,
}

/// A table drop operation.
pub struct TableDrop {
	/// The source expression.
	pub source: Expression,
}

/// A memory store operation.
pub struct MemoryStore {
	/// The destination location.
	pub destination: Location,
	/// The source expression.
	pub source: Expression,
	/// The store type.
	pub kind: StoreType,
}

/// A memory fill operation.
pub struct MemoryFill {
	/// The destination location.
	pub destination: Location,
	/// The fill byte expression.
	pub byte: Expression,
	/// The size expression.
	pub size: Expression,
}

/// A memory copy operation.
pub struct MemoryCopy {
	/// The destination location.
	pub destination: Location,
	/// The source location.
	pub source: Location,
	/// The size expression.
	pub size: Expression,
}

/// A memory drop operation.
pub struct MemoryDrop {
	/// The source expression.
	pub source: Expression,
}

/// A statement node.
pub enum Statement {
	/// A conditional match.
	Match(Box<Match>),
	/// A repeat loop.
	Repeat(Box<Repeat>),

	/// A local variable assignment.
	Assign(Box<Assign>),
	/// A swap-all operation.
	SwapAll(Box<SwapAll>),

	/// A function call.
	Call(Box<Call>),

	/// A global variable write.
	GlobalSet(Box<GlobalSet>),

	/// A table element write.
	TableSet(Box<TableSet>),
	/// A table fill.
	TableFill(Box<TableFill>),
	/// A table copy.
	TableCopy(Box<TableCopy>),
	/// A table drop.
	TableDrop(Box<TableDrop>),

	/// A memory store.
	MemoryStore(Box<MemoryStore>),
	/// A memory fill.
	MemoryFill(Box<MemoryFill>),
	/// A memory copy.
	MemoryCopy(Box<MemoryCopy>),
	/// A memory drop.
	MemoryDrop(Box<MemoryDrop>),
}

/// An export declaration.
pub struct Export {
	/// The export name.
	pub identifier: Arc<str>,
	/// The source expression.
	pub source: Expression,
}
