use core::iter;

use list::resizable::Resizable;

use ir_graph::{Link, Node, operation, region::Match};
use web_assembly_graph::instruction::{
	Call, DataDrop, ElementsDrop, F32Constant, F64Constant, GlobalGet, GlobalSet, I32Constant,
	I64Constant, Instruction, IntegerBinaryOperation, IntegerCompareOperation,
	IntegerConvertToNumber, IntegerExtend, IntegerNarrow, IntegerTransmuteToNumber,
	IntegerUnaryOperation, IntegerWiden, LocalBranch, LocalSet, Location, MemoryCopy, MemoryFill,
	MemoryGrow, MemoryInit, MemoryLoad, MemorySize, MemoryStore, Name, NumberBinaryOperation,
	NumberCompareOperation, NumberNarrow, NumberTransmuteToInteger, NumberTruncateToInteger,
	NumberUnaryOperation, NumberWiden, RefFunction, RefIsNull, RefNull, Reference, ReferenceType,
	TableCopy, TableFill, TableGet, TableGrow, TableInit, TableSet, TableSize,
};

use super::{
	MEMORY_CONTENT_FIELD, MEMORY_MAXIMUM_FIELD, MEMORY_SIZE_FIELD, dependencies::DependencyMap,
	function::LocalKind,
};

const LOCAL_BASE: usize = Name::COUNT as usize;

fn to_port(index: usize) -> u16 {
	let Ok(port) = index.try_into() else {
		unreachable!()
	};

	port
}

pub struct FunctionHeader<'function> {
	pub arguments: u32,
	pub argument_count: usize,
	pub result_count: usize,
	pub local_kinds: &'function [LocalKind],
	pub stack_size: u16,
	pub dependencies: &'function [Reference],
}

#[derive(Clone, Copy)]
struct MemoryAccess {
	location: operation::Location,
	state: Link,
	content_state: Link,
}

pub struct SlotFile {
	locals: Vec<Link>,

	condition: Link,
	trap: Link,
	dependencies: DependencyMap,
}

impl SlotFile {
	pub const fn new() -> Self {
		Self {
			locals: Vec::new(),

			condition: Link::DANGLING,
			trap: Link::DANGLING,
			dependencies: DependencyMap::new(),
		}
	}

	pub const fn condition(&self) -> Link {
		self.condition
	}

	fn create_fence(&mut self, nodes: &mut Vec<Node>) {
		let mut sources = vec![self.trap];

		self.dependencies.get_mutable_into(&mut sources);

		let source_count = to_port(sources.len());
		let fence = operation::Fence::add_into(nodes, Resizable::Heap(sources));

		self.trap = Link(fence, 0);

		self.dependencies
			.set_mutable_from((1..source_count).map(|port| Link(fence, port)));
	}

	pub fn capture_outputs(&mut self, nodes: &mut Vec<Node>, result_count: usize) -> Vec<Link> {
		let mut results = self.locals[LOCAL_BASE..LOCAL_BASE + result_count].to_vec();

		self.create_fence(nodes);

		results.push(self.trap);

		results
	}

	fn seed_dependencies_from_closure(&mut self, nodes: &mut Vec<Node>, closure: Link) {
		let Ok(count) = u32::try_from(self.dependencies.count()) else {
			unreachable!()
		};
		let extracts = (1..=count).map(|port| operation::Extract::add_into(nodes, closure, port));

		self.dependencies.set_all_from(extracts);
	}

	fn seed_declared_locals(&mut self, nodes: &mut Vec<Node>, kinds: &[LocalKind]) {
		self.locals.extend(kinds.iter().map(|&local| match local {
			LocalKind::I32 => Node::add_i32_into(nodes, 0),
			LocalKind::I64 => Node::add_i64_into(nodes, 0),
			LocalKind::F32 => Node::add_f32_into(nodes, 0.0),
			LocalKind::F64 => Node::add_f64_into(nodes, 0.0),
			LocalKind::Reference => Node::add_null_into(nodes),
		}));
	}

	// The slot layout is [scratch | parameters | declared locals | stack]; the
	// trap rides the argument port directly after the parameters.
	pub fn seed(&mut self, nodes: &mut Vec<Node>, header: &FunctionHeader<'_>) {
		let &FunctionHeader {
			arguments,
			argument_count,
			local_kinds,
			stack_size,
			dependencies,
			..
		} = header;

		self.dependencies.fill_keys(dependencies);
		self.seed_dependencies_from_closure(nodes, Link(arguments, 0));

		let null = Node::add_null_into(nodes);
		let zero = Node::add_i32_into(nodes, 0);
		let parameters = (1..=to_port(argument_count)).map(|port| Link(arguments, port));

		self.locals.clear();
		self.locals.extend(iter::repeat_n(zero, LOCAL_BASE));
		self.locals.extend(parameters);
		self.seed_declared_locals(nodes, local_kinds);

		let padding = usize::from(stack_size).saturating_sub(self.locals.len());

		self.locals.extend(iter::repeat_n(null, padding));

		self.trap = Link(arguments, to_port(argument_count + 1));
	}

	pub fn capture_bindings(&self, live: &[u16]) -> Vec<Link> {
		let mut links = Vec::new();

		self.dependencies.get_all_into(&mut links);

		links.extend(live.iter().map(|&slot| self.locals[usize::from(slot)]));
		links.push(self.trap);

		links
	}

	pub fn rebind_bindings(&mut self, producer: u32, live: &[u16]) {
		let dependency_count = to_port(self.dependencies.count());

		self.dependencies
			.set_all_from((0..dependency_count).map(|port| Link(producer, port)));

		for (offset, &slot) in (0..).zip(live) {
			self.locals[usize::from(slot)] = Link(producer, dependency_count + offset);
		}

		self.trap = Link(producer, dependency_count + to_port(live.len()));
	}

	fn handle_local_set(&mut self, instruction: LocalSet) {
		let LocalSet {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] = self.locals[usize::from(source)];
	}

	fn handle_local_branch(&mut self, instruction: LocalBranch) {
		let LocalBranch { source } = instruction;

		self.condition = self.locals[usize::from(source)];
	}

	fn handle_i32_constant(&mut self, nodes: &mut Vec<Node>, instruction: I32Constant) {
		let I32Constant { destination, data } = instruction;

		self.locals[usize::from(destination)] = Node::add_i32_into(nodes, data);
	}

	fn handle_i64_constant(&mut self, nodes: &mut Vec<Node>, instruction: I64Constant) {
		let I64Constant { destination, data } = instruction;

		self.locals[usize::from(destination)] = Node::add_i64_into(nodes, data);
	}

	fn handle_f32_constant(&mut self, nodes: &mut Vec<Node>, instruction: F32Constant) {
		let F32Constant { destination, data } = instruction;

		self.locals[usize::from(destination)] = Node::add_f32_into(nodes, data);
	}

	fn handle_f64_constant(&mut self, nodes: &mut Vec<Node>, instruction: F64Constant) {
		let F64Constant { destination, data } = instruction;

		self.locals[usize::from(destination)] = Node::add_f64_into(nodes, data);
	}

	fn handle_ref_is_null(&mut self, nodes: &mut Vec<Node>, instruction: RefIsNull) {
		let RefIsNull {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] =
			operation::RefIsNull::add_into(nodes, self.locals[usize::from(source)]);
	}

	fn handle_ref_null(&mut self, nodes: &mut Vec<Node>, instruction: RefNull) {
		let RefNull { destination } = instruction;

		self.locals[usize::from(destination)] = Node::add_null_into(nodes);
	}

	fn handle_ref_function(&mut self, nodes: &mut Vec<Node>, instruction: RefFunction) {
		let RefFunction {
			destination,
			function,
		} = instruction;

		let slot = self.dependencies.get(ReferenceType::Function, function);

		self.locals[usize::from(destination)] = operation::MutableGet::add_into(nodes, slot).0;
	}

	fn handle_unreachable(&mut self, nodes: &mut Vec<Node>) {
		self.trap = Node::add_trap_into(nodes);
	}

	fn handle_pre_call(
		&mut self,
		nodes: &mut Vec<Node>,
		closure: Link,
		from: u16,
		to: u16,
	) -> Vec<Link> {
		let mut arguments = Vec::with_capacity(usize::from(to - from) + 2);

		arguments.push(closure);
		arguments.extend_from_slice(&self.locals[usize::from(from)..usize::from(to)]);

		self.create_fence(nodes);

		arguments.push(self.trap);

		arguments
	}

	fn handle_post_call(&mut self, nodes: &mut Vec<Node>, call: u32, from: u16, to: u16) {
		let destinations = self.locals[usize::from(from)..usize::from(to)].iter_mut();

		for (port, destination) in (0..).zip(destinations) {
			*destination = Link(call, port);
		}

		self.trap = Link(call, to - from);

		self.create_fence(nodes);
	}

	fn handle_call(&mut self, nodes: &mut Vec<Node>, instruction: Call) {
		let Call {
			destinations,
			sources,
			function: callee,
		} = instruction;

		let closure = self.locals[usize::from(callee)];
		let function = operation::Extract::add_into(nodes, closure, 0);

		let arguments = self.handle_pre_call(nodes, closure, sources.0, sources.1);

		let call = operation::Apply::add_into(
			nodes,
			function,
			arguments,
			destinations.1 - destinations.0 + 1,
		);

		self.handle_post_call(nodes, call, destinations.0, destinations.1);
	}

	fn handle_integer_unary_operation(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: IntegerUnaryOperation,
	) {
		let IntegerUnaryOperation {
			destination,
			source,
			kind,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = operation::integer::UnaryOperation::add_into(
			nodes,
			self.locals[usize::from(source)],
			kind,
			operator,
		);
	}

	fn handle_integer_binary_operation(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: IntegerBinaryOperation,
	) {
		let IntegerBinaryOperation {
			destination,
			lhs,
			rhs,
			kind,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = operation::integer::BinaryOperation::add_into(
			nodes,
			self.locals[usize::from(lhs)],
			self.locals[usize::from(rhs)],
			kind,
			operator,
		);
	}

	fn handle_trapping_integer_binary_operation(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: IntegerBinaryOperation,
	) {
		let IntegerBinaryOperation {
			destination,
			lhs,
			rhs,
			kind,
			operator,
		} = instruction;

		let result = operation::integer::BinaryOperation::add_into(
			nodes,
			self.locals[usize::from(lhs)],
			self.locals[usize::from(rhs)],
			kind,
			operator,
		);
		let fence = operation::Fence::add_into(nodes, list::resizable![self.trap, result]);

		self.trap = Link(fence, 0);
		self.locals[usize::from(destination)] = result;
	}

	fn handle_integer_compare_operation(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: IntegerCompareOperation,
	) {
		let IntegerCompareOperation {
			destination,
			lhs,
			rhs,
			kind,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = operation::integer::CompareOperation::add_into(
			nodes,
			self.locals[usize::from(lhs)],
			self.locals[usize::from(rhs)],
			kind,
			operator,
		);
	}

	fn handle_integer_narrow(&mut self, nodes: &mut Vec<Node>, instruction: IntegerNarrow) {
		let IntegerNarrow {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] =
			operation::IntegerNarrow::add_into(nodes, self.locals[usize::from(source)]);
	}

	fn handle_integer_widen(&mut self, nodes: &mut Vec<Node>, instruction: IntegerWiden) {
		let IntegerWiden {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] =
			operation::IntegerWiden::add_into(nodes, self.locals[usize::from(source)]);
	}

	fn handle_integer_extend(&mut self, nodes: &mut Vec<Node>, instruction: IntegerExtend) {
		let IntegerExtend {
			destination,
			source,
			kind,
		} = instruction;

		self.locals[usize::from(destination)] =
			operation::IntegerSignExtend::add_into(nodes, self.locals[usize::from(source)], kind);
	}

	fn handle_integer_convert_to_number(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: IntegerConvertToNumber,
	) {
		let IntegerConvertToNumber {
			destination,
			source,
			is_signed,
			to,
			from,
		} = instruction;

		self.locals[usize::from(destination)] = operation::IntegerConvertToNumber::add_into(
			nodes,
			self.locals[usize::from(source)],
			is_signed,
			to,
			from,
		);
	}

	fn handle_integer_transmute_to_number(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: IntegerTransmuteToNumber,
	) {
		let IntegerTransmuteToNumber {
			destination,
			source,
			from,
		} = instruction;

		self.locals[usize::from(destination)] = operation::IntegerTransmuteToNumber::add_into(
			nodes,
			self.locals[usize::from(source)],
			from,
		);
	}

	fn handle_number_unary_operation(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: NumberUnaryOperation,
	) {
		let NumberUnaryOperation {
			destination,
			source,
			kind,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = operation::number::UnaryOperation::add_into(
			nodes,
			self.locals[usize::from(source)],
			kind,
			operator,
		);
	}

	fn handle_number_binary_operation(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: NumberBinaryOperation,
	) {
		let NumberBinaryOperation {
			destination,
			lhs,
			rhs,
			kind,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = operation::number::BinaryOperation::add_into(
			nodes,
			self.locals[usize::from(lhs)],
			self.locals[usize::from(rhs)],
			kind,
			operator,
		);
	}

	fn handle_number_compare_operation(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: NumberCompareOperation,
	) {
		let NumberCompareOperation {
			destination,
			lhs,
			rhs,
			kind,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = operation::number::CompareOperation::add_into(
			nodes,
			self.locals[usize::from(lhs)],
			self.locals[usize::from(rhs)],
			kind,
			operator,
		);
	}

	fn handle_number_narrow(&mut self, nodes: &mut Vec<Node>, instruction: NumberNarrow) {
		let NumberNarrow {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] =
			operation::NumberNarrow::add_into(nodes, self.locals[usize::from(source)]);
	}

	fn handle_number_widen(&mut self, nodes: &mut Vec<Node>, instruction: NumberWiden) {
		let NumberWiden {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] =
			operation::NumberWiden::add_into(nodes, self.locals[usize::from(source)]);
	}

	fn handle_number_truncate_to_integer(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: NumberTruncateToInteger,
	) {
		let NumberTruncateToInteger {
			destination,
			source,
			is_signed,
			is_saturating,
			to,
			from,
		} = instruction;

		self.locals[usize::from(destination)] = operation::NumberTruncateToInteger::add_into(
			nodes,
			self.locals[usize::from(source)],
			is_signed,
			is_saturating,
			to,
			from,
		);
	}

	fn handle_trapping_number_truncate_to_integer(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: NumberTruncateToInteger,
	) {
		let NumberTruncateToInteger {
			destination,
			source,
			is_signed,
			is_saturating,
			to,
			from,
		} = instruction;

		let result = operation::NumberTruncateToInteger::add_into(
			nodes,
			self.locals[usize::from(source)],
			is_signed,
			is_saturating,
			to,
			from,
		);
		let fence = operation::Fence::add_into(nodes, list::resizable![self.trap, result]);

		self.trap = Link(fence, 0);
		self.locals[usize::from(destination)] = result;
	}

	fn handle_number_transmute_to_integer(
		&mut self,
		nodes: &mut Vec<Node>,
		instruction: NumberTransmuteToInteger,
	) {
		let NumberTransmuteToInteger {
			destination,
			source,
			from,
		} = instruction;

		self.locals[usize::from(destination)] = operation::NumberTransmuteToInteger::add_into(
			nodes,
			self.locals[usize::from(source)],
			from,
		);
	}

	fn handle_global_get(&mut self, nodes: &mut Vec<Node>, instruction: GlobalGet) {
		let GlobalGet {
			destination,
			source,
		} = instruction;

		let state = self.dependencies.get(ReferenceType::Global, source);
		let (result, state) = operation::MutableGet::add_into(nodes, state);

		self.locals[usize::from(destination)] = result;

		self.dependencies.set(ReferenceType::Global, source, state);
	}

	fn handle_global_set(&mut self, nodes: &mut Vec<Node>, instruction: GlobalSet) {
		let GlobalSet {
			destination,
			source,
		} = instruction;

		let state = operation::MutableSet::add_into(
			nodes,
			self.dependencies.get(ReferenceType::Global, destination),
			self.locals[usize::from(source)],
		);

		self.dependencies
			.set(ReferenceType::Global, destination, state);
	}

	fn fence_state<I>(nodes: &mut Vec<Node>, state: Link, effects: I) -> Link
	where
		I: IntoIterator<Item = Link>,
	{
		let sources = iter::once(state).chain(effects).collect();
		let fence = operation::Fence::add_into(nodes, sources);

		Link(fence, 0)
	}

	fn load_location(&self, kind: ReferenceType, location: Location) -> operation::Location {
		let Location { reference, offset } = location;

		operation::Location {
			reference: self.dependencies.get(kind, reference),
			offset: self.locals[usize::from(offset)],
		}
	}

	fn read_memory(nodes: &mut Vec<Node>, state: Link) -> (Link, Link) {
		let content = operation::Extract::add_into(nodes, state, MEMORY_CONTENT_FIELD);

		operation::MutableGet::add_into(nodes, content)
	}

	fn read_memory_size(nodes: &mut Vec<Node>, state: Link) -> (Link, Link) {
		let size = operation::Extract::add_into(nodes, state, MEMORY_SIZE_FIELD);

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

	fn handle_table_get(&mut self, nodes: &mut Vec<Node>, instruction: TableGet) {
		let TableGet {
			destination,
			source,
		} = instruction;

		let state = self.load_location(ReferenceType::Table, source);
		let (result, state) = operation::TableGet::add_into(nodes, state);

		self.locals[usize::from(destination)] = result;

		self.dependencies
			.set(ReferenceType::Table, source.reference, state);
	}

	fn handle_table_set(&mut self, nodes: &mut Vec<Node>, instruction: TableSet) {
		let TableSet {
			destination,
			source,
		} = instruction;

		let state = operation::TableSet::add_into(
			nodes,
			self.load_location(ReferenceType::Table, destination),
			self.locals[usize::from(source)],
		);

		self.dependencies
			.set(ReferenceType::Table, destination.reference, state);
	}

	fn handle_table_size(&mut self, nodes: &mut Vec<Node>, instruction: TableSize) {
		let TableSize { destination, table } = instruction;

		let state = self.dependencies.get(ReferenceType::Table, table);
		let (result, state) = operation::TableSize::add_into(nodes, state);

		self.locals[usize::from(destination)] = result;

		self.dependencies.set(ReferenceType::Table, table, state);
	}

	fn handle_table_grow(&mut self, nodes: &mut Vec<Node>, instruction: TableGrow) {
		let TableGrow {
			destination,
			table,
			size,
			initializer,
		} = instruction;

		let (result, state) = operation::TableGrow::add_into(
			nodes,
			self.dependencies.get(ReferenceType::Table, table),
			self.locals[usize::from(initializer)],
			self.locals[usize::from(size)],
		);

		self.locals[usize::from(destination)] = result;

		self.dependencies.set(ReferenceType::Table, table, state);
	}

	fn handle_table_fill(&mut self, nodes: &mut Vec<Node>, instruction: TableFill) {
		let TableFill {
			destination,
			source,
			size,
		} = instruction;

		let state = operation::TableFill::add_into(
			nodes,
			self.load_location(ReferenceType::Table, destination),
			self.locals[usize::from(source)],
			self.locals[usize::from(size)],
		);

		self.dependencies
			.set(ReferenceType::Table, destination.reference, state);
	}

	fn handle_table_copy(&mut self, nodes: &mut Vec<Node>, instruction: TableCopy) {
		let TableCopy {
			destination,
			source,
			size,
		} = instruction;

		let (destination_state, source_state) = operation::TableCopy::add_into(
			nodes,
			self.load_location(ReferenceType::Table, destination),
			self.load_location(ReferenceType::Table, source),
			self.locals[usize::from(size)],
		);

		self.dependencies.set(
			ReferenceType::Table,
			destination.reference,
			destination_state,
		);

		self.dependencies
			.set(ReferenceType::Table, source.reference, source_state);
	}

	fn handle_table_init(&mut self, nodes: &mut Vec<Node>, instruction: TableInit) {
		let TableInit {
			destination,
			source,
			size,
		} = instruction;

		let (destination_state, source_state) = operation::TableCopy::add_into(
			nodes,
			self.load_location(ReferenceType::Table, destination),
			self.load_location(ReferenceType::Elements, source),
			self.locals[usize::from(size)],
		);

		self.dependencies.set(
			ReferenceType::Table,
			destination.reference,
			destination_state,
		);

		self.dependencies
			.set(ReferenceType::Elements, source.reference, source_state);
	}

	fn handle_elements_drop(&mut self, nodes: &mut Vec<Node>, instruction: ElementsDrop) {
		let ElementsDrop { source } = instruction;

		let state = self.dependencies.get(ReferenceType::Elements, source);
		let state = operation::TableDrop::add_into(nodes, state);

		self.dependencies
			.set(ReferenceType::Elements, source, state);
	}

	fn handle_memory_load(&mut self, nodes: &mut Vec<Node>, instruction: MemoryLoad) {
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

	fn handle_memory_store(&mut self, nodes: &mut Vec<Node>, instruction: MemoryStore) {
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

	fn handle_memory_size(&mut self, nodes: &mut Vec<Node>, instruction: MemorySize) {
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

	fn handle_memory_grow(&mut self, nodes: &mut Vec<Node>, instruction: MemoryGrow) {
		let MemoryGrow {
			destination,
			memory,
			size,
		} = instruction;

		let state = self.dependencies.get(ReferenceType::Memory, memory);
		let (content, content_state) = Self::read_memory(nodes, state);
		let (old_size, size_state) = Self::read_memory_size(nodes, state);
		let maximum = operation::Extract::add_into(nodes, state, MEMORY_MAXIMUM_FIELD);
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

	fn handle_memory_fill(&mut self, nodes: &mut Vec<Node>, instruction: MemoryFill) {
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

	fn handle_memory_copy(&mut self, nodes: &mut Vec<Node>, instruction: MemoryCopy) {
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

	fn handle_memory_init(&mut self, nodes: &mut Vec<Node>, instruction: MemoryInit) {
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

	fn handle_data_drop(&mut self, nodes: &mut Vec<Node>, instruction: DataDrop) {
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

	#[expect(
		clippy::too_many_lines,
		reason = "exhaustive match over instruction variants"
	)]
	fn handle_instruction(&mut self, nodes: &mut Vec<Node>, instruction: Instruction) {
		match instruction {
			Instruction::LocalSet(instruction) => self.handle_local_set(instruction),
			Instruction::LocalBranch(instruction) => self.handle_local_branch(instruction),
			Instruction::I32Constant(instruction) => self.handle_i32_constant(nodes, instruction),
			Instruction::I64Constant(instruction) => self.handle_i64_constant(nodes, instruction),
			Instruction::F32Constant(instruction) => self.handle_f32_constant(nodes, instruction),
			Instruction::F64Constant(instruction) => self.handle_f64_constant(nodes, instruction),
			Instruction::RefIsNull(instruction) => self.handle_ref_is_null(nodes, instruction),
			Instruction::RefNull(instruction) => self.handle_ref_null(nodes, instruction),
			Instruction::RefFunction(instruction) => self.handle_ref_function(nodes, instruction),
			Instruction::Call(instruction) => self.handle_call(nodes, instruction),
			Instruction::Unreachable => self.handle_unreachable(nodes),
			Instruction::IntegerUnaryOperation(instruction) => {
				self.handle_integer_unary_operation(nodes, instruction);
			}
			Instruction::IntegerBinaryOperation(
				instruction @ IntegerBinaryOperation {
					operator:
						operation::integer::BinaryOperator::Divide { .. }
						| operation::integer::BinaryOperator::Remainder { .. },
					..
				},
			) => self.handle_trapping_integer_binary_operation(nodes, instruction),
			Instruction::IntegerBinaryOperation(instruction) => {
				self.handle_integer_binary_operation(nodes, instruction);
			}
			Instruction::IntegerCompareOperation(instruction) => {
				self.handle_integer_compare_operation(nodes, instruction);
			}
			Instruction::IntegerNarrow(instruction) => {
				self.handle_integer_narrow(nodes, instruction);
			}
			Instruction::IntegerWiden(instruction) => self.handle_integer_widen(nodes, instruction),
			Instruction::IntegerExtend(instruction) => {
				self.handle_integer_extend(nodes, instruction);
			}
			Instruction::IntegerConvertToNumber(instruction) => {
				self.handle_integer_convert_to_number(nodes, instruction);
			}
			Instruction::IntegerTransmuteToNumber(instruction) => {
				self.handle_integer_transmute_to_number(nodes, instruction);
			}
			Instruction::NumberUnaryOperation(instruction) => {
				self.handle_number_unary_operation(nodes, instruction);
			}
			Instruction::NumberBinaryOperation(instruction) => {
				self.handle_number_binary_operation(nodes, instruction);
			}
			Instruction::NumberCompareOperation(instruction) => {
				self.handle_number_compare_operation(nodes, instruction);
			}
			Instruction::NumberNarrow(instruction) => self.handle_number_narrow(nodes, instruction),
			Instruction::NumberWiden(instruction) => self.handle_number_widen(nodes, instruction),
			Instruction::NumberTruncateToInteger(
				instruction @ NumberTruncateToInteger {
					is_saturating: false,
					..
				},
			) => self.handle_trapping_number_truncate_to_integer(nodes, instruction),
			Instruction::NumberTruncateToInteger(instruction) => {
				self.handle_number_truncate_to_integer(nodes, instruction);
			}
			Instruction::NumberTransmuteToInteger(instruction) => {
				self.handle_number_transmute_to_integer(nodes, instruction);
			}
			Instruction::GlobalGet(instruction) => self.handle_global_get(nodes, instruction),
			Instruction::GlobalSet(instruction) => self.handle_global_set(nodes, instruction),
			Instruction::TableGet(instruction) => self.handle_table_get(nodes, instruction),
			Instruction::TableSet(instruction) => self.handle_table_set(nodes, instruction),
			Instruction::TableSize(instruction) => self.handle_table_size(nodes, instruction),
			Instruction::TableGrow(instruction) => self.handle_table_grow(nodes, instruction),
			Instruction::TableFill(instruction) => self.handle_table_fill(nodes, instruction),
			Instruction::TableCopy(instruction) => self.handle_table_copy(nodes, instruction),
			Instruction::TableInit(instruction) => self.handle_table_init(nodes, instruction),
			Instruction::ElementsDrop(instruction) => self.handle_elements_drop(nodes, instruction),
			Instruction::MemoryLoad(instruction) => self.handle_memory_load(nodes, instruction),
			Instruction::MemoryStore(instruction) => self.handle_memory_store(nodes, instruction),
			Instruction::MemorySize(instruction) => self.handle_memory_size(nodes, instruction),
			Instruction::MemoryGrow(instruction) => self.handle_memory_grow(nodes, instruction),
			Instruction::MemoryFill(instruction) => self.handle_memory_fill(nodes, instruction),
			Instruction::MemoryCopy(instruction) => self.handle_memory_copy(nodes, instruction),
			Instruction::MemoryInit(instruction) => self.handle_memory_init(nodes, instruction),
			Instruction::DataDrop(instruction) => self.handle_data_drop(nodes, instruction),
		}
	}

	pub fn run(&mut self, nodes: &mut Vec<Node>, instructions: &[Instruction]) {
		for &instruction in instructions {
			self.handle_instruction(nodes, instruction);
		}
	}
}
