use hashbrown::HashMap;

use ir_graph::{Link, operation};
use luau_tree::{
	expression::{Expression, Local},
	statement::{
		Assign, Call, GlobalSet, Match, MemoryCopy, MemoryDrop, MemoryFill, MemoryStore, Repeat,
		RuntimeCall, Sequence, Statement, SwapAll, TableCopy, TableDrop, TableFill, TableSet,
	},
};

use super::{assignment_simplifier::AssignmentSimplifier, data_handler::DataHandler};

pub struct CodeHandler {
	scopes: Vec<Vec<Statement>>,

	regions: HashMap<usize, Sequence>,
}

impl CodeHandler {
	pub fn new() -> Self {
		Self {
			scopes: Vec::new(),

			regions: HashMap::new(),
		}
	}

	pub fn pop_scope(&mut self) -> Sequence {
		let list = self.scopes.pop().unwrap();

		Sequence { list }
	}

	pub fn pop_branch(&mut self, key: usize) {
		let code = self.pop_scope();

		self.regions.insert(key, code);
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
		branch_keys: &[usize],
		condition: Link,
		data_handler: &mut DataHandler,
	) {
		let condition = data_handler.load(condition);
		let condition = if branch_keys.len() == 2 {
			condition.into_boolean()
		} else {
			condition
		};

		let branches: Vec<_> = branch_keys
			.iter()
			.map(|key| self.regions.remove(key).unwrap())
			.collect();

		if let Some(destination) = Sequence::as_branch_destination(&branches) {
			let source = DataHandler::load_match_expression(condition, branches);

			self.do_assign(destination, source);
		} else {
			self.do_match_statement(condition, branches);
		}
	}

	pub fn do_repeat(&mut self, condition: Link, data_handler: &mut DataHandler) {
		let condition = data_handler.load(condition);
		let code = self.pop_scope();

		let statement = Statement::Repeat(Repeat { code, condition }.into());

		self.push_statement(statement);
	}

	pub fn do_rename(&mut self, destination: Link, source: Link, data_handler: &DataHandler) {
		let Some(destination) = data_handler.get_local(destination) else {
			return;
		};

		let source = data_handler.get_local(source).unwrap();

		self.do_assign(destination, Expression::Local(source));
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

	pub fn do_bulk_assignment(
		&mut self,
		id: u32,
		sources: &[Link],
		source_scope: usize,
		data_handler: &DataHandler,
	) {
		let scope = self.scopes.last_mut().unwrap();

		let mut handler =
			AssignmentSimplifier::new(data_handler.load_assign_all(id, sources, source_scope));

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

	pub fn do_call(&mut self, node: &operation::Apply, id: u32, data_handler: &mut DataHandler) {
		let function = data_handler.load(node.function);
		let arguments = data_handler.load_all(&node.arguments);
		let results = data_handler.load_local_assignments(id, node.results);

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

	pub fn do_mutable_set(&mut self, node: operation::MutableSet, data_handler: &mut DataHandler) {
		let destination = data_handler.load(node.destination);
		let source = data_handler.load(node.source);

		let statement = Statement::GlobalSet(
			GlobalSet {
				destination,
				source,
			}
			.into(),
		);

		self.push_statement(statement);
	}

	pub fn do_table_set(&mut self, node: operation::TableSet, data_handler: &mut DataHandler) {
		let destination = data_handler.load_location(node.destination);
		let source = data_handler.load(node.source);

		let statement = Statement::TableSet(
			TableSet {
				destination,
				source,
			}
			.into(),
		);

		self.push_statement(statement);
	}

	pub fn do_table_fill(&mut self, node: operation::TableFill, data_handler: &mut DataHandler) {
		let destination = data_handler.load_location(node.destination);
		let source = data_handler.load(node.source);
		let size = data_handler.load(node.size);

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

	pub fn do_table_copy(&mut self, node: operation::TableCopy, data_handler: &mut DataHandler) {
		let destination = data_handler.load_location(node.destination);
		let source = data_handler.load_location(node.source);
		let size = data_handler.load(node.size);

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

	pub fn do_table_drop(&mut self, node: operation::TableDrop, data_handler: &mut DataHandler) {
		let statement = Statement::TableDrop(
			TableDrop {
				source: data_handler.load(node.source),
			}
			.into(),
		);

		self.push_statement(statement);
	}

	pub fn do_memory_store(
		&mut self,
		node: operation::MemoryStore,
		data_handler: &mut DataHandler,
	) {
		let destination = data_handler.load_location(node.destination);
		let source = data_handler.load(node.source);

		let statement = Statement::MemoryStore(
			MemoryStore {
				destination,
				source,
				kind: node.kind,
			}
			.into(),
		);

		self.push_statement(statement);
	}

	pub fn do_memory_fill(&mut self, node: operation::MemoryFill, data_handler: &mut DataHandler) {
		let destination = data_handler.load_location(node.destination);
		let byte = data_handler.load(node.byte);
		let size = data_handler.load(node.size);

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

	pub fn do_memory_copy(&mut self, node: operation::MemoryCopy, data_handler: &mut DataHandler) {
		let destination = data_handler.load_location(node.destination);
		let source = data_handler.load_location(node.source);
		let size = data_handler.load(node.size);

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

	pub fn do_memory_drop(&mut self, node: operation::MemoryDrop, data_handler: &mut DataHandler) {
		let statement = Statement::MemoryDrop(
			MemoryDrop {
				source: data_handler.load(node.source),
			}
			.into(),
		);

		self.push_statement(statement);
	}
}
