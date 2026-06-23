use ir_graph::operation::StoreType;
use luau_tree::{
	expression::{Expression, Local, Location},
	statement::{
		Assign, Call, GlobalSet, Match, MemoryCopy, MemoryDrop, MemoryFill, MemoryStore, Repeat,
		RuntimeCall, Sequence, Statement, SwapAll, TableCopy, TableDrop, TableFill, TableSet,
	},
};

use super::assignment_simplifier::AssignmentSimplifier;

pub struct CodeHandler {
	scopes: Vec<Vec<Statement>>,
}

impl CodeHandler {
	#[must_use]
	pub const fn new() -> Self {
		Self { scopes: Vec::new() }
	}

	pub fn pop_scope(&mut self) -> Sequence {
		let statements = self.scopes.pop().unwrap();

		Sequence { statements }
	}

	pub fn push_scope(&mut self) {
		self.scopes.push(Vec::new());
	}

	fn push_statement(&mut self, statement: Statement) {
		self.scopes.last_mut().unwrap().push(statement);
	}

	pub fn emit_match(&mut self, condition: Expression, branches: Vec<Sequence>) {
		let statement = Statement::Match(
			Match {
				branches,
				condition,
			}
			.into(),
		);

		self.push_statement(statement);
	}

	pub fn emit_repeat(&mut self, condition: Expression, rotation: Sequence) {
		let code = self.pop_scope();

		let statement = Statement::Repeat(
			Repeat {
				code,
				condition,
				rotation,
			}
			.into(),
		);

		self.push_statement(statement);
	}

	pub fn emit_assign(&mut self, destination: Local, source: Expression) {
		if let Expression::Local(source) = source
			&& destination == source
		{
			return;
		}

		let statement = Statement::Assign(
			Assign {
				destination,
				source,
			}
			.into(),
		);

		self.push_statement(statement);
	}

	pub fn emit_local_moves(&mut self, pairs: Vec<(Local, Local)>) {
		let scope = self.scopes.last_mut().unwrap();
		let mut simplifier = AssignmentSimplifier::new(pairs);

		simplifier.find_all_assigns(|destination, source| {
			let source = Expression::Local(source);

			scope.push(Statement::Assign(
				Assign {
					destination,
					source,
				}
				.into(),
			));
		});

		simplifier.find_all_swaps(|locals| {
			if locals.len() <= 1 {
				return;
			}

			let locals = locals.to_vec();

			scope.push(Statement::SwapAll(SwapAll { locals }.into()));
		});
	}

	pub fn emit_call(
		&mut self,
		function: Expression,
		results: Vec<Local>,
		arguments: Vec<Expression>,
	) {
		let statement = Statement::Call(
			Call {
				function,
				results,
				arguments,
			}
			.into(),
		);

		self.push_statement(statement);
	}

	pub fn emit_runtime_call(&mut self, name: &'static str, arguments: Vec<Expression>) {
		let statement = Statement::RuntimeCall(RuntimeCall { name, arguments }.into());

		self.push_statement(statement);
	}

	pub fn emit_mutable_set(&mut self, destination: Expression, source: Expression) {
		let statement = Statement::GlobalSet(
			GlobalSet {
				destination,
				source,
			}
			.into(),
		);

		self.push_statement(statement);
	}

	pub fn emit_table_set(&mut self, destination: Location, source: Expression) {
		let statement = Statement::TableSet(
			TableSet {
				destination,
				source,
			}
			.into(),
		);

		self.push_statement(statement);
	}

	pub fn emit_table_fill(&mut self, destination: Location, source: Expression, size: Expression) {
		let statement = Statement::TableFill(
			TableFill {
				destination,
				source,
				size,
			}
			.into(),
		);

		self.push_statement(statement);
	}

	pub fn emit_table_copy(&mut self, destination: Location, source: Location, size: Expression) {
		let statement = Statement::TableCopy(
			TableCopy {
				destination,
				source,
				size,
			}
			.into(),
		);

		self.push_statement(statement);
	}

	pub fn emit_table_drop(&mut self, source: Expression) {
		let statement = Statement::TableDrop(TableDrop { source }.into());

		self.push_statement(statement);
	}

	pub fn emit_memory_store(
		&mut self,
		destination: Location,
		source: Expression,
		kind: StoreType,
	) {
		let statement = Statement::MemoryStore(
			MemoryStore {
				destination,
				source,
				kind,
			}
			.into(),
		);

		self.push_statement(statement);
	}

	pub fn emit_memory_fill(&mut self, destination: Location, byte: Expression, size: Expression) {
		let statement = Statement::MemoryFill(
			MemoryFill {
				destination,
				byte,
				size,
			}
			.into(),
		);

		self.push_statement(statement);
	}

	pub fn emit_memory_copy(&mut self, destination: Location, source: Location, size: Expression) {
		let statement = Statement::MemoryCopy(
			MemoryCopy {
				destination,
				source,
				size,
			}
			.into(),
		);

		self.push_statement(statement);
	}

	pub fn emit_memory_drop(&mut self, source: Expression) {
		let statement = Statement::MemoryDrop(MemoryDrop { source }.into());

		self.push_statement(statement);
	}
}
