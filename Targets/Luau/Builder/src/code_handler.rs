use alloc::vec::Vec;
use data_flow_graph::mvp;

use luau_tree::{
	expression::{Expression, Local, Name},
	statement::{
		Assign, AssignAll, Call, DataDrop, ElementsDrop, FastDefine, GlobalSet, Match, MemoryCopy,
		MemoryFill, MemoryInit, MemoryStore, Repeat, Sequence, SlowDefine, Statement, TableCopy,
		TableFill, TableInit, TableSet,
	},
};

use super::data_handler::DataHandler;

pub struct CodeHandler {
	scopes: Vec<Vec<Statement>>,

	list: Vec<Statement>,
}

impl CodeHandler {
	pub const fn new() -> Self {
		Self {
			scopes: Vec::new(),

			list: Vec::new(),
		}
	}

	pub fn pop_scope(&mut self) -> Sequence {
		let parent = self.scopes.pop().unwrap_or_default();
		let list = core::mem::replace(&mut self.list, parent);

		Sequence { list }
	}

	pub fn push_scope(&mut self) {
		let parent = core::mem::take(&mut self.list);

		self.scopes.push(parent);
	}

	pub fn do_match(&mut self, branches: Vec<Sequence>, condition: Expression) {
		let r#match = Statement::Match(
			Match {
				branches,
				condition,
			}
			.into(),
		);

		self.list.push(r#match);
	}

	pub fn do_repeat(&mut self, code: Sequence, post: AssignAll, condition: Expression) {
		let repeat = Statement::Repeat(
			Repeat {
				code,
				post,
				condition,
			}
			.into(),
		);

		self.list.push(repeat);
	}

	pub fn do_fast_define(&mut self, name: Name, source: Expression) {
		let define = Statement::FastDefine(FastDefine { name, source }.into());

		self.list.push(define);
	}

	pub fn do_slow_define(&mut self, name: Name, len: u32) {
		let define = Statement::SlowDefine(SlowDefine { name, len }.into());

		self.list.push(define);
	}

	pub fn do_assign(&mut self, local: Local, source: Expression) {
		if let Expression::Local(source) = source {
			if local == source {
				return;
			}
		}

		let assign = Statement::Assign(Assign { local, source }.into());

		self.list.push(assign);
	}

	pub fn do_assign_all(&mut self, assignments: Vec<(Local, Local)>) {
		if assignments.is_empty() {
			return;
		}

		let assign_all = Statement::AssignAll(AssignAll { assignments }.into());

		self.list.push(assign_all);
	}

	pub fn do_call(
		&mut self,
		call: &mvp::Call,
		results: Vec<Local>,
		data_handler: &mut DataHandler,
	) {
		let end = call.arguments.len() - usize::from(call.states);
		let call = Statement::Call(
			Call {
				function: data_handler.load(call.function).unwrap(),
				arguments: data_handler.load_sources(&call.arguments[..end]),
				results,
			}
			.into(),
		);

		self.list.push(call);
	}

	pub fn do_global_set(&mut self, global_set: mvp::GlobalSet, data_handler: &mut DataHandler) {
		let global_set = Statement::GlobalSet(
			GlobalSet {
				destination: data_handler.load(global_set.destination).unwrap(),
				source: data_handler.load(global_set.source).unwrap(),
			}
			.into(),
		);

		self.list.push(global_set);
	}

	pub fn do_table_set(&mut self, table_set: mvp::TableSet, data_handler: &mut DataHandler) {
		let table_set = Statement::TableSet(
			TableSet {
				destination: data_handler.load_location(table_set.destination),
				source: data_handler.load(table_set.source).unwrap(),
			}
			.into(),
		);

		self.list.push(table_set);
	}

	pub fn do_table_fill(&mut self, table_fill: mvp::TableFill, data_handler: &mut DataHandler) {
		let table_fill = Statement::TableFill(
			TableFill {
				destination: data_handler.load_location(table_fill.destination),
				source: data_handler.load(table_fill.source).unwrap(),
				size: data_handler.load(table_fill.size).unwrap(),
			}
			.into(),
		);

		self.list.push(table_fill);
	}

	pub fn do_table_copy(&mut self, table_copy: mvp::TableCopy, data_handler: &mut DataHandler) {
		let table_copy = Statement::TableCopy(
			TableCopy {
				destination: data_handler.load_location(table_copy.destination),
				source: data_handler.load_location(table_copy.source),
				size: data_handler.load(table_copy.size).unwrap(),
			}
			.into(),
		);

		self.list.push(table_copy);
	}

	pub fn do_table_init(&mut self, table_init: mvp::TableInit, data_handler: &mut DataHandler) {
		let table_init = Statement::TableInit(
			TableInit {
				destination: data_handler.load_location(table_init.destination),
				source: data_handler.load_location(table_init.source),
				size: data_handler.load(table_init.size).unwrap(),
			}
			.into(),
		);

		self.list.push(table_init);
	}

	pub fn do_elements_drop(
		&mut self,
		elements_drop: mvp::ElementsDrop,
		data_handler: &mut DataHandler,
	) {
		let elements_drop = Statement::ElementsDrop(
			ElementsDrop {
				source: data_handler.load(elements_drop.source).unwrap(),
			}
			.into(),
		);

		self.list.push(elements_drop);
	}

	pub fn do_memory_store(
		&mut self,
		memory_store: mvp::MemoryStore,
		data_handler: &mut DataHandler,
	) {
		let memory_store = Statement::MemoryStore(
			MemoryStore {
				destination: data_handler.load_location(memory_store.destination),
				source: data_handler.load(memory_store.source).unwrap(),
				r#type: memory_store.r#type,
			}
			.into(),
		);

		self.list.push(memory_store);
	}

	pub fn do_memory_fill(&mut self, memory_fill: mvp::MemoryFill, data_handler: &mut DataHandler) {
		let memory_fill = Statement::MemoryFill(
			MemoryFill {
				destination: data_handler.load_location(memory_fill.destination),
				byte: data_handler.load(memory_fill.byte).unwrap(),
				size: data_handler.load(memory_fill.size).unwrap(),
			}
			.into(),
		);

		self.list.push(memory_fill);
	}

	pub fn do_memory_copy(&mut self, memory_copy: mvp::MemoryCopy, data_handler: &mut DataHandler) {
		let memory_copy = Statement::MemoryCopy(
			MemoryCopy {
				destination: data_handler.load_location(memory_copy.destination),
				source: data_handler.load_location(memory_copy.source),
				size: data_handler.load(memory_copy.size).unwrap(),
			}
			.into(),
		);

		self.list.push(memory_copy);
	}

	pub fn do_memory_init(&mut self, memory_init: mvp::MemoryInit, data_handler: &mut DataHandler) {
		let memory_init = Statement::MemoryInit(
			MemoryInit {
				destination: data_handler.load_location(memory_init.destination),
				source: data_handler.load_location(memory_init.source),
				size: data_handler.load(memory_init.size).unwrap(),
			}
			.into(),
		);

		self.list.push(memory_init);
	}

	pub fn do_data_drop(&mut self, data_drop: mvp::DataDrop, data_handler: &mut DataHandler) {
		let data_drop = Statement::DataDrop(
			DataDrop {
				source: data_handler.load(data_drop.source).unwrap(),
			}
			.into(),
		);

		self.list.push(data_drop);
	}
}
