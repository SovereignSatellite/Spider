use core::iter;

use list::resizable::Resizable;

use ir_graph::{Link, Node, operation, region::ValueType};
use web_assembly_graph::instruction::{
	Call, DataDrop, ElementsDrop, F32Constant, F64Constant, GlobalGet, GlobalSet, I32Constant,
	I64Constant, Instruction, IntegerBinaryOperation, IntegerCompareOperation,
	IntegerConvertToNumber, IntegerExtend, IntegerNarrow, IntegerTransmuteToNumber,
	IntegerUnaryOperation, IntegerWiden, LocalBranch, LocalSet, Location, MemoryCopy, MemoryFill,
	MemoryGrow, MemoryInit, MemoryLoad, MemorySize, MemoryStore, Name, NumberBinaryOperation,
	NumberCompareOperation, NumberNarrow, NumberTransmuteToInteger, NumberTruncateToInteger,
	NumberUnaryOperation, NumberWiden, RefFunction, RefIsNull, RefNull, TableCopy, TableFill,
	TableGet, TableGrow, TableInit, TableSet, TableSize,
};
use web_assembly_liveness::references::{Reference, ReferenceType};

use super::dependencies::DependencyMap;

const LOCAL_BASE: usize = Name::COUNT as usize;

pub struct BasicBlockLifter {
	locals: Vec<Link>,

	condition: Link,
	trap: Link,
	dependencies: DependencyMap,
}

impl BasicBlockLifter {
	pub const fn new() -> Self {
		Self {
			locals: Vec::new(),

			condition: Link::DANGLING,
			trap: Link::DANGLING,
			dependencies: DependencyMap::new(),
		}
	}

	pub const fn get_condition(&self) -> Link {
		self.condition
	}

	fn create_fence(&mut self, nodes: &mut Vec<Node>) {
		let mut sources = vec![self.trap];

		self.dependencies.get_mutable_into(&mut sources);

		let fence = operation::Fence::add_into(nodes, Resizable::Heap(sources));
		let mut fence = (0..u16::MAX).map(|port| Link(fence, port));

		self.trap = fence.next().unwrap();

		self.dependencies.set_mutable_from(fence);
	}

	pub fn get_function_outputs(&mut self, nodes: &mut Vec<Node>, results: usize) -> Vec<Link> {
		let mut results = self.locals[LOCAL_BASE..LOCAL_BASE + results].to_vec();

		self.create_fence(nodes);

		results.push(self.trap);

		results
	}

	pub fn set_function_inputs(
		&mut self,
		captures: u32,
		arguments: u32,
		argument_count: usize,
		dependencies: &[Reference],
	) {
		let mut captures = (0..u16::MAX).map(|port| Link(captures, port));

		self.dependencies.fill_keys(dependencies);
		self.dependencies.set_all_from(&mut captures);

		let reserved = iter::repeat_n(Link::DANGLING, LOCAL_BASE);
		let mut arguments = (0..u16::MAX).map(|port| Link(arguments, port));

		self.locals.clear();
		self.locals.extend(reserved);
		self.locals.extend(arguments.by_ref().take(argument_count));

		self.trap = arguments.next().unwrap();
	}

	pub fn set_local_types(&mut self, nodes: &mut Vec<Node>, types: &[ValueType]) {
		self.locals.extend(types.iter().map(|&local| match local {
			ValueType::I32 => Node::add_i32_into(nodes, 0),
			ValueType::I64 => Node::add_i64_into(nodes, 0),
			ValueType::F32 => Node::add_f32_into(nodes, 0.0),
			ValueType::F64 => Node::add_f64_into(nodes, 0.0),
			ValueType::Reference => Node::add_null_into(nodes),
		}));
	}

	pub fn set_stack_size(&mut self, nodes: &mut Vec<Node>, size: u16) {
		let null = Node::add_null_into(nodes);
		let count = usize::from(size).saturating_sub(self.locals.len());

		self.locals.extend(iter::repeat_n(null, count));
		self.locals[..LOCAL_BASE].fill(null);
	}

	pub fn get_active_bindings(&self, locals: &[u16]) -> Vec<Link> {
		let mut results = Vec::new();

		self.dependencies.get_all_into(&mut results);

		results.extend(
			locals
				.iter()
				.copied()
				.map(usize::from)
				.map(|local| self.locals[local]),
		);
		results.push(self.trap);

		results
	}

	pub fn set_active_bindings(&mut self, producer: u32, locals: &[u16]) {
		let mut producer = (0..u16::MAX).map(|port| Link(producer, port));

		self.dependencies.set_all_from(&mut producer);

		locals
			.iter()
			.copied()
			.map(usize::from)
			.zip(&mut producer)
			.for_each(|(local, link)| self.locals[local] = link);

		self.trap = producer.next().unwrap();
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

		let state = self.dependencies.get(ReferenceType::Function, function);

		self.locals[usize::from(destination)] = operation::MutableGet::add_into(nodes, state).0;
	}

	fn handle_unreachable(&mut self, nodes: &mut Vec<Node>) {
		self.trap = Node::add_trap_into(nodes);
	}

	fn handle_pre_call(&mut self, nodes: &mut Vec<Node>, from: u16, to: u16) -> Vec<Link> {
		let mut arguments = self.locals[usize::from(from)..usize::from(to)].to_vec();

		self.create_fence(nodes);

		arguments.push(self.trap);

		arguments
	}

	fn handle_post_call(&mut self, nodes: &mut Vec<Node>, call: u32, from: u16, to: u16) {
		let destinations = self.locals[usize::from(from)..usize::from(to)].iter_mut();
		let mut call = (0..u16::MAX).map(|port| Link(call, port));

		for (destination, result) in destinations.zip(&mut call) {
			*destination = result;
		}

		self.trap = call.next().unwrap();

		self.create_fence(nodes);
	}

	fn handle_call(&mut self, nodes: &mut Vec<Node>, instruction: Call) {
		let Call {
			destinations,
			sources,
			function,
		} = instruction;

		let arguments = self.handle_pre_call(nodes, sources.0, sources.1);

		let call = operation::Apply::add_into(
			nodes,
			self.locals[usize::from(function)],
			arguments,
			destinations.1 - destinations.0,
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
			signed,
			to,
			from,
		} = instruction;

		self.locals[usize::from(destination)] = operation::IntegerConvertToNumber::add_into(
			nodes,
			self.locals[usize::from(source)],
			signed,
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
			signed,
			saturate,
			to,
			from,
		} = instruction;

		self.locals[usize::from(destination)] = operation::NumberTruncateToInteger::add_into(
			nodes,
			self.locals[usize::from(source)],
			signed,
			saturate,
			to,
			from,
		);
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

	fn load_location(&self, kind: ReferenceType, location: Location) -> operation::Location {
		let Location { reference, offset } = location;

		operation::Location {
			reference: self.dependencies.get(kind, reference),
			offset: self.locals[usize::from(offset)],
		}
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

		let elements = self.load_location(ReferenceType::Elements, source);

		let (destination_state, source_state) = operation::TableCopy::add_into(
			nodes,
			self.load_location(ReferenceType::Table, destination),
			elements,
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

		let state = self.load_location(ReferenceType::Memory, source);
		let (result, state) = operation::MemoryLoad::add_into(nodes, state, kind);

		self.locals[usize::from(destination)] = result;

		self.dependencies
			.set(ReferenceType::Memory, source.reference, state);
	}

	fn handle_memory_store(&mut self, nodes: &mut Vec<Node>, instruction: MemoryStore) {
		let MemoryStore {
			destination,
			source,
			kind,
		} = instruction;

		let state = operation::MemoryStore::add_into(
			nodes,
			self.load_location(ReferenceType::Memory, destination),
			self.locals[usize::from(source)],
			kind,
		);

		self.dependencies
			.set(ReferenceType::Memory, destination.reference, state);
	}

	fn handle_memory_size(&mut self, nodes: &mut Vec<Node>, instruction: MemorySize) {
		let MemorySize {
			destination,
			memory,
		} = instruction;

		let state = self.dependencies.get(ReferenceType::Memory, memory);
		let (result, state) = operation::MemorySize::add_into(nodes, state);

		self.locals[usize::from(destination)] = result;

		self.dependencies.set(ReferenceType::Memory, memory, state);
	}

	fn handle_memory_grow(&mut self, nodes: &mut Vec<Node>, instruction: MemoryGrow) {
		let MemoryGrow {
			destination,
			memory,
			size,
		} = instruction;

		let (result, state) = operation::MemoryGrow::add_into(
			nodes,
			self.dependencies.get(ReferenceType::Memory, memory),
			self.locals[usize::from(size)],
		);

		self.locals[usize::from(destination)] = result;

		self.dependencies.set(ReferenceType::Memory, memory, state);
	}

	fn handle_memory_fill(&mut self, nodes: &mut Vec<Node>, instruction: MemoryFill) {
		let MemoryFill {
			destination,
			byte,
			size,
		} = instruction;

		let state = operation::MemoryFill::add_into(
			nodes,
			self.load_location(ReferenceType::Memory, destination),
			self.locals[usize::from(byte)],
			self.locals[usize::from(size)],
		);

		self.dependencies
			.set(ReferenceType::Memory, destination.reference, state);
	}

	fn handle_memory_copy(&mut self, nodes: &mut Vec<Node>, instruction: MemoryCopy) {
		let MemoryCopy {
			destination,
			source,
			size,
		} = instruction;

		let (destination_state, source_state) = operation::MemoryCopy::add_into(
			nodes,
			self.load_location(ReferenceType::Memory, destination),
			self.load_location(ReferenceType::Memory, source),
			self.locals[usize::from(size)],
		);

		self.dependencies.set(
			ReferenceType::Memory,
			destination.reference,
			destination_state,
		);

		self.dependencies
			.set(ReferenceType::Memory, source.reference, source_state);
	}

	fn handle_memory_init(&mut self, nodes: &mut Vec<Node>, instruction: MemoryInit) {
		let MemoryInit {
			destination,
			source,
			size,
		} = instruction;

		let (destination_state, source_state) = operation::MemoryCopy::add_into(
			nodes,
			self.load_location(ReferenceType::Memory, destination),
			self.load_location(ReferenceType::Data, source),
			self.locals[usize::from(size)],
		);

		self.dependencies.set(
			ReferenceType::Memory,
			destination.reference,
			destination_state,
		);

		self.dependencies
			.set(ReferenceType::Data, source.reference, source_state);
	}

	fn handle_data_drop(&mut self, nodes: &mut Vec<Node>, instruction: DataDrop) {
		let DataDrop { source } = instruction;

		let state = self.dependencies.get(ReferenceType::Data, source);
		let state = operation::MemoryDrop::add_into(nodes, state);

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
