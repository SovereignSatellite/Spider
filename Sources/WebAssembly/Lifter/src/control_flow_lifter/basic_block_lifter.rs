use alloc::vec::Vec;
use ir_graph::{DataFlowGraph, Link, Node, control::ValueType, simple};
use list::resizable::Resizable;
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

use super::dependency_map::DependencyMap;

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

	fn create_fence(&mut self, graph: &mut DataFlowGraph) {
		let mut sources = alloc::vec![self.trap];

		self.dependencies.get_mutable_into(&mut sources);

		let fence = simple::Fence::add_into(graph, Resizable::Heap(sources));
		let mut fence = (0..u16::MAX).map(|port| Link(fence, port));

		self.trap = fence.next().unwrap();

		self.dependencies.set_mutable_from(fence);
	}

	pub fn get_function_outputs(&mut self, graph: &mut DataFlowGraph, results: usize) -> Vec<Link> {
		let mut results = self.locals[LOCAL_BASE..LOCAL_BASE + results].to_vec();

		self.create_fence(graph);

		results.push(self.trap);

		results
	}

	pub fn set_function_inputs(
		&mut self,
		lambda_in: u32,
		arguments: usize,
		dependencies: &[Reference],
	) {
		let mut inputs = (0..u16::MAX).map(|port| Link(lambda_in, port));

		self.dependencies.fill_keys(dependencies);
		self.dependencies.set_all_from(&mut inputs);

		let reserved = core::iter::repeat_n(Link::DANGLING, LOCAL_BASE);

		self.locals.clear();
		self.locals.extend(reserved);
		self.locals.extend(inputs.by_ref().take(arguments));

		self.trap = inputs.next().unwrap();
	}

	pub fn set_local_types(&mut self, graph: &mut DataFlowGraph, types: &[ValueType]) {
		self.locals.extend(types.iter().map(|&local| match local {
			ValueType::I32 => Node::add_i32_into(graph, 0),
			ValueType::I64 => Node::add_i64_into(graph, 0),
			ValueType::F32 => Node::add_f32_into(graph, 0.0),
			ValueType::F64 => Node::add_f64_into(graph, 0.0),
			ValueType::Reference => Node::add_null_into(graph),
		}));
	}

	pub fn set_stack_size(&mut self, graph: &mut DataFlowGraph, size: u16) {
		let null = Node::add_null_into(graph);
		let count = usize::from(size).saturating_sub(self.locals.len());

		self.locals.extend(core::iter::repeat_n(null, count));
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

	fn handle_i32_constant(&mut self, graph: &mut DataFlowGraph, instruction: I32Constant) {
		let I32Constant { destination, data } = instruction;

		self.locals[usize::from(destination)] = Node::add_i32_into(graph, data);
	}

	fn handle_i64_constant(&mut self, graph: &mut DataFlowGraph, instruction: I64Constant) {
		let I64Constant { destination, data } = instruction;

		self.locals[usize::from(destination)] = Node::add_i64_into(graph, data);
	}

	fn handle_f32_constant(&mut self, graph: &mut DataFlowGraph, instruction: F32Constant) {
		let F32Constant { destination, data } = instruction;

		self.locals[usize::from(destination)] = Node::add_f32_into(graph, data);
	}

	fn handle_f64_constant(&mut self, graph: &mut DataFlowGraph, instruction: F64Constant) {
		let F64Constant { destination, data } = instruction;

		self.locals[usize::from(destination)] = Node::add_f64_into(graph, data);
	}

	fn handle_ref_is_null(&mut self, graph: &mut DataFlowGraph, instruction: RefIsNull) {
		let RefIsNull {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] =
			simple::RefIsNull::add_into(graph, self.locals[usize::from(source)]);
	}

	fn handle_ref_null(&mut self, graph: &mut DataFlowGraph, instruction: RefNull) {
		let RefNull { destination } = instruction;

		self.locals[usize::from(destination)] = Node::add_null_into(graph);
	}

	fn handle_ref_function(&mut self, graph: &mut DataFlowGraph, instruction: RefFunction) {
		let RefFunction {
			destination,
			function,
		} = instruction;

		let state = self.dependencies.get(ReferenceType::Function, function);

		self.locals[usize::from(destination)] = simple::GlobalGet::add_into(graph, state).0;
	}

	fn handle_unreachable(&mut self, graph: &mut DataFlowGraph) {
		self.trap = Node::add_trap_into(graph);
	}

	fn handle_pre_call(&mut self, graph: &mut DataFlowGraph, from: u16, to: u16) -> Vec<Link> {
		let mut arguments = self.locals[usize::from(from)..usize::from(to)].to_vec();

		self.create_fence(graph);

		arguments.push(self.trap);

		arguments
	}

	fn handle_post_call(&mut self, graph: &mut DataFlowGraph, call: u32, from: u16, to: u16) {
		let destinations = self.locals[usize::from(from)..usize::from(to)].iter_mut();
		let mut call = (0..u16::MAX).map(|port| Link(call, port));

		for (destination, result) in destinations.zip(&mut call) {
			*destination = result;
		}

		self.trap = call.next().unwrap();

		self.create_fence(graph);
	}

	fn handle_call(&mut self, graph: &mut DataFlowGraph, instruction: Call) {
		let Call {
			destinations,
			sources,
			function,
		} = instruction;

		let arguments = self.handle_pre_call(graph, sources.0, sources.1);

		let call = simple::Apply::add_into(
			graph,
			self.locals[usize::from(function)],
			arguments,
			destinations.1 - destinations.0,
		);

		self.handle_post_call(graph, call, destinations.0, destinations.1);
	}

	fn handle_integer_unary_operation(
		&mut self,
		graph: &mut DataFlowGraph,
		instruction: IntegerUnaryOperation,
	) {
		let IntegerUnaryOperation {
			destination,
			source,
			kind,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = simple::IntegerUnaryOperation::add_into(
			graph,
			self.locals[usize::from(source)],
			kind,
			operator,
		);
	}

	fn handle_integer_binary_operation(
		&mut self,
		graph: &mut DataFlowGraph,
		instruction: IntegerBinaryOperation,
	) {
		let IntegerBinaryOperation {
			destination,
			lhs,
			rhs,
			kind,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = simple::IntegerBinaryOperation::add_into(
			graph,
			self.locals[usize::from(lhs)],
			self.locals[usize::from(rhs)],
			kind,
			operator,
		);
	}

	fn handle_integer_compare_operation(
		&mut self,
		graph: &mut DataFlowGraph,
		instruction: IntegerCompareOperation,
	) {
		let IntegerCompareOperation {
			destination,
			lhs,
			rhs,
			kind,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = simple::IntegerCompareOperation::add_into(
			graph,
			self.locals[usize::from(lhs)],
			self.locals[usize::from(rhs)],
			kind,
			operator,
		);
	}

	fn handle_integer_narrow(&mut self, graph: &mut DataFlowGraph, instruction: IntegerNarrow) {
		let IntegerNarrow {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] =
			simple::IntegerNarrow::add_into(graph, self.locals[usize::from(source)]);
	}

	fn handle_integer_widen(&mut self, graph: &mut DataFlowGraph, instruction: IntegerWiden) {
		let IntegerWiden {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] =
			simple::IntegerWiden::add_into(graph, self.locals[usize::from(source)]);
	}

	fn handle_integer_extend(&mut self, graph: &mut DataFlowGraph, instruction: IntegerExtend) {
		let IntegerExtend {
			destination,
			source,
			kind,
		} = instruction;

		self.locals[usize::from(destination)] =
			simple::IntegerExtend::add_into(graph, self.locals[usize::from(source)], kind);
	}

	fn handle_integer_convert_to_number(
		&mut self,
		graph: &mut DataFlowGraph,
		instruction: IntegerConvertToNumber,
	) {
		let IntegerConvertToNumber {
			destination,
			source,
			signed,
			to,
			from,
		} = instruction;

		self.locals[usize::from(destination)] = simple::IntegerConvertToNumber::add_into(
			graph,
			self.locals[usize::from(source)],
			signed,
			to,
			from,
		);
	}

	fn handle_integer_transmute_to_number(
		&mut self,
		graph: &mut DataFlowGraph,
		instruction: IntegerTransmuteToNumber,
	) {
		let IntegerTransmuteToNumber {
			destination,
			source,
			from,
		} = instruction;

		self.locals[usize::from(destination)] = simple::IntegerTransmuteToNumber::add_into(
			graph,
			self.locals[usize::from(source)],
			from,
		);
	}

	fn handle_number_unary_operation(
		&mut self,
		graph: &mut DataFlowGraph,
		instruction: NumberUnaryOperation,
	) {
		let NumberUnaryOperation {
			destination,
			source,
			kind,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = simple::NumberUnaryOperation::add_into(
			graph,
			self.locals[usize::from(source)],
			kind,
			operator,
		);
	}

	fn handle_number_binary_operation(
		&mut self,
		graph: &mut DataFlowGraph,
		instruction: NumberBinaryOperation,
	) {
		let NumberBinaryOperation {
			destination,
			lhs,
			rhs,
			kind,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = simple::NumberBinaryOperation::add_into(
			graph,
			self.locals[usize::from(lhs)],
			self.locals[usize::from(rhs)],
			kind,
			operator,
		);
	}

	fn handle_number_compare_operation(
		&mut self,
		graph: &mut DataFlowGraph,
		instruction: NumberCompareOperation,
	) {
		let NumberCompareOperation {
			destination,
			lhs,
			rhs,
			kind,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = simple::NumberCompareOperation::add_into(
			graph,
			self.locals[usize::from(lhs)],
			self.locals[usize::from(rhs)],
			kind,
			operator,
		);
	}

	fn handle_number_narrow(&mut self, graph: &mut DataFlowGraph, instruction: NumberNarrow) {
		let NumberNarrow {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] =
			simple::NumberNarrow::add_into(graph, self.locals[usize::from(source)]);
	}

	fn handle_number_widen(&mut self, graph: &mut DataFlowGraph, instruction: NumberWiden) {
		let NumberWiden {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] =
			simple::NumberWiden::add_into(graph, self.locals[usize::from(source)]);
	}

	fn handle_number_truncate_to_integer(
		&mut self,
		graph: &mut DataFlowGraph,
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

		self.locals[usize::from(destination)] = simple::NumberTruncateToInteger::add_into(
			graph,
			self.locals[usize::from(source)],
			signed,
			saturate,
			to,
			from,
		);
	}

	fn handle_number_transmute_to_integer(
		&mut self,
		graph: &mut DataFlowGraph,
		instruction: NumberTransmuteToInteger,
	) {
		let NumberTransmuteToInteger {
			destination,
			source,
			from,
		} = instruction;

		self.locals[usize::from(destination)] = simple::NumberTransmuteToInteger::add_into(
			graph,
			self.locals[usize::from(source)],
			from,
		);
	}

	fn handle_global_get(&mut self, graph: &mut DataFlowGraph, instruction: GlobalGet) {
		let GlobalGet {
			destination,
			source,
		} = instruction;

		let state = self.dependencies.get(ReferenceType::Global, source);
		let (result, state) = simple::GlobalGet::add_into(graph, state);

		self.locals[usize::from(destination)] = result;

		self.dependencies.set(ReferenceType::Global, source, state);
	}

	fn handle_global_set(&mut self, graph: &mut DataFlowGraph, instruction: GlobalSet) {
		let GlobalSet {
			destination,
			source,
		} = instruction;

		let state = simple::GlobalSet::add_into(
			graph,
			self.dependencies.get(ReferenceType::Global, destination),
			self.locals[usize::from(source)],
		);

		self.dependencies
			.set(ReferenceType::Global, destination, state);
	}

	fn load_location(&self, kind: ReferenceType, location: Location) -> simple::Location {
		let Location { reference, offset } = location;

		simple::Location {
			reference: self.dependencies.get(kind, reference),
			offset: self.locals[usize::from(offset)],
		}
	}

	fn handle_table_get(&mut self, graph: &mut DataFlowGraph, instruction: TableGet) {
		let TableGet {
			destination,
			source,
		} = instruction;

		let state = self.load_location(ReferenceType::Table, source);
		let (result, state) = simple::TableGet::add_into(graph, state);

		self.locals[usize::from(destination)] = result;

		self.dependencies
			.set(ReferenceType::Table, source.reference, state);
	}

	fn handle_table_set(&mut self, graph: &mut DataFlowGraph, instruction: TableSet) {
		let TableSet {
			destination,
			source,
		} = instruction;

		let state = simple::TableSet::add_into(
			graph,
			self.load_location(ReferenceType::Table, destination),
			self.locals[usize::from(source)],
		);

		self.dependencies
			.set(ReferenceType::Table, destination.reference, state);
	}

	fn handle_table_size(&mut self, graph: &mut DataFlowGraph, instruction: TableSize) {
		let TableSize { destination, table } = instruction;

		let state = self.dependencies.get(ReferenceType::Table, table);
		let (result, state) = simple::TableSize::add_into(graph, state);

		self.locals[usize::from(destination)] = result;

		self.dependencies.set(ReferenceType::Table, table, state);
	}

	fn handle_table_grow(&mut self, graph: &mut DataFlowGraph, instruction: TableGrow) {
		let TableGrow {
			destination,
			table,
			size,
			initializer,
		} = instruction;

		let (result, state) = simple::TableGrow::add_into(
			graph,
			self.dependencies.get(ReferenceType::Table, table),
			self.locals[usize::from(initializer)],
			self.locals[usize::from(size)],
		);

		self.locals[usize::from(destination)] = result;

		self.dependencies.set(ReferenceType::Table, table, state);
	}

	fn handle_table_fill(&mut self, graph: &mut DataFlowGraph, instruction: TableFill) {
		let TableFill {
			destination,
			source,
			size,
		} = instruction;

		let state = simple::TableFill::add_into(
			graph,
			self.load_location(ReferenceType::Table, destination),
			self.locals[usize::from(source)],
			self.locals[usize::from(size)],
		);

		self.dependencies
			.set(ReferenceType::Table, destination.reference, state);
	}

	fn handle_table_copy(&mut self, graph: &mut DataFlowGraph, instruction: TableCopy) {
		let TableCopy {
			destination,
			source,
			size,
		} = instruction;

		let (destination_state, source_state) = simple::TableCopy::add_into(
			graph,
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

	fn handle_table_init(&mut self, graph: &mut DataFlowGraph, instruction: TableInit) {
		let TableInit {
			destination,
			source,
			size,
		} = instruction;

		let elements = self.load_location(ReferenceType::Elements, source);

		let (destination_state, source_state) = simple::TableCopy::add_into(
			graph,
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

	fn handle_elements_drop(&mut self, graph: &mut DataFlowGraph, instruction: ElementsDrop) {
		let ElementsDrop { source } = instruction;

		let state = self.dependencies.get(ReferenceType::Elements, source);
		let state = simple::TableDrop::add_into(graph, state);

		self.dependencies
			.set(ReferenceType::Elements, source, state);
	}

	fn handle_memory_load(&mut self, graph: &mut DataFlowGraph, instruction: MemoryLoad) {
		let MemoryLoad {
			destination,
			source,
			kind,
		} = instruction;

		let state = self.load_location(ReferenceType::Memory, source);
		let (result, state) = simple::MemoryLoad::add_into(graph, state, kind);

		self.locals[usize::from(destination)] = result;

		self.dependencies
			.set(ReferenceType::Memory, source.reference, state);
	}

	fn handle_memory_store(&mut self, graph: &mut DataFlowGraph, instruction: MemoryStore) {
		let MemoryStore {
			destination,
			source,
			kind,
		} = instruction;

		let state = simple::MemoryStore::add_into(
			graph,
			self.load_location(ReferenceType::Memory, destination),
			self.locals[usize::from(source)],
			kind,
		);

		self.dependencies
			.set(ReferenceType::Memory, destination.reference, state);
	}

	fn handle_memory_size(&mut self, graph: &mut DataFlowGraph, instruction: MemorySize) {
		let MemorySize {
			destination,
			memory,
		} = instruction;

		let state = self.dependencies.get(ReferenceType::Memory, memory);
		let (result, state) = simple::MemorySize::add_into(graph, state);

		self.locals[usize::from(destination)] = result;

		self.dependencies.set(ReferenceType::Memory, memory, state);
	}

	fn handle_memory_grow(&mut self, graph: &mut DataFlowGraph, instruction: MemoryGrow) {
		let MemoryGrow {
			destination,
			memory,
			size,
		} = instruction;

		let (result, state) = simple::MemoryGrow::add_into(
			graph,
			self.dependencies.get(ReferenceType::Memory, memory),
			self.locals[usize::from(size)],
		);

		self.locals[usize::from(destination)] = result;

		self.dependencies.set(ReferenceType::Memory, memory, state);
	}

	fn handle_memory_fill(&mut self, graph: &mut DataFlowGraph, instruction: MemoryFill) {
		let MemoryFill {
			destination,
			byte,
			size,
		} = instruction;

		let state = simple::MemoryFill::add_into(
			graph,
			self.load_location(ReferenceType::Memory, destination),
			self.locals[usize::from(byte)],
			self.locals[usize::from(size)],
		);

		self.dependencies
			.set(ReferenceType::Memory, destination.reference, state);
	}

	fn handle_memory_copy(&mut self, graph: &mut DataFlowGraph, instruction: MemoryCopy) {
		let MemoryCopy {
			destination,
			source,
			size,
		} = instruction;

		let (destination_state, source_state) = simple::MemoryCopy::add_into(
			graph,
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

	fn handle_memory_init(&mut self, graph: &mut DataFlowGraph, instruction: MemoryInit) {
		let MemoryInit {
			destination,
			source,
			size,
		} = instruction;

		let (destination_state, source_state) = simple::MemoryCopy::add_into(
			graph,
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

	fn handle_data_drop(&mut self, graph: &mut DataFlowGraph, instruction: DataDrop) {
		let DataDrop { source } = instruction;

		let state = self.dependencies.get(ReferenceType::Data, source);
		let state = simple::MemoryDrop::add_into(graph, state);

		self.dependencies.set(ReferenceType::Data, source, state);
	}

	fn handle_instruction(&mut self, graph: &mut DataFlowGraph, instruction: Instruction) {
		match instruction {
			Instruction::LocalSet(instruction) => self.handle_local_set(instruction),
			Instruction::LocalBranch(instruction) => self.handle_local_branch(instruction),
			Instruction::I32Constant(instruction) => self.handle_i32_constant(graph, instruction),
			Instruction::I64Constant(instruction) => self.handle_i64_constant(graph, instruction),
			Instruction::F32Constant(instruction) => self.handle_f32_constant(graph, instruction),
			Instruction::F64Constant(instruction) => self.handle_f64_constant(graph, instruction),
			Instruction::RefIsNull(instruction) => self.handle_ref_is_null(graph, instruction),
			Instruction::RefNull(instruction) => self.handle_ref_null(graph, instruction),
			Instruction::RefFunction(instruction) => self.handle_ref_function(graph, instruction),
			Instruction::Call(instruction) => self.handle_call(graph, instruction),
			Instruction::Unreachable => self.handle_unreachable(graph),
			Instruction::IntegerUnaryOperation(instruction) => {
				self.handle_integer_unary_operation(graph, instruction);
			}
			Instruction::IntegerBinaryOperation(instruction) => {
				self.handle_integer_binary_operation(graph, instruction);
			}
			Instruction::IntegerCompareOperation(instruction) => {
				self.handle_integer_compare_operation(graph, instruction);
			}
			Instruction::IntegerNarrow(instruction) => {
				self.handle_integer_narrow(graph, instruction);
			}
			Instruction::IntegerWiden(instruction) => self.handle_integer_widen(graph, instruction),
			Instruction::IntegerExtend(instruction) => {
				self.handle_integer_extend(graph, instruction);
			}
			Instruction::IntegerConvertToNumber(instruction) => {
				self.handle_integer_convert_to_number(graph, instruction);
			}
			Instruction::IntegerTransmuteToNumber(instruction) => {
				self.handle_integer_transmute_to_number(graph, instruction);
			}
			Instruction::NumberUnaryOperation(instruction) => {
				self.handle_number_unary_operation(graph, instruction);
			}
			Instruction::NumberBinaryOperation(instruction) => {
				self.handle_number_binary_operation(graph, instruction);
			}
			Instruction::NumberCompareOperation(instruction) => {
				self.handle_number_compare_operation(graph, instruction);
			}
			Instruction::NumberNarrow(instruction) => self.handle_number_narrow(graph, instruction),
			Instruction::NumberWiden(instruction) => self.handle_number_widen(graph, instruction),
			Instruction::NumberTruncateToInteger(instruction) => {
				self.handle_number_truncate_to_integer(graph, instruction);
			}
			Instruction::NumberTransmuteToInteger(instruction) => {
				self.handle_number_transmute_to_integer(graph, instruction);
			}
			Instruction::GlobalGet(instruction) => self.handle_global_get(graph, instruction),
			Instruction::GlobalSet(instruction) => self.handle_global_set(graph, instruction),
			Instruction::TableGet(instruction) => self.handle_table_get(graph, instruction),
			Instruction::TableSet(instruction) => self.handle_table_set(graph, instruction),
			Instruction::TableSize(instruction) => self.handle_table_size(graph, instruction),
			Instruction::TableGrow(instruction) => self.handle_table_grow(graph, instruction),
			Instruction::TableFill(instruction) => self.handle_table_fill(graph, instruction),
			Instruction::TableCopy(instruction) => self.handle_table_copy(graph, instruction),
			Instruction::TableInit(instruction) => self.handle_table_init(graph, instruction),
			Instruction::ElementsDrop(instruction) => self.handle_elements_drop(graph, instruction),
			Instruction::MemoryLoad(instruction) => self.handle_memory_load(graph, instruction),
			Instruction::MemoryStore(instruction) => self.handle_memory_store(graph, instruction),
			Instruction::MemorySize(instruction) => self.handle_memory_size(graph, instruction),
			Instruction::MemoryGrow(instruction) => self.handle_memory_grow(graph, instruction),
			Instruction::MemoryFill(instruction) => self.handle_memory_fill(graph, instruction),
			Instruction::MemoryCopy(instruction) => self.handle_memory_copy(graph, instruction),
			Instruction::MemoryInit(instruction) => self.handle_memory_init(graph, instruction),
			Instruction::DataDrop(instruction) => self.handle_data_drop(graph, instruction),
		}
	}

	pub fn run(&mut self, graph: &mut DataFlowGraph, instructions: &[Instruction]) {
		for &instruction in instructions {
			self.handle_instruction(graph, instruction);
		}
	}
}
