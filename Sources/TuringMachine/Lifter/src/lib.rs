//! Turing machine lifter that compiles source code into a data flow graph.

extern crate alloc;

use alloc::sync::{Arc, Weak};

use parking_lot::Mutex;

use ir_graph::{
	Link, Node,
	list::resizable::Resizable,
	operation::{
		Apply, Fence, Import, LoadType, Location, MemoryLoad, MemoryNew, MemoryStore, StoreType,
		integer,
	},
	region::{Branch, Function, Match, Repeat},
};

const CELL_SIZE: u32 = 4;
const MEMORY_SIZE: u32 = 1_024 * 4 * CELL_SIZE;

enum Operator {
	OffsetAdd,
	OffsetSubtract,
	MemoryAdd,
	MemorySubtract,
	Input,
	Output,
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
			',' => Self::Input,
			'.' => Self::Output,
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

	io_state: Link,

	operators: Vec<Operator>,

	namespace: Arc<str>,
	input_identifier: Arc<str>,
	output_identifier: Arc<str>,
}

impl TuringMachineLifter {
	/// Creates a new Turing machine lifter.
	#[must_use]
	pub fn new() -> Self {
		Self {
			loads: Vec::new(),
			store: Link::DANGLING,
			offset: Link::DANGLING,

			io_state: Link::DANGLING,

			operators: Vec::new(),

			namespace: Arc::from("turing"),
			input_identifier: Arc::from("ask"),
			output_identifier: Arc::from("tell"),
		}
	}

	fn create_memory(&mut self, nodes: &mut Vec<Node>) {
		self.loads.clear();

		self.store = MemoryNew::add_into(nodes, Vec::new(), MEMORY_SIZE, MEMORY_SIZE);
		self.offset = Node::add_i32_into(nodes, 0);
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

	fn emit_load(&mut self, nodes: &mut Vec<Node>) -> Link {
		let source = Location {
			reference: self.store,
			offset: self.offset,
		};
		let (result, state) = MemoryLoad::add_into(nodes, source, LoadType::I32);

		self.loads.push(state);

		result
	}

	fn emit_store(&mut self, nodes: &mut Vec<Node>, source: Link) {
		let destination = Location {
			reference: self.reconcile_store(nodes),
			offset: self.offset,
		};

		self.store = MemoryStore::add_into(nodes, destination, source, StoreType::I32);
	}

	fn emit_condition(&mut self, nodes: &mut Vec<Node>) -> Link {
		let lhs = self.emit_load(nodes);
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
		let lhs = self.emit_load(nodes);
		let rhs = Node::add_i32_into(nodes, 1);
		let source =
			integer::BinaryOperation::add_into(nodes, lhs, rhs, integer::Type::I32, operator);

		self.emit_store(nodes, source);
	}

	fn handle_input(&mut self, nodes: &mut Vec<Node>) {
		let function = Import::add_into(
			nodes,
			Arc::clone(&self.namespace),
			Arc::clone(&self.input_identifier),
		);
		let call = Apply::add_into(nodes, function, vec![self.io_state], 2);

		self.io_state = Link(call, 0);

		let character = Link(call, 1);

		self.emit_store(nodes, character);
	}

	fn handle_output(&mut self, nodes: &mut Vec<Node>) {
		let source = self.emit_load(nodes);
		let function = Import::add_into(
			nodes,
			Arc::clone(&self.namespace),
			Arc::clone(&self.output_identifier),
		);
		let call = Apply::add_into(nodes, function, vec![self.io_state, source], 1);

		self.io_state = Link(call, 0);
	}

	fn capture_state(&mut self, nodes: &mut Vec<Node>) -> Vec<Link> {
		vec![self.reconcile_store(nodes), self.offset, self.io_state]
	}

	const fn rebind_state(&mut self, source: u32) {
		self.store = Link(source, 0);
		self.offset = Link(source, 1);
		self.io_state = Link(source, 2);
	}

	fn create_false_branch(
		&mut self,
		parent: &Weak<Mutex<Match>>,
		argument_count: u16,
	) -> Arc<Mutex<Branch>> {
		Branch::create(Weak::clone(parent), argument_count, |nodes, arguments| {
			self.rebind_state(arguments);

			self.capture_state(nodes)
		})
	}

	fn create_repeat(&mut self, nodes: &mut Vec<Node>) {
		let arguments = self.capture_state(nodes);

		let repeat = Repeat::add_into(nodes, arguments, |nodes, repeat_arguments| {
			self.rebind_state(repeat_arguments);
			self.handle_code(nodes);

			let condition = self.emit_condition(nodes);
			let results = self.capture_state(nodes);

			(results, condition)
		});

		self.rebind_state(repeat);
	}

	fn create_true_branch(
		&mut self,
		parent: &Weak<Mutex<Match>>,
		argument_count: u16,
	) -> Arc<Mutex<Branch>> {
		Branch::create(Weak::clone(parent), argument_count, |nodes, arguments| {
			self.rebind_state(arguments);
			self.create_repeat(nodes);

			self.capture_state(nodes)
		})
	}

	fn handle_block(&mut self, nodes: &mut Vec<Node>) {
		let condition = self.emit_condition(nodes);
		let arguments = self.capture_state(nodes);

		let match_id = Match::add_into(nodes, arguments, condition, |parent, argument_count| {
			let false_branch = self.create_false_branch(parent, argument_count);
			let true_branch = self.create_true_branch(parent, argument_count);

			vec![false_branch, true_branch]
		});

		self.rebind_state(match_id);
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

				Operator::Input => self.handle_input(nodes),
				Operator::Output => self.handle_output(nodes),

				Operator::Start => self.handle_block(nodes),
				Operator::End => return,
			}
		}
	}

	/// Compiles the given source code into a root function.
	pub fn run(&mut self, source: &str) -> Arc<Mutex<Function>> {
		self.operators.clear();
		self.operators.extend(
			source
				.chars()
				.rev()
				.filter_map(|character| Operator::try_from(character).ok()),
		);

		Function::create(1, |nodes, arguments| {
			self.io_state = Link(arguments, 0);

			self.create_memory(nodes);
			self.handle_code(nodes);

			vec![self.io_state]
		})
	}
}

impl Default for TuringMachineLifter {
	fn default() -> Self {
		Self::new()
	}
}
