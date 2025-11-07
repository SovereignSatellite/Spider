use alloc::vec::Vec;
use control_flow_graph::instruction::{
	Call, DataDrop, ElementsDrop, F32Constant, F64Constant, GlobalGet, GlobalSet, I32Constant,
	I64Constant, Instruction, IntegerBinaryOperation, IntegerCompareOperation,
	IntegerConvertToNumber, IntegerExtend, IntegerNarrow, IntegerTransmuteToNumber,
	IntegerUnaryOperation, IntegerWiden, LocalBranch, LocalSet, MemoryCopy, MemoryFill, MemoryGrow,
	MemoryInit, MemoryLoad, MemorySize, MemoryStore, Name, NumberBinaryOperation,
	NumberCompareOperation, NumberNarrow, NumberTransmuteToInteger, NumberTruncateToInteger,
	NumberUnaryOperation, NumberWiden, RefFunction, RefIsNull, RefNull, TableCopy, TableFill,
	TableGet, TableGrow, TableInit, TableSet, TableSize,
};
use control_flow_liveness::references::{Reference, ReferenceType};
use data_flow_graph::{DataFlowGraph, Link, base::Location, control::ValueType};

use super::dependency_map::DependencyMap;

const LOCAL_BASE: usize = Name::COUNT as usize;

pub struct BasicBlockConverter {
	locals: Vec<Link>,

	condition: Link,
	trap: Link,
	dependencies: DependencyMap,
}

impl BasicBlockConverter {
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

	pub fn get_function_outputs(&self, results: usize) -> Vec<Link> {
		let mut list = self.locals[LOCAL_BASE..LOCAL_BASE + results].to_vec();

		self.dependencies.extend_into(&mut list);

		list.push(self.trap);

		list
	}

	pub fn set_function_inputs(
		&mut self,
		lambda_in: u32,
		arguments: usize,
		dependencies: &[Reference],
	) {
		let mut inputs = (0..u16::MAX).map(|port| Link(lambda_in, port));

		self.dependencies.fill_keys(dependencies);
		self.dependencies.fill_values(&mut inputs);

		let reserved = core::iter::repeat_n(Link::DANGLING, LOCAL_BASE);

		self.locals.clear();
		self.locals.extend(reserved);
		self.locals.extend(inputs.by_ref().take(arguments));

		self.trap = inputs.next().unwrap();
	}

	pub fn set_local_types(&mut self, graph: &mut DataFlowGraph, types: &[ValueType]) {
		self.locals.extend(types.iter().map(|&local| match local {
			ValueType::I32 => graph.add_i32(0),
			ValueType::I64 => graph.add_i64(0),
			ValueType::F32 => graph.add_f32(0.0),
			ValueType::F64 => graph.add_f64(0.0),
			ValueType::Reference => graph.add_null(),
		}));
	}

	pub fn set_stack_size(&mut self, graph: &mut DataFlowGraph, size: u16) {
		let null = graph.add_null();
		let count = usize::from(size).saturating_sub(self.locals.len());

		self.locals.extend(core::iter::repeat_n(null, count));
		self.locals[..LOCAL_BASE].fill(null);
	}

	pub fn get_active_bindings(&self, locals: &[u16]) -> Vec<Link> {
		let mut results = Vec::new();

		self.dependencies.extend_into(&mut results);

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

		self.dependencies.fill_values(&mut producer);

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

		self.locals[usize::from(destination)] = graph.add_i32(data);
	}

	fn handle_i64_constant(&mut self, graph: &mut DataFlowGraph, instruction: I64Constant) {
		let I64Constant { destination, data } = instruction;

		self.locals[usize::from(destination)] = graph.add_i64(data);
	}

	fn handle_f32_constant(&mut self, graph: &mut DataFlowGraph, instruction: F32Constant) {
		let F32Constant { destination, data } = instruction;

		self.locals[usize::from(destination)] = graph.add_f32(data);
	}

	fn handle_f64_constant(&mut self, graph: &mut DataFlowGraph, instruction: F64Constant) {
		let F64Constant { destination, data } = instruction;

		self.locals[usize::from(destination)] = graph.add_f64(data);
	}

	fn handle_ref_is_null(&mut self, graph: &mut DataFlowGraph, instruction: RefIsNull) {
		let RefIsNull {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] =
			graph.add_ref_is_null(self.locals[usize::from(source)]);
	}

	fn handle_ref_null(&mut self, graph: &mut DataFlowGraph, instruction: RefNull) {
		let RefNull { destination } = instruction;

		self.locals[usize::from(destination)] = graph.add_null();
	}

	fn handle_ref_function(&mut self, graph: &mut DataFlowGraph, instruction: RefFunction) {
		let RefFunction {
			destination,
			function,
		} = instruction;

		let state = self.dependencies.get(ReferenceType::Function, function);

		self.locals[usize::from(destination)] = graph.add_global_get(state).0;
	}

	fn handle_unreachable(&mut self, graph: &mut DataFlowGraph) {
		self.trap = graph.add_trap();
	}

	fn handle_pre_call(&self, sources: core::ops::Range<usize>) -> (Vec<Link>, usize) {
		let mut arguments = self.locals[sources.clone()].to_vec();

		arguments.push(self.trap);

		self.dependencies.extend_into(&mut arguments);

		let count = arguments.len() - sources.len();

		(arguments, count)
	}

	fn handle_post_call(&mut self, call: u32, destinations: core::ops::Range<usize>) {
		let mut call = (0..u16::MAX).map(|port| Link(call, port));

		for (destination, result) in self.locals[destinations].iter_mut().zip(&mut call) {
			*destination = result;
		}

		self.trap = call.next().unwrap();

		self.dependencies.fill_values(call);
	}

	fn handle_call(&mut self, graph: &mut DataFlowGraph, instruction: Call) {
		let Call {
			destinations,
			sources,
			function,
		} = instruction;

		let destinations = usize::from(destinations.0)..usize::from(destinations.1);
		let sources = usize::from(sources.0)..usize::from(sources.1);

		let (arguments, states) = self.handle_pre_call(sources);
		let results = destinations.len();

		self.handle_post_call(
			graph.add_apply(
				self.locals[usize::from(function)],
				arguments,
				results.try_into().unwrap(),
				states.try_into().unwrap(),
			),
			destinations,
		);
	}

	fn handle_integer_unary_operation(
		&mut self,
		graph: &mut DataFlowGraph,
		instruction: IntegerUnaryOperation,
	) {
		let IntegerUnaryOperation {
			destination,
			source,
			r#type,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] =
			graph.add_integer_unary_operation(self.locals[usize::from(source)], r#type, operator);
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
			r#type,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = graph.add_integer_binary_operation(
			self.locals[usize::from(lhs)],
			self.locals[usize::from(rhs)],
			r#type,
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
			r#type,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = graph.add_integer_compare_operation(
			self.locals[usize::from(lhs)],
			self.locals[usize::from(rhs)],
			r#type,
			operator,
		);
	}

	fn handle_integer_narrow(&mut self, graph: &mut DataFlowGraph, instruction: IntegerNarrow) {
		let IntegerNarrow {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] =
			graph.add_integer_narrow(self.locals[usize::from(source)]);
	}

	fn handle_integer_widen(&mut self, graph: &mut DataFlowGraph, instruction: IntegerWiden) {
		let IntegerWiden {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] =
			graph.add_integer_widen(self.locals[usize::from(source)]);
	}

	fn handle_integer_extend(&mut self, graph: &mut DataFlowGraph, instruction: IntegerExtend) {
		let IntegerExtend {
			destination,
			source,
			r#type,
		} = instruction;

		self.locals[usize::from(destination)] =
			graph.add_integer_extend(self.locals[usize::from(source)], r#type);
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

		self.locals[usize::from(destination)] =
			graph.add_integer_convert_to_number(self.locals[usize::from(source)], signed, to, from);
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

		self.locals[usize::from(destination)] =
			graph.add_integer_transmute_to_number(self.locals[usize::from(source)], from);
	}

	fn handle_number_unary_operation(
		&mut self,
		graph: &mut DataFlowGraph,
		instruction: NumberUnaryOperation,
	) {
		let NumberUnaryOperation {
			destination,
			source,
			r#type,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] =
			graph.add_number_unary_operation(self.locals[usize::from(source)], r#type, operator);
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
			r#type,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = graph.add_number_binary_operation(
			self.locals[usize::from(lhs)],
			self.locals[usize::from(rhs)],
			r#type,
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
			r#type,
			operator,
		} = instruction;

		self.locals[usize::from(destination)] = graph.add_number_compare_operation(
			self.locals[usize::from(lhs)],
			self.locals[usize::from(rhs)],
			r#type,
			operator,
		);
	}

	fn handle_number_narrow(&mut self, graph: &mut DataFlowGraph, instruction: NumberNarrow) {
		let NumberNarrow {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] =
			graph.add_number_narrow(self.locals[usize::from(source)]);
	}

	fn handle_number_widen(&mut self, graph: &mut DataFlowGraph, instruction: NumberWiden) {
		let NumberWiden {
			destination,
			source,
		} = instruction;

		self.locals[usize::from(destination)] =
			graph.add_number_widen(self.locals[usize::from(source)]);
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

		self.locals[usize::from(destination)] = graph.add_number_truncate_to_integer(
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

		self.locals[usize::from(destination)] =
			graph.add_number_transmute_to_integer(self.locals[usize::from(source)], from);
	}

	fn handle_global_get(&mut self, graph: &mut DataFlowGraph, instruction: GlobalGet) {
		let GlobalGet {
			destination,
			source,
		} = instruction;

		let state = self.dependencies.get(ReferenceType::Global, source);
		let (result, state) = graph.add_global_get(state);

		self.locals[usize::from(destination)] = result;

		self.dependencies.set(ReferenceType::Global, source, state);
	}

	fn handle_global_set(&mut self, graph: &mut DataFlowGraph, instruction: GlobalSet) {
		let GlobalSet {
			destination,
			source,
		} = instruction;

		let state = graph.add_global_set(
			self.dependencies.get(ReferenceType::Global, destination),
			self.locals[usize::from(source)],
		);

		self.dependencies
			.set(ReferenceType::Global, destination, state);
	}

	fn load_location(
		&self,
		r#type: ReferenceType,
		location: control_flow_graph::instruction::Location,
	) -> Location {
		Location {
			reference: self.dependencies.get(r#type, location.reference),
			offset: self.locals[usize::from(location.offset)],
		}
	}

	fn handle_table_get(&mut self, graph: &mut DataFlowGraph, instruction: TableGet) {
		let TableGet {
			destination,
			source,
		} = instruction;

		let state = self.load_location(ReferenceType::Table, source);
		let (result, state) = graph.add_table_get(state);

		self.locals[usize::from(destination)] = result;

		self.dependencies
			.set(ReferenceType::Table, source.reference, state);
	}

	fn handle_table_set(&mut self, graph: &mut DataFlowGraph, instruction: TableSet) {
		let TableSet {
			destination,
			source,
		} = instruction;

		let state = graph.add_table_set(
			self.load_location(ReferenceType::Table, destination),
			self.locals[usize::from(source)],
		);

		self.dependencies
			.set(ReferenceType::Table, destination.reference, state);
	}

	fn handle_table_size(&mut self, graph: &mut DataFlowGraph, instruction: TableSize) {
		let TableSize { destination, table } = instruction;

		let state = self.dependencies.get(ReferenceType::Table, table);
		let (result, state) = graph.add_table_size(state);

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

		let (result, state) = graph.add_table_grow(
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

		let state = graph.add_table_fill(
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

		let (destination_state, source_state) = graph.add_table_copy(
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

		let (destination_state, source_state) = graph.add_table_copy(
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
		let state = graph.add_table_drop(state);

		self.dependencies
			.set(ReferenceType::Elements, source, state);
	}

	fn handle_memory_load(&mut self, graph: &mut DataFlowGraph, instruction: MemoryLoad) {
		let MemoryLoad {
			destination,
			source,
			r#type,
		} = instruction;

		let state = self.load_location(ReferenceType::Memory, source);
		let (result, state) = graph.add_memory_load(state, r#type);

		self.locals[usize::from(destination)] = result;

		self.dependencies
			.set(ReferenceType::Memory, source.reference, state);
	}

	fn handle_memory_store(&mut self, graph: &mut DataFlowGraph, instruction: MemoryStore) {
		let MemoryStore {
			destination,
			source,
			r#type,
		} = instruction;

		let state = graph.add_memory_store(
			self.load_location(ReferenceType::Memory, destination),
			self.locals[usize::from(source)],
			r#type,
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
		let (result, state) = graph.add_memory_size(state);

		self.locals[usize::from(destination)] = result;

		self.dependencies.set(ReferenceType::Memory, memory, state);
	}

	fn handle_memory_grow(&mut self, graph: &mut DataFlowGraph, instruction: MemoryGrow) {
		let MemoryGrow {
			destination,
			memory,
			size,
		} = instruction;

		let (result, state) = graph.add_memory_grow(
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

		let state = graph.add_memory_fill(
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

		let (destination_state, source_state) = graph.add_memory_copy(
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

		let (destination_state, source_state) = graph.add_memory_copy(
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
		let state = graph.add_memory_drop(state);

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
