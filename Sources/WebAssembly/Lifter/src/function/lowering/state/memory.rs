use core::iter;

use ir_graph::{Link, Node, operation, region::Match};
use web_assembly_control_flow::instruction::{
	DataDrop, Location, MemoryCopy, MemoryFill, MemoryGrow, MemoryInit, MemoryLoad, MemorySize,
	MemoryStore, ReferenceType,
};

use super::LoweringState;
use crate::memory::{CONTENT_FIELD, MAXIMUM_FIELD, SIZE_FIELD};

#[derive(Clone, Copy)]
struct MemoryAccess {
	location: operation::Location,
	state: Link,
	content_state: Link,
}

impl LoweringState {
	fn fence_state<Effects>(nodes: &mut Vec<Node>, state: Link, effects: Effects) -> Link
	where
		Effects: IntoIterator<Item = Link>,
	{
		let sources = iter::once(state).chain(effects).collect();
		let fence = operation::Fence::add_into(nodes, sources);

		Link(fence, 0)
	}

	fn read_memory(nodes: &mut Vec<Node>, state: Link) -> (Link, Link) {
		let content = operation::Extract::add_into(nodes, state, CONTENT_FIELD);

		operation::MutableGet::add_into(nodes, content)
	}

	fn read_memory_size(nodes: &mut Vec<Node>, state: Link) -> (Link, Link) {
		let size = operation::Extract::add_into(nodes, state, SIZE_FIELD);

		operation::MutableGet::add_into(nodes, size)
	}

	fn read_memory_location(&self, nodes: &mut Vec<Node>, source: Location) -> MemoryAccess {
		let Location { reference, offset } = source;

		let state = self.dependencies.get(ReferenceType::Memory, reference);
		let (content, content_state) = Self::read_memory(nodes, state);

		let location = operation::Location {
			reference: content,
			offset: self.locals[usize::from(offset)],
		};

		MemoryAccess {
			location,
			state,
			content_state,
		}
	}

	fn read_data_location(&self, nodes: &mut Vec<Node>, source: Location) -> MemoryAccess {
		let Location { reference, offset } = source;

		let state = self.dependencies.get(ReferenceType::Data, reference);
		let (content, content_state) = operation::MutableGet::add_into(nodes, state);

		let location = operation::Location {
			reference: content,
			offset: self.locals[usize::from(offset)],
		};

		MemoryAccess {
			location,
			state,
			content_state,
		}
	}

	fn copy_memory(
		nodes: &mut Vec<Node>,
		destination: MemoryAccess,
		source: MemoryAccess,
		size: Link,
	) -> (Link, Link) {
		let (destination_content_state, source_content_state) =
			operation::MemoryCopy::add_into(nodes, destination.location, source.location, size);

		let destination_state = Self::fence_state(
			nodes,
			destination.state,
			[destination.content_state, destination_content_state],
		);

		let source_state = Self::fence_state(
			nodes,
			source.state,
			[source.content_state, source_content_state],
		);

		(destination_state, source_state)
	}

	pub fn handle_memory_load(&mut self, nodes: &mut Vec<Node>, instruction: MemoryLoad) {
		let MemoryLoad {
			destination,
			source,
			kind,
		} = instruction;

		let reference = source.reference;

		let source = self.read_memory_location(nodes, source);

		let (result, content_state) = operation::MemoryLoad::add_into(nodes, source.location, kind);

		let state = Self::fence_state(nodes, source.state, [source.content_state, content_state]);

		self.locals[usize::from(destination)] = result;

		self.dependencies
			.set(ReferenceType::Memory, reference, state);
	}

	pub fn handle_memory_store(&mut self, nodes: &mut Vec<Node>, instruction: MemoryStore) {
		let MemoryStore {
			destination,
			source,
			kind,
		} = instruction;

		let reference = destination.reference;

		let destination = self.read_memory_location(nodes, destination);

		let content_state = operation::MemoryStore::add_into(
			nodes,
			destination.location,
			self.locals[usize::from(source)],
			kind,
		);

		let state = Self::fence_state(
			nodes,
			destination.state,
			[destination.content_state, content_state],
		);

		self.dependencies
			.set(ReferenceType::Memory, reference, state);
	}

	pub fn handle_memory_size(&mut self, nodes: &mut Vec<Node>, instruction: MemorySize) {
		let MemorySize {
			destination,
			memory,
		} = instruction;

		let state = self.dependencies.get(ReferenceType::Memory, memory);
		let (size, size_state) = Self::read_memory_size(nodes, state);

		let state = Self::fence_state(nodes, state, [size_state]);

		self.locals[usize::from(destination)] = size;

		self.dependencies.set(ReferenceType::Memory, memory, state);
	}

	fn build_grow_failure(nodes: &mut Vec<Node>, arguments: u32) -> Vec<Link> {
		let state = Link(arguments, 0);
		let content_state = Link(arguments, 1);
		let size_state = Link(arguments, 2);

		let state = Self::fence_state(nodes, state, [content_state, size_state]);

		let failure = Node::add_i32_into(nodes, -1);

		vec![state, failure]
	}

	fn build_grow_success(nodes: &mut Vec<Node>, arguments: u32) -> Vec<Link> {
		let state = Link(arguments, 0);
		let content_state = Link(arguments, 1);
		let size_state = Link(arguments, 2);
		let old_content = Link(arguments, 3);
		let replacement = Link(arguments, 4);
		let new_size = Link(arguments, 5);
		let old_size = Link(arguments, 6);

		let zero = Node::add_i32_into(nodes, 0);

		let destination = operation::Location {
			reference: replacement,
			offset: zero,
		};
		let source = operation::Location {
			reference: old_content,
			offset: zero,
		};

		let (replacement_state, old_content_state) =
			operation::MemoryCopy::add_into(nodes, destination, source, old_size);
		let old_content_state = operation::MemoryDrop::add_into(nodes, old_content_state);

		let content_state = Self::fence_state(nodes, content_state, [old_content_state]);
		let content_state =
			operation::MutableSet::add_into(nodes, content_state, replacement_state);
		let size_state = operation::MutableSet::add_into(nodes, size_state, new_size);

		let state = Self::fence_state(nodes, state, [content_state, size_state]);

		vec![state, old_size]
	}

	fn build_grow_attempt(nodes: &mut Vec<Node>, arguments: u32) -> Vec<Link> {
		let state = Link(arguments, 0);
		let content_state = Link(arguments, 1);
		let size_state = Link(arguments, 2);
		let old_content = Link(arguments, 3);
		let old_size = Link(arguments, 4);
		let delta = Link(arguments, 5);

		let new_size = operation::integer::BinaryOperation::add_into(
			nodes,
			old_size,
			delta,
			operation::integer::Type::I32,
			operation::integer::BinaryOperator::Add,
		);

		let replacement = operation::MemoryNew::add_into(nodes, Vec::new(), new_size);
		let condition = operation::RefIsNull::add_into(nodes, replacement);

		let captures = vec![
			state,
			content_state,
			size_state,
			old_content,
			replacement,
			new_size,
			old_size,
		];

		let matcher = Match::add_if_into(
			nodes,
			captures,
			condition,
			Self::build_grow_success,
			Self::build_grow_failure,
		);

		vec![Link(matcher, 0), Link(matcher, 1)]
	}

	pub fn handle_memory_grow(&mut self, nodes: &mut Vec<Node>, instruction: MemoryGrow) {
		let MemoryGrow {
			destination,
			memory,
			size,
		} = instruction;

		let state = self.dependencies.get(ReferenceType::Memory, memory);
		let (content, content_state) = Self::read_memory(nodes, state);
		let (old_size, size_state) = Self::read_memory_size(nodes, state);
		let maximum = operation::Extract::add_into(nodes, state, MAXIMUM_FIELD);
		let delta = self.locals[usize::from(size)];

		// The byte sum can wrap `u32`, so growth is bounded by the headroom
		// instead, which is exact because the size never exceeds the maximum.
		let headroom = operation::integer::BinaryOperation::add_into(
			nodes,
			maximum,
			old_size,
			operation::integer::Type::I32,
			operation::integer::BinaryOperator::Subtract,
		);

		let condition = operation::integer::CompareOperation::add_into(
			nodes,
			headroom,
			delta,
			operation::integer::Type::I32,
			operation::integer::CompareOperator::LessThan { is_signed: false },
		);

		let arguments = vec![state, content_state, size_state, content, old_size, delta];

		let matcher = Match::add_if_into(
			nodes,
			arguments,
			condition,
			Self::build_grow_attempt,
			Self::build_grow_failure,
		);

		self.locals[usize::from(destination)] = Link(matcher, 1);

		self.dependencies
			.set(ReferenceType::Memory, memory, Link(matcher, 0));
	}

	pub fn handle_memory_fill(&mut self, nodes: &mut Vec<Node>, instruction: MemoryFill) {
		let MemoryFill {
			destination,
			byte,
			size,
		} = instruction;

		let reference = destination.reference;

		let destination = self.read_memory_location(nodes, destination);

		let content_state = operation::MemoryFill::add_into(
			nodes,
			destination.location,
			self.locals[usize::from(byte)],
			self.locals[usize::from(size)],
		);

		let state = Self::fence_state(
			nodes,
			destination.state,
			[destination.content_state, content_state],
		);

		self.dependencies
			.set(ReferenceType::Memory, reference, state);
	}

	pub fn handle_memory_copy(&mut self, nodes: &mut Vec<Node>, instruction: MemoryCopy) {
		let MemoryCopy {
			destination,
			source,
			size,
		} = instruction;

		let destination_reference = destination.reference;
		let source_reference = source.reference;

		let destination = self.read_memory_location(nodes, destination);
		let source = self.read_memory_location(nodes, source);

		let (destination_state, source_state) =
			Self::copy_memory(nodes, destination, source, self.locals[usize::from(size)]);

		self.dependencies.set(
			ReferenceType::Memory,
			destination_reference,
			destination_state,
		);

		self.dependencies
			.set(ReferenceType::Memory, source_reference, source_state);
	}

	pub fn handle_memory_init(&mut self, nodes: &mut Vec<Node>, instruction: MemoryInit) {
		let MemoryInit {
			destination,
			source,
			size,
		} = instruction;

		let destination_reference = destination.reference;
		let source_reference = source.reference;

		let destination = self.read_memory_location(nodes, destination);
		let source = self.read_data_location(nodes, source);

		let (destination_state, source_state) =
			Self::copy_memory(nodes, destination, source, self.locals[usize::from(size)]);

		self.dependencies.set(
			ReferenceType::Memory,
			destination_reference,
			destination_state,
		);

		self.dependencies
			.set(ReferenceType::Data, source_reference, source_state);
	}

	pub fn handle_data_drop(&mut self, nodes: &mut Vec<Node>, instruction: DataDrop) {
		let DataDrop { source } = instruction;

		let state = self.dependencies.get(ReferenceType::Data, source);
		let (content, state) = operation::MutableGet::add_into(nodes, state);

		let content_state = operation::MemoryDrop::add_into(nodes, content);
		let state = Self::fence_state(nodes, state, [content_state]);

		let zero = Node::add_i32_into(nodes, 0);
		let empty = operation::MemoryNew::add_into(nodes, Vec::new(), zero);

		let state = operation::MutableSet::add_into(nodes, state, empty);

		self.dependencies.set(ReferenceType::Data, source, state);
	}
}
