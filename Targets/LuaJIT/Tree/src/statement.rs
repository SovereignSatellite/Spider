//! Statement types for the `LuaJIT` tree.

use super::expression::{Expression, Local};

/// A sequence of statements.
pub struct Sequence {
	/// The statements in execution order.
	pub statements: Vec<Statement>,
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
	/// The continuation condition; the loop exits when it evaluates to zero.
	pub condition: Expression,
	/// The carried-value rotation, run after the body on every continuing pass.
	pub rotation: Sequence,
}

/// A local variable assignment.
pub struct Assign {
	/// The destination local.
	pub destination: Local,
	/// The source expression.
	pub source: Expression,
}

/// A cyclic swap of locals.
pub struct SwapAll {
	/// The locals to swap, in cycle order.
	pub locals: Vec<Local>,
}

/// A call statement binding its results: `r0, r1 = call;` (or just `call;`).
pub struct Call {
	/// The result locals bound from the call, in port order.
	pub results: Vec<Local>,
	/// The call expression performed for its results and side effects.
	pub call: Expression,
}

/// A table element write: `(table)[offset] = value`.
pub struct SetIndex {
	/// The destination table.
	pub table: Expression,
	/// The element offset.
	pub offset: Expression,
	/// The value being stored.
	pub value: Expression,
}

/// A statement node.
pub enum Statement {
	/// A conditional match.
	Match(Box<Match>),
	/// A repeat loop.
	Repeat(Box<Repeat>),

	/// A local variable assignment.
	Assign(Box<Assign>),
	/// A cyclic swap of locals.
	SwapAll(Box<SwapAll>),

	/// A call statement binding its results.
	Call(Box<Call>),

	/// A table element write.
	SetIndex(Box<SetIndex>),
}
