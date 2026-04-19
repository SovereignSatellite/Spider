//! Turing machine lifter that compiles source code into a data flow graph.

extern crate alloc;

use alloc::sync::{Arc, Weak};

use parking_lot::Mutex;

use ir_graph::{
	Link, Node,
	list::resizable::Resizable,
	operation::{
		Apply, Fence, LoadType, Location, MemoryLoad, MemoryNew, MemoryStore, StoreType, integer,
	},
	region::{Branch, Import, Match, Module, Repeat, module},
};

const CELL_SIZE: u32 = 4;
const MEMORY_SIZE: u32 = 1_024 * 4 * CELL_SIZE;

enum Operator {
	OffsetAdd,
	OffsetSubtract,
	MemoryAdd,
	MemorySubtract,
	Ask,
	Tell,
	Start,
	End,
}

impl TryFrom<char> for Operator {
	type Error = ();

	fn try_from(character: char) -> Result<Self, Self::Error> {
		let operator = match character {
			'>' => Self::OffsetAdd,
			'<' => Self::OffsetSubtract,
			'+' => Self::MemoryAdd,
			'-' => Self::MemorySubtract,
			',' => Self::Ask,
			'.' => Self::Tell,
			'[' => Self::Start,
			']' => Self::End,

			_ => return Err(()),
		};

		Ok(operator)
	}
}

/// A lifter that compiles source code into a data flow graph.
pub struct TuringMachineLifter {
	loads: Vec<Link>,
	store: Link,
	offset: Link,

	io: Link,
	ask: Link,
	tell: Link,

	operators: Vec<Operator>,
}

impl TuringMachineLifter {
	/// Creates a new Turing machine lifter.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			loads: Vec::new(),
			store: Link::DANGLING,
			offset: Link::DANGLING,

			io: Link::DANGLING,
			ask: Link::DANGLING,
			tell: Link::DANGLING,

			operators: Vec::new(),
		}
	}

	fn create_memory(&mut self, nodes: &mut Vec<Node>) {
		self.loads.clear();

		self.store = MemoryNew::add_into(nodes, Vec::new(), MEMORY_SIZE, MEMORY_SIZE);
		self.offset = Node::add_i32_into(nodes, 0);
	}

	fn create_io(&mut self, nodes: &mut Vec<Node>, arguments: u32) {
		let environment = Link(arguments, module::Arguments::ENVIRONMENT_PORT);
		let namespace = Arc::<str>::from("turing");

		self.io = Link(arguments, module::Arguments::STATE_PORT);
		self.ask = Import::add_into(nodes, environment, Arc::clone(&namespace), "ask".into());
		self.tell = Import::add_into(nodes, environment, namespace, "tell".into());
	}

	fn reconcile_store(&mut self, nodes: &mut Vec<Node>) -> Link {
		let sources: Resizable<_, _> = self.loads.iter().copied().collect();

		self.loads.clear();

		match *sources {
			[state] => state,
			[] => self.store,
			[..] => Link(Fence::add_into(nodes, sources), 0),
		}
	}

	fn do_load(&mut self, nodes: &mut Vec<Node>) -> Link {
		let source = Location {
			reference: self.store,
			offset: self.offset,
		};
		let (result, state) = MemoryLoad::add_into(nodes, source, LoadType::I32);

		self.loads.push(state);

		result
	}

	fn do_store(&mut self, nodes: &mut Vec<Node>, source: Link) {
		let destination = Location {
			reference: self.reconcile_store(nodes),
			offset: self.offset,
		};

		self.store = MemoryStore::add_into(nodes, destination, source, StoreType::I32);
	}

	fn do_condition(&mut self, nodes: &mut Vec<Node>) -> Link {
		let lhs = self.do_load(nodes);
		let rhs = Node::add_i32_into(nodes, 0);

		integer::CompareOperation::add_into(
			nodes,
			lhs,
			rhs,
			integer::Type::I32,
			integer::CompareOperator::NotEqual,
		)
	}

	fn handle_offset_operation(
		&mut self,
		nodes: &mut Vec<Node>,
		operator: integer::BinaryOperator,
	) {
		let rhs = Node::add_i32_into(nodes, CELL_SIZE.try_into().unwrap());

		self.offset = integer::BinaryOperation::add_into(
			nodes,
			self.offset,
			rhs,
			integer::Type::I32,
			operator,
		);
	}

	fn handle_memory_operation(
		&mut self,
		nodes: &mut Vec<Node>,
		operator: integer::BinaryOperator,
	) {
		let lhs = self.do_load(nodes);
		let rhs = Node::add_i32_into(nodes, 1);
		let source =
			integer::BinaryOperation::add_into(nodes, lhs, rhs, integer::Type::I32, operator);

		self.do_store(nodes, source);
	}

	fn handle_ask(&mut self, nodes: &mut Vec<Node>) {
		let apply = Apply::add_into(nodes, self.ask, vec![self.io], 2);

		self.io = Link(apply, 0);

		self.do_store(nodes, Link(apply, 1));
	}

	fn handle_tell(&mut self, nodes: &mut Vec<Node>) {
		let source = self.do_load(nodes);
		let apply = Apply::add_into(nodes, self.tell, vec![self.io, source], 1);

		self.io = Link(apply, 0);
	}

	fn pull_all_active(&mut self, nodes: &mut Vec<Node>) -> Vec<Link> {
		vec![
			self.reconcile_store(nodes),
			self.offset,
			self.io,
			self.ask,
			self.tell,
		]
	}

	const fn push_all_active(&mut self, source: u32) {
		self.store = Link(source, 0);
		self.offset = Link(source, 1);
		self.io = Link(source, 2);
		self.ask = Link(source, 3);
		self.tell = Link(source, 4);
	}

	fn create_false_branch(&mut self, parent: &Weak<Mutex<Match>>) -> Arc<Mutex<Branch>> {
		Branch::create(Weak::clone(parent), |nodes, arguments| {
			self.push_all_active(arguments);
			self.pull_all_active(nodes)
		})
	}

	fn create_repeat(&mut self, nodes: &mut Vec<Node>) {
		let arguments = self.pull_all_active(nodes);

		let repeat = Repeat::add_into(nodes, arguments, |nodes, repeat_arguments| {
			self.push_all_active(repeat_arguments);
			self.handle_code(nodes);

			let condition = self.do_condition(nodes);
			let results = self.pull_all_active(nodes);

			(results, condition)
		});

		self.push_all_active(repeat);
	}

	fn create_true_branch(&mut self, parent: &Weak<Mutex<Match>>) -> Arc<Mutex<Branch>> {
		Branch::create(Weak::clone(parent), |nodes, arguments| {
			self.push_all_active(arguments);
			self.create_repeat(nodes);
			self.pull_all_active(nodes)
		})
	}

	fn handle_block(&mut self, nodes: &mut Vec<Node>) {
		let condition = self.do_condition(nodes);
		let arguments = self.pull_all_active(nodes);

		let match_id = Match::add_into(nodes, arguments, condition, |parent| {
			let false_branch = self.create_false_branch(parent);
			let true_branch = self.create_true_branch(parent);

			vec![false_branch, true_branch]
		});

		self.push_all_active(match_id);
	}

	fn handle_code(&mut self, nodes: &mut Vec<Node>) {
		while let Some(operator) = self.operators.pop() {
			match operator {
				Operator::OffsetAdd => {
					self.handle_offset_operation(nodes, integer::BinaryOperator::Add);
				}
				Operator::OffsetSubtract => {
					self.handle_offset_operation(nodes, integer::BinaryOperator::Subtract);
				}

				Operator::MemoryAdd => {
					self.handle_memory_operation(nodes, integer::BinaryOperator::Add);
				}
				Operator::MemorySubtract => {
					self.handle_memory_operation(nodes, integer::BinaryOperator::Subtract);
				}

				Operator::Ask => self.handle_ask(nodes),
				Operator::Tell => self.handle_tell(nodes),

				Operator::Start => self.handle_block(nodes),
				Operator::End => return,
			}
		}
	}

	/// Compiles the given source code into a module.
	pub fn run(&mut self, source: &str) -> Arc<Mutex<Module>> {
		self.operators.clear();
		self.operators.extend(
			source
				.chars()
				.rev()
				.filter_map(|character| Operator::try_from(character).ok()),
		);

		Module::create(|nodes, arguments| {
			self.create_memory(nodes);
			self.create_io(nodes, arguments);
			self.handle_code(nodes);

			(self.io, Vec::new())
		})
	}
}

impl Default for TuringMachineLifter {
	fn default() -> Self {
		Self::new()
	}
}
