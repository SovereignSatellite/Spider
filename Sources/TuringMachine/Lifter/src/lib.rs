//! Turing machine lifter that compiles source code into a data flow graph.

extern crate alloc;

use alloc::sync::{Arc, Weak};
use core::{mem, str::Bytes};

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

/// A lifter that compiles source code into a data flow graph.
pub struct TuringMachineLifter {
	loads: Resizable<Link, 4>,
	store: Link,
	offset: Link,

	io_state: Link,

	namespace: Arc<str>,
	input_identifier: Arc<str>,
	output_identifier: Arc<str>,
}

impl TuringMachineLifter {
	/// Creates a new Turing machine lifter.
	#[must_use]
	pub fn new() -> Self {
		Self {
			loads: Resizable::new(),
			store: Link::DANGLING,
			offset: Link::DANGLING,

			io_state: Link::DANGLING,

			namespace: Arc::from("turing"),
			input_identifier: Arc::from("ask"),
			output_identifier: Arc::from("tell"),
		}
	}

	fn create_memory(&mut self, nodes: &mut Vec<Node>) {
		self.loads.clear();

		let size = Node::add_i32_into(nodes, MEMORY_SIZE.try_into().unwrap());

		self.store = MemoryNew::add_into(nodes, Vec::new(), size);
		self.offset = Node::add_i32_into(nodes, 0);
	}

	fn reconcile_store(&mut self, nodes: &mut Vec<Node>) -> Link {
		let sources = mem::take(&mut self.loads);

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

	fn create_repeat(&mut self, nodes: &mut Vec<Node>, source: &mut Bytes<'_>) {
		let arguments = self.capture_state(nodes);

		let repeat = Repeat::add_into(nodes, arguments, |nodes, repeat_arguments| {
			self.rebind_state(repeat_arguments);
			self.handle_code(nodes, source);

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
		source: &mut Bytes<'_>,
	) -> Arc<Mutex<Branch>> {
		Branch::create(Weak::clone(parent), argument_count, |nodes, arguments| {
			self.rebind_state(arguments);
			self.create_repeat(nodes, source);

			self.capture_state(nodes)
		})
	}

	fn handle_block_unbounded(&mut self, nodes: &mut Vec<Node>, source: &mut Bytes<'_>) {
		stacker::maybe_grow(0x1_0000, 0x10_0000, || self.handle_block(nodes, source));
	}

	fn handle_block(&mut self, nodes: &mut Vec<Node>, source: &mut Bytes<'_>) {
		let condition = self.emit_condition(nodes);
		let arguments = self.capture_state(nodes);

		let match_id = Match::add_into(nodes, arguments, condition, |parent, argument_count| {
			let false_branch = self.create_false_branch(parent, argument_count);
			let true_branch = self.create_true_branch(parent, argument_count, source);

			vec![false_branch, true_branch]
		});

		self.rebind_state(match_id);
	}

	fn handle_code(&mut self, nodes: &mut Vec<Node>, source: &mut Bytes<'_>) {
		while let Some(operator) = source.next() {
			match operator {
				b'>' => {
					self.handle_offset_operation(nodes, integer::BinaryOperator::Add);
				}
				b'<' => {
					self.handle_offset_operation(nodes, integer::BinaryOperator::Subtract);
				}

				b'+' => {
					self.handle_memory_operation(nodes, integer::BinaryOperator::Add);
				}
				b'-' => {
					self.handle_memory_operation(nodes, integer::BinaryOperator::Subtract);
				}

				b',' => self.handle_input(nodes),
				b'.' => self.handle_output(nodes),

				b'[' => self.handle_block_unbounded(nodes, source),
				b']' => return,

				_ => {}
			}
		}
	}

	/// Compiles the given source code into a root function.
	#[must_use = "use the lifted root function"]
	pub fn run(&mut self, source: &str) -> Arc<Mutex<Function>> {
		let mut source = source.bytes();

		Function::create(1, |nodes, arguments| {
			self.io_state = Link(arguments, 0);

			self.create_memory(nodes);
			self.handle_code(nodes, &mut source);

			vec![self.io_state]
		})
	}
}

impl Default for TuringMachineLifter {
	fn default() -> Self {
		Self::new()
	}
}
