use ir_graph::{
	Link,
	operation::{self, StoreType},
};
use luajit_tree::{
	expression::{Expression, Local, Location},
	statement::{
		Assign, Call, GlobalSet, Match, MemoryCopy, MemoryDrop, MemoryFill, MemoryStore, Repeat,
		RuntimeCall, Sequence, Statement, SwapAll, TableCopy, TableDrop, TableFill, TableSet,
	},
};

use super::{assignment_simplifier::AssignmentSimplifier, data_handler::DataHandler};

pub struct CodeHandler {
	scopes: Vec<Vec<Statement>>,
}

impl CodeHandler {
	pub const fn new() -> Self {
		Self { scopes: Vec::new() }
	}

	pub fn pop_scope(&mut self) -> Sequence {
		let list = self.scopes.pop().unwrap();

		Sequence { list }
	}

	pub fn push_scope(&mut self) {
		self.scopes.push(Vec::new());
	}

	fn push_statement(&mut self, statement: Statement) {
		self.scopes.last_mut().unwrap().push(statement);
	}

	fn do_match_statement(&mut self, condition: Expression, branches: Vec<Sequence>) {
		let statement = Statement::Match(
			Match {
				branches,
				condition,
			}
			.into(),
		);

		self.push_statement(statement);
	}

	pub fn do_match(
		&mut self,
		branches: Vec<Sequence>,
		condition: Link,
		scope: usize,
		data_handler: &mut DataHandler,
	) {
		let condition = data_handler.load(scope, condition);
		let condition = if branches.len() == 2 {
			condition.into_boolean()
		} else {
			condition
		};

		self.do_match_statement(condition, branches);
	}

	pub fn do_repeat(&mut self, condition: Link, scope: usize, data_handler: &mut DataHandler) {
		let condition = data_handler.load(scope, condition);
		let code = self.pop_scope();

		let statement = Statement::Repeat(Repeat { code, condition }.into());

		self.push_statement(statement);
	}

	pub fn do_assign(&mut self, destination: Local, source: Expression) {
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

	pub fn do_local_moves(&mut self, pairs: Vec<(Local, Local)>) {
		let scope = self.scopes.last_mut().unwrap();

		let mut handler = AssignmentSimplifier::new(pairs);

		handler.find_all_assigns(|destination, source| {
			let source = Expression::Local(source);
			let statement = Statement::Assign(
				Assign {
					destination,
					source,
				}
				.into(),
			);

			scope.push(statement);
		});

		handler.find_all_swaps(|locals| {
			if locals.len() <= 1 {
				return;
			}

			let locals = locals.to_vec();
			let statement = Statement::SwapAll(SwapAll { locals }.into());

			scope.push(statement);
		});
	}

	pub fn do_call(
		&mut self,
		scope: usize,
		node: &operation::Apply,
		id: u32,
		data_handler: &mut DataHandler,
	) {
		let function = data_handler.load(scope, node.function);
		let arguments = data_handler.load_all(scope, &node.arguments);
		let results = data_handler.load_result_locals(scope, id, node.results);

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

	pub fn do_runtime_call(&mut self, name: &'static str, arguments: Vec<Expression>) {
		let statement = Statement::RuntimeCall(RuntimeCall { name, arguments }.into());

		self.push_statement(statement);
	}

	pub fn do_mutable_set(&mut self, destination: Expression, source: Expression) {
		let statement = Statement::GlobalSet(
			GlobalSet {
				destination,
				source,
			}
			.into(),
		);

		self.push_statement(statement);
	}

	pub fn do_table_set(&mut self, destination: Location, source: Expression) {
		let statement = Statement::TableSet(
			TableSet {
				destination,
				source,
			}
			.into(),
		);

		self.push_statement(statement);
	}

	pub fn do_table_fill(&mut self, destination: Location, source: Expression, size: Expression) {
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

	pub fn do_table_copy(&mut self, destination: Location, source: Location, size: Expression) {
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

	pub fn do_table_drop(&mut self, source: Expression) {
		let statement = Statement::TableDrop(TableDrop { source }.into());

		self.push_statement(statement);
	}

	pub fn do_memory_store(&mut self, destination: Location, source: Expression, kind: StoreType) {
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

	pub fn do_memory_fill(&mut self, destination: Location, byte: Expression, size: Expression) {
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

	pub fn do_memory_copy(&mut self, destination: Location, source: Location, size: Expression) {
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

	pub fn do_memory_drop(&mut self, source: Expression) {
		let statement = Statement::MemoryDrop(MemoryDrop { source }.into());

		self.push_statement(statement);
	}
}
