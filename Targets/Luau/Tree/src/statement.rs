use alloc::{boxed::Box, sync::Arc, vec::Vec};

use crate::expression::{Expression, Local, Location};

pub use ir_graph::simple::StoreType;

pub struct Sequence {
	pub list: Vec<Statement>,
}

impl Sequence {
	fn as_assign_destination(&self) -> Option<Local> {
		if let [Statement::Assign(assign)] = self.list.as_slice() {
			Some(assign.destination)
		} else {
			None
		}
	}

	#[must_use]
	pub fn as_branch_destination(branches: &[Self]) -> Option<Local> {
		let mut locals = branches.iter().map(Self::as_assign_destination);

		locals
			.next()
			.flatten()
			.filter(|&local| locals.all(|other| other == Some(local)))
	}

	#[must_use]
	pub fn into_assign_source(mut self) -> Expression {
		let source = if let Some(Statement::Assign(assign)) = self.list.pop() {
			assign.source
		} else {
			panic!("should be an assignment")
		};

		assert!(self.list.is_empty(), "should be only statement");

		source
	}
}

pub struct Match {
	pub branches: Vec<Sequence>,
	pub condition: Expression,
}

pub struct Repeat {
	pub code: Sequence,
	pub condition: Expression,
}

pub struct Assign {
	pub destination: Local,
	pub source: Expression,
}

pub struct SwapAll {
	pub locals: Vec<Local>,
}

pub struct Call {
	pub function: Expression,
	pub results: Vec<Local>,
	pub arguments: Vec<Expression>,
}

pub struct GlobalSet {
	pub destination: Expression,
	pub source: Expression,
}

pub struct TableSet {
	pub destination: Location,
	pub source: Expression,
}

pub struct TableFill {
	pub destination: Location,
	pub source: Expression,
	pub size: Expression,
}

pub struct TableCopy {
	pub destination: Location,
	pub source: Location,
	pub size: Expression,
}

pub struct TableDrop {
	pub source: Expression,
}

pub struct MemoryStore {
	pub destination: Location,
	pub source: Expression,
	pub r#type: StoreType,
}

pub struct MemoryFill {
	pub destination: Location,
	pub byte: Expression,
	pub size: Expression,
}

pub struct MemoryCopy {
	pub destination: Location,
	pub source: Location,
	pub size: Expression,
}

pub struct MemoryDrop {
	pub source: Expression,
}

pub enum Statement {
	Match(Box<Match>),
	Repeat(Box<Repeat>),

	Assign(Box<Assign>),
	SwapAll(Box<SwapAll>),

	Call(Box<Call>),

	GlobalSet(Box<GlobalSet>),

	TableSet(Box<TableSet>),
	TableFill(Box<TableFill>),
	TableCopy(Box<TableCopy>),
	TableDrop(Box<TableDrop>),

	MemoryStore(Box<MemoryStore>),
	MemoryFill(Box<MemoryFill>),
	MemoryCopy(Box<MemoryCopy>),
	MemoryDrop(Box<MemoryDrop>),
}

pub struct Export {
	pub identifier: Arc<str>,
	pub source: Expression,
}
