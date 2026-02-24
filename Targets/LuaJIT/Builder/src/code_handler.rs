use alloc::vec::Vec;
use hashbrown::HashMap;
use ir_graph::{Link, simple};
use luajit_tree::{
	expression::{Expression, Local},
	statement::{
		Assign, Call, GlobalSet, Match, MemoryCopy, MemoryDrop, MemoryFill, MemoryStore, Repeat,
		Sequence, Statement, SwapAll, TableCopy, TableDrop, TableFill, TableSet,
	},
};

use super::{assignment_simplifier::AssignmentSimplifier, data_handler::DataHandler};

pub struct CodeHandler {
	scopes: Vec<Vec<Statement>>,

	regions: HashMap<u32, Sequence>,
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

	pub fn pop_branch(&mut self, id: u32) {
		let code = self.pop_scope();

		self.regions.insert(id, code);
	}

	pub fn push_scope(&mut self) {
		self.scopes.push(Vec::new());
	}

	pub fn do_match(&mut self, regions: &[u32], condition: Link, data_handler: &mut DataHandler) {
		let condition = data_handler.load(condition);
		let condition = if regions.len() == 2 {
			condition.into_boolean()
		} else {
			condition
		};

		let branches: Vec<_> = regions
			.iter()
			.map(|id| self.regions.remove(id).unwrap())
			.collect();

		let statement = Statement::Match(
			Match {
				branches,
				condition,
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(statement);
	}

	pub fn do_repeat(&mut self, condition: Link, data_handler: &mut DataHandler) {
		let condition = data_handler.load(condition);
		let code = self.pop_scope();

		let statement = Statement::Repeat(Repeat { code, condition }.into());

		self.scopes.last_mut().unwrap().push(statement);
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

		self.scopes.last_mut().unwrap().push(statement);
	}

	pub fn do_bulk_assignment(&mut self, id: u32, sources: &[Link], data_handler: &DataHandler) {
		let scope = self.scopes.last_mut().unwrap();

		let mut handler = AssignmentSimplifier::new(data_handler.load_assign_all(id, sources));

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

	pub fn do_call(&mut self, node: &simple::Apply, id: u32, data_handler: &mut DataHandler) {
		let end = node.arguments.len() - usize::from(node.states);
		let statement = Statement::Call(
			Call {
				function: data_handler.load(node.function),
				arguments: data_handler.load_all(&node.arguments[..end]),
				results: data_handler.load_local_assignments(id, 0..node.results),
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(statement);
	}

	pub fn do_global_set(&mut self, node: simple::GlobalSet, data_handler: &mut DataHandler) {
		let statement = Statement::GlobalSet(
			GlobalSet {
				destination: data_handler.load(node.destination),
				source: data_handler.load(node.source),
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(statement);
	}

	pub fn do_table_set(&mut self, node: simple::TableSet, data_handler: &mut DataHandler) {
		let statement = Statement::TableSet(
			TableSet {
				destination: data_handler.load_location(node.destination),
				source: data_handler.load(node.source),
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(statement);
	}

	pub fn do_table_fill(&mut self, node: simple::TableFill, data_handler: &mut DataHandler) {
		let statement = Statement::TableFill(
			TableFill {
				destination: data_handler.load_location(node.destination),
				source: data_handler.load(node.source),
				size: data_handler.load(node.size),
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(statement);
	}

	pub fn do_table_copy(&mut self, node: simple::TableCopy, data_handler: &mut DataHandler) {
		let statement = Statement::TableCopy(
			TableCopy {
				destination: data_handler.load_location(node.destination),
				source: data_handler.load_location(node.source),
				size: data_handler.load(node.size),
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(statement);
	}

	pub fn do_table_drop(&mut self, node: simple::TableDrop, data_handler: &mut DataHandler) {
		let statement = Statement::TableDrop(
			TableDrop {
				source: data_handler.load(node.source),
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(statement);
	}

	pub fn do_memory_store(&mut self, node: simple::MemoryStore, data_handler: &mut DataHandler) {
		let statement = Statement::MemoryStore(
			MemoryStore {
				destination: data_handler.load_location(node.destination),
				source: data_handler.load(node.source),
				r#type: node.r#type,
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(statement);
	}

	pub fn do_memory_fill(&mut self, node: simple::MemoryFill, data_handler: &mut DataHandler) {
		let statement = Statement::MemoryFill(
			MemoryFill {
				destination: data_handler.load_location(node.destination),
				byte: data_handler.load(node.byte),
				size: data_handler.load(node.size),
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(statement);
	}

	pub fn do_memory_copy(&mut self, node: simple::MemoryCopy, data_handler: &mut DataHandler) {
		let statement = Statement::MemoryCopy(
			MemoryCopy {
				destination: data_handler.load_location(node.destination),
				source: data_handler.load_location(node.source),
				size: data_handler.load(node.size),
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(statement);
	}

	pub fn do_memory_drop(&mut self, node: simple::MemoryDrop, data_handler: &mut DataHandler) {
		let statement = Statement::MemoryDrop(
			MemoryDrop {
				source: data_handler.load(node.source),
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(statement);
	}
}
