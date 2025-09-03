use alloc::{boxed::Box, sync::Arc, vec::Vec};

use crate::expression::{Expression, Local, Location, Name};

pub use data_flow_graph::mvp::StoreType;

pub struct Sequence {
	pub list: Vec<Statement>,
}

impl Sequence {
	#[must_use]
	pub fn as_assign_destination(&self) -> Option<Local> {
		match self.list.as_slice() {
			[Statement::AssignAll(assign_all)] => assign_all.as_assign_destination(),
			[Statement::Assign(assign)] => Some(assign.local),
			_ => None,
		}
	}

	#[must_use]
	pub fn into_assign_source(mut self) -> Expression {
		let source = match self.list.pop().unwrap() {
			Statement::AssignAll(assign_all) => Expression::Local(assign_all.into_assign_source()),
			Statement::Assign(assign) => assign.source,
			_ => panic!("should be an assignment"),
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
	pub post: AssignAll,
	pub condition: Expression,
}

pub struct FastDefine {
	pub name: Name,
	pub source: Expression,
}

pub struct SlowDefine {
	pub name: Name,
	pub len: u32,
}

pub struct Assign {
	pub local: Local,
	pub source: Expression,
}

pub struct AssignAll {
	pub assignments: Vec<(Local, Local)>,
}

impl AssignAll {
	const fn as_assign_destination(&self) -> Option<Local> {
		if let &[(local, _)] = self.assignments.as_slice() {
			Some(local)
		} else {
			None
		}
	}

	fn into_assign_source(mut self) -> Local {
		let (_, source) = self.assignments.pop().unwrap();

		assert!(self.assignments.is_empty(), "should be only statement");

		source
	}
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

pub struct TableInit {
	pub destination: Location,
	pub source: Location,
	pub size: Expression,
}

pub struct ElementsDrop {
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

pub struct MemoryInit {
	pub destination: Location,
	pub source: Location,
	pub size: Expression,
}

pub struct DataDrop {
	pub source: Expression,
}

pub enum Statement {
	Match(Box<Match>),
	Repeat(Box<Repeat>),

	FastDefine(Box<FastDefine>),
	SlowDefine(Box<SlowDefine>),
	Assign(Box<Assign>),
	AssignAll(Box<AssignAll>),

	Call(Box<Call>),

	GlobalSet(Box<GlobalSet>),

	TableSet(Box<TableSet>),
	TableFill(Box<TableFill>),
	TableCopy(Box<TableCopy>),
	TableInit(Box<TableInit>),

	ElementsDrop(Box<ElementsDrop>),

	MemoryStore(Box<MemoryStore>),
	MemoryFill(Box<MemoryFill>),
	MemoryCopy(Box<MemoryCopy>),
	MemoryInit(Box<MemoryInit>),

	DataDrop(Box<DataDrop>),
}

pub struct Export {
	pub identifier: Arc<str>,
	pub source: Expression,
}
