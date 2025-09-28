use alloc::vec::Vec;
use data_flow_graph::{Link, mvp};

use hashbrown::HashMap;
use luau_tree::{
	expression::{Expression, Local},
	statement::{
		Assign, AssignAll, Call, DataDrop, ElementsDrop, GlobalSet, Match, MemoryCopy, MemoryFill,
		MemoryInit, MemoryStore, Repeat, Sequence, Statement, TableCopy, TableFill, TableInit,
		TableSet,
	},
};

use super::data_handler::DataHandler;

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

		let branches = regions
			.iter()
			.map(|id| self.regions.remove(id).unwrap())
			.collect();

		let r#match = Statement::Match(
			Match {
				branches,
				condition,
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(r#match);
	}

	pub fn do_repeat(&mut self, condition: Link, data_handler: &mut DataHandler) {
		let condition = data_handler.load(condition).into_boolean();
		let code = self.pop_scope();

		let repeat = Statement::Repeat(Repeat { code, condition }.into());

		self.scopes.last_mut().unwrap().push(repeat);
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

		let assign = Statement::Assign(
			Assign {
				destination,
				source,
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(assign);
	}

	pub fn do_assign_all(&mut self, id: u32, sources: &[Link], data_handler: &DataHandler) {
		let mut assignments = data_handler.load_assign_all(id, sources);

		if assignments.is_empty() {
			return;
		}

		assignments.sort_unstable();

		let assign_all = Statement::AssignAll(AssignAll { assignments }.into());

		self.scopes.last_mut().unwrap().push(assign_all);
	}

	pub fn do_call(&mut self, call: &mvp::Call, id: u32, data_handler: &mut DataHandler) {
		let end = call.arguments.len() - usize::from(call.states);
		let call = Statement::Call(
			Call {
				function: data_handler.load(call.function),
				arguments: data_handler.load_all(&call.arguments[..end]),
				results: data_handler.load_local_assignments(id, 0..call.results),
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(call);
	}

	pub fn do_global_set(&mut self, global_set: mvp::GlobalSet, data_handler: &mut DataHandler) {
		let global_set = Statement::GlobalSet(
			GlobalSet {
				destination: data_handler.load(global_set.destination),
				source: data_handler.load(global_set.source),
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(global_set);
	}

	pub fn do_table_set(&mut self, table_set: mvp::TableSet, data_handler: &mut DataHandler) {
		let table_set = Statement::TableSet(
			TableSet {
				destination: data_handler.load_location(table_set.destination),
				source: data_handler.load(table_set.source),
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(table_set);
	}

	pub fn do_table_fill(&mut self, table_fill: mvp::TableFill, data_handler: &mut DataHandler) {
		let table_fill = Statement::TableFill(
			TableFill {
				destination: data_handler.load_location(table_fill.destination),
				source: data_handler.load(table_fill.source),
				size: data_handler.load(table_fill.size),
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(table_fill);
	}

	pub fn do_table_copy(&mut self, table_copy: mvp::TableCopy, data_handler: &mut DataHandler) {
		let table_copy = Statement::TableCopy(
			TableCopy {
				destination: data_handler.load_location(table_copy.destination),
				source: data_handler.load_location(table_copy.source),
				size: data_handler.load(table_copy.size),
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(table_copy);
	}

	pub fn do_table_init(&mut self, table_init: mvp::TableInit, data_handler: &mut DataHandler) {
		let table_init = Statement::TableInit(
			TableInit {
				destination: data_handler.load_location(table_init.destination),
				source: data_handler.load_location(table_init.source),
				size: data_handler.load(table_init.size),
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(table_init);
	}

	pub fn do_elements_drop(
		&mut self,
		elements_drop: mvp::ElementsDrop,
		data_handler: &mut DataHandler,
	) {
		let elements_drop = Statement::ElementsDrop(
			ElementsDrop {
				source: data_handler.load(elements_drop.source),
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(elements_drop);
	}

	pub fn do_memory_store(
		&mut self,
		memory_store: mvp::MemoryStore,
		data_handler: &mut DataHandler,
	) {
		let memory_store = Statement::MemoryStore(
			MemoryStore {
				destination: data_handler.load_location(memory_store.destination),
				source: data_handler.load(memory_store.source),
				r#type: memory_store.r#type,
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(memory_store);
	}

	pub fn do_memory_fill(&mut self, memory_fill: mvp::MemoryFill, data_handler: &mut DataHandler) {
		let memory_fill = Statement::MemoryFill(
			MemoryFill {
				destination: data_handler.load_location(memory_fill.destination),
				byte: data_handler.load(memory_fill.byte),
				size: data_handler.load(memory_fill.size),
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(memory_fill);
	}

	pub fn do_memory_copy(&mut self, memory_copy: mvp::MemoryCopy, data_handler: &mut DataHandler) {
		let memory_copy = Statement::MemoryCopy(
			MemoryCopy {
				destination: data_handler.load_location(memory_copy.destination),
				source: data_handler.load_location(memory_copy.source),
				size: data_handler.load(memory_copy.size),
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(memory_copy);
	}

	pub fn do_memory_init(&mut self, memory_init: mvp::MemoryInit, data_handler: &mut DataHandler) {
		let memory_init = Statement::MemoryInit(
			MemoryInit {
				destination: data_handler.load_location(memory_init.destination),
				source: data_handler.load_location(memory_init.source),
				size: data_handler.load(memory_init.size),
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(memory_init);
	}

	pub fn do_data_drop(&mut self, data_drop: mvp::DataDrop, data_handler: &mut DataHandler) {
		let data_drop = Statement::DataDrop(
			DataDrop {
				source: data_handler.load(data_drop.source),
			}
			.into(),
		);

		self.scopes.last_mut().unwrap().push(data_drop);
	}
}
