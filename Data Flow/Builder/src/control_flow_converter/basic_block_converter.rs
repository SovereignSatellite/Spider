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
use data_flow_graph::{DataFlowGraph, Link, mvp::Location, nested::ValueType};

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

	fn handle_local_set(&mut self, local_set: LocalSet) {
		let LocalSet {
			destination,
			source,
		} = local_set;

		self.locals[usize::from(destination)] = self.locals[usize::from(source)];
	}

	fn handle_local_branch(&mut self, local_branch: LocalBranch) {
		let LocalBranch { source } = local_branch;

		self.condition = self.locals[usize::from(source)];
	}

	fn handle_i32_constant(&mut self, graph: &mut DataFlowGraph, i32_constant: I32Constant) {
		let I32Constant { destination, data } = i32_constant;

		self.locals[usize::from(destination)] = graph.add_i32(data);
	}

	fn handle_i64_constant(&mut self, graph: &mut DataFlowGraph, i64_constant: I64Constant) {
		let I64Constant { destination, data } = i64_constant;

		self.locals[usize::from(destination)] = graph.add_i64(data);
	}

	fn handle_f32_constant(&mut self, graph: &mut DataFlowGraph, f32_constant: F32Constant) {
		let F32Constant { destination, data } = f32_constant;

		self.locals[usize::from(destination)] = graph.add_f32(data);
	}

	fn handle_f64_constant(&mut self, graph: &mut DataFlowGraph, f64_constant: F64Constant) {
		let F64Constant { destination, data } = f64_constant;

		self.locals[usize::from(destination)] = graph.add_f64(data);
	}

	fn handle_ref_is_null(&mut self, graph: &mut DataFlowGraph, ref_is_null: RefIsNull) {
		let RefIsNull {
			destination,
			source,
		} = ref_is_null;

		self.locals[usize::from(destination)] =
			graph.add_ref_is_null(self.locals[usize::from(source)]);
	}

	fn handle_ref_null(&mut self, graph: &mut DataFlowGraph, ref_null: RefNull) {
		let RefNull { destination } = ref_null;

		self.locals[usize::from(destination)] = graph.add_null();
	}

	fn handle_ref_function(&mut self, graph: &mut DataFlowGraph, ref_function: RefFunction) {
		let RefFunction {
			destination,
			function,
		} = ref_function;

		let state = self.dependencies.get(ReferenceType::Function, function);

		self.locals[usize::from(destination)] = graph.add_global_get(state);
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

	fn handle_call(&mut self, graph: &mut DataFlowGraph, call: Call) {
		let Call {
			destinations,
			sources,
			function,
		} = call;

		let destinations = usize::from(destinations.0)..usize::from(destinations.1);
		let sources = usize::from(sources.0)..usize::from(sources.1);

		let (arguments, states) = self.handle_pre_call(sources);
		let results = destinations.len();

		self.handle_post_call(
			graph.add_call(
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
		integer_unary_operation: IntegerUnaryOperation,
	) {
		let IntegerUnaryOperation {
			destination,
			source,
			r#type,
			operator,
		} = integer_unary_operation;

		self.locals[usize::from(destination)] =
			graph.add_integer_unary_operation(self.locals[usize::from(source)], r#type, operator);
	}

	fn handle_integer_binary_operation(
		&mut self,
		graph: &mut DataFlowGraph,
		integer_binary_operation: IntegerBinaryOperation,
	) {
		let IntegerBinaryOperation {
			destination,
			lhs,
			rhs,
			r#type,
			operator,
		} = integer_binary_operation;

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
		integer_compare_operation: IntegerCompareOperation,
	) {
		let IntegerCompareOperation {
			destination,
			lhs,
			rhs,
			r#type,
			operator,
		} = integer_compare_operation;

		self.locals[usize::from(destination)] = graph.add_integer_compare_operation(
			self.locals[usize::from(lhs)],
			self.locals[usize::from(rhs)],
			r#type,
			operator,
		);
	}

	fn handle_integer_narrow(&mut self, graph: &mut DataFlowGraph, integer_narrow: IntegerNarrow) {
		let IntegerNarrow {
			destination,
			source,
		} = integer_narrow;

		self.locals[usize::from(destination)] =
			graph.add_integer_narrow(self.locals[usize::from(source)]);
	}

	fn handle_integer_widen(&mut self, graph: &mut DataFlowGraph, integer_widen: IntegerWiden) {
		let IntegerWiden {
			destination,
			source,
		} = integer_widen;

		self.locals[usize::from(destination)] =
			graph.add_integer_widen(self.locals[usize::from(source)]);
	}

	fn handle_integer_extend(&mut self, graph: &mut DataFlowGraph, integer_extend: IntegerExtend) {
		let IntegerExtend {
			destination,
			source,
			r#type,
		} = integer_extend;

		self.locals[usize::from(destination)] =
			graph.add_integer_extend(self.locals[usize::from(source)], r#type);
	}

	fn handle_integer_convert_to_number(
		&mut self,
		graph: &mut DataFlowGraph,
		integer_convert_to_number: IntegerConvertToNumber,
	) {
		let IntegerConvertToNumber {
			destination,
			source,
			signed,
			to,
			from,
		} = integer_convert_to_number;

		self.locals[usize::from(destination)] =
			graph.add_integer_convert_to_number(self.locals[usize::from(source)], signed, to, from);
	}

	fn handle_integer_transmute_to_number(
		&mut self,
		graph: &mut DataFlowGraph,
		integer_transmute_to_number: IntegerTransmuteToNumber,
	) {
		let IntegerTransmuteToNumber {
			destination,
			source,
			from,
		} = integer_transmute_to_number;

		self.locals[usize::from(destination)] =
			graph.add_integer_transmute_to_number(self.locals[usize::from(source)], from);
	}

	fn handle_number_unary_operation(
		&mut self,
		graph: &mut DataFlowGraph,
		number_unary_operation: NumberUnaryOperation,
	) {
		let NumberUnaryOperation {
			destination,
			source,
			r#type,
			operator,
		} = number_unary_operation;

		self.locals[usize::from(destination)] =
			graph.add_number_unary_operation(self.locals[usize::from(source)], r#type, operator);
	}

	fn handle_number_binary_operation(
		&mut self,
		graph: &mut DataFlowGraph,
		number_binary_operation: NumberBinaryOperation,
	) {
		let NumberBinaryOperation {
			destination,
			lhs,
			rhs,
			r#type,
			operator,
		} = number_binary_operation;

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
		number_compare_operation: NumberCompareOperation,
	) {
		let NumberCompareOperation {
			destination,
			lhs,
			rhs,
			r#type,
			operator,
		} = number_compare_operation;

		self.locals[usize::from(destination)] = graph.add_number_compare_operation(
			self.locals[usize::from(lhs)],
			self.locals[usize::from(rhs)],
			r#type,
			operator,
		);
	}

	fn handle_number_narrow(&mut self, graph: &mut DataFlowGraph, number_narrow: NumberNarrow) {
		let NumberNarrow {
			destination,
			source,
		} = number_narrow;

		self.locals[usize::from(destination)] =
			graph.add_number_narrow(self.locals[usize::from(source)]);
	}

	fn handle_number_widen(&mut self, graph: &mut DataFlowGraph, number_widen: NumberWiden) {
		let NumberWiden {
			destination,
			source,
		} = number_widen;

		self.locals[usize::from(destination)] =
			graph.add_number_widen(self.locals[usize::from(source)]);
	}

	fn handle_number_truncate_to_integer(
		&mut self,
		graph: &mut DataFlowGraph,
		number_truncate_to_integer: NumberTruncateToInteger,
	) {
		let NumberTruncateToInteger {
			destination,
			source,
			signed,
			saturate,
			to,
			from,
		} = number_truncate_to_integer;

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
		number_transmute_to_integer: NumberTransmuteToInteger,
	) {
		let NumberTransmuteToInteger {
			destination,
			source,
			from,
		} = number_transmute_to_integer;

		self.locals[usize::from(destination)] =
			graph.add_number_transmute_to_integer(self.locals[usize::from(source)], from);
	}

	fn handle_global_get(&mut self, graph: &mut DataFlowGraph, global_get: GlobalGet) {
		let GlobalGet {
			destination,
			source,
		} = global_get;

		let state = self.dependencies.get(ReferenceType::Global, source);

		self.locals[usize::from(destination)] = graph.add_global_get(state);
	}

	fn handle_global_set(&mut self, graph: &mut DataFlowGraph, global_set: GlobalSet) {
		let GlobalSet {
			destination,
			source,
		} = global_set;

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

	fn handle_table_get(&mut self, graph: &mut DataFlowGraph, table_get: TableGet) {
		let TableGet {
			destination,
			source,
		} = table_get;

		let state = self.load_location(ReferenceType::Table, source);

		self.locals[usize::from(destination)] = graph.add_table_get(state);
	}

	fn handle_table_set(&mut self, graph: &mut DataFlowGraph, table_set: TableSet) {
		let TableSet {
			destination,
			source,
		} = table_set;

		let state = graph.add_table_set(
			self.load_location(ReferenceType::Table, destination),
			self.locals[usize::from(source)],
		);

		self.dependencies
			.set(ReferenceType::Table, destination.reference, state);
	}

	fn handle_table_size(&mut self, graph: &mut DataFlowGraph, table_size: TableSize) {
		let TableSize {
			reference,
			destination,
		} = table_size;

		let state = self.dependencies.get(ReferenceType::Table, reference);

		self.locals[usize::from(destination)] = graph.add_table_size(state);
	}

	fn handle_table_grow(&mut self, graph: &mut DataFlowGraph, table_grow: TableGrow) {
		let TableGrow {
			reference,
			destination,
			size,
			initializer,
		} = table_grow;

		let (result, state) = graph.add_table_grow(
			self.dependencies.get(ReferenceType::Table, reference),
			self.locals[usize::from(initializer)],
			self.locals[usize::from(size)],
		);

		self.locals[usize::from(destination)] = result;

		self.dependencies
			.set(ReferenceType::Table, reference, state);
	}

	fn handle_table_fill(&mut self, graph: &mut DataFlowGraph, table_fill: TableFill) {
		let TableFill {
			destination,
			source,
			size,
		} = table_fill;

		let state = graph.add_table_fill(
			self.load_location(ReferenceType::Table, destination),
			self.locals[usize::from(source)],
			self.locals[usize::from(size)],
		);

		self.dependencies
			.set(ReferenceType::Table, destination.reference, state);
	}

	fn handle_table_copy(&mut self, graph: &mut DataFlowGraph, table_copy: TableCopy) {
		let TableCopy {
			destination,
			source,
			size,
		} = table_copy;

		let state = graph.add_table_copy(
			self.load_location(ReferenceType::Table, destination),
			self.load_location(ReferenceType::Table, source),
			self.locals[usize::from(size)],
		);

		self.dependencies
			.set(ReferenceType::Table, destination.reference, state);
	}

	fn handle_table_init(&mut self, graph: &mut DataFlowGraph, table_init: TableInit) {
		let TableInit {
			destination,
			source,
			size,
		} = table_init;

		let mut source = self.load_location(ReferenceType::Elements, source);

		source.reference = graph.add_global_get(source.reference);

		let state = graph.add_table_init(
			self.load_location(ReferenceType::Table, destination),
			source,
			self.locals[usize::from(size)],
		);

		self.dependencies
			.set(ReferenceType::Table, destination.reference, state);
	}

	fn handle_elements_drop(&mut self, graph: &mut DataFlowGraph, elements_drop: ElementsDrop) {
		let ElementsDrop { source } = elements_drop;

		let state = self.dependencies.get(ReferenceType::Elements, source);
		let inner = graph.add_global_get(state);
		let inner = graph.add_elements_drop(inner);
		let state = graph.add_global_set(state, inner);

		self.dependencies
			.set(ReferenceType::Elements, source, state);
	}

	fn handle_memory_load(&mut self, graph: &mut DataFlowGraph, memory_load: MemoryLoad) {
		let MemoryLoad {
			destination,
			source,
			r#type,
		} = memory_load;

		let state = self.load_location(ReferenceType::Memory, source);

		self.locals[usize::from(destination)] = graph.add_memory_load(state, r#type);
	}

	fn handle_memory_store(&mut self, graph: &mut DataFlowGraph, memory_store: MemoryStore) {
		let MemoryStore {
			destination,
			source,
			r#type,
		} = memory_store;

		let state = graph.add_memory_store(
			self.load_location(ReferenceType::Memory, destination),
			self.locals[usize::from(source)],
			r#type,
		);

		self.dependencies
			.set(ReferenceType::Memory, destination.reference, state);
	}

	fn handle_memory_size(&mut self, graph: &mut DataFlowGraph, memory_size: MemorySize) {
		let MemorySize {
			reference,
			destination,
		} = memory_size;

		let state = self.dependencies.get(ReferenceType::Memory, reference);

		self.locals[usize::from(destination)] = graph.add_memory_size(state);
	}

	fn handle_memory_grow(&mut self, graph: &mut DataFlowGraph, memory_grow: MemoryGrow) {
		let MemoryGrow {
			reference,
			destination,
			size,
		} = memory_grow;

		let (result, state) = graph.add_memory_grow(
			self.dependencies.get(ReferenceType::Memory, reference),
			self.locals[usize::from(size)],
		);

		self.locals[usize::from(destination)] = result;

		self.dependencies
			.set(ReferenceType::Memory, reference, state);
	}

	fn handle_memory_fill(&mut self, graph: &mut DataFlowGraph, memory_fill: MemoryFill) {
		let MemoryFill {
			destination,
			byte,
			size,
		} = memory_fill;

		let state = graph.add_memory_fill(
			self.load_location(ReferenceType::Memory, destination),
			self.locals[usize::from(byte)],
			self.locals[usize::from(size)],
		);

		self.dependencies
			.set(ReferenceType::Memory, destination.reference, state);
	}

	fn handle_memory_copy(&mut self, graph: &mut DataFlowGraph, memory_copy: MemoryCopy) {
		let MemoryCopy {
			destination,
			source,
			size,
		} = memory_copy;

		let state = graph.add_memory_copy(
			self.load_location(ReferenceType::Memory, destination),
			self.load_location(ReferenceType::Memory, source),
			self.locals[usize::from(size)],
		);

		self.dependencies
			.set(ReferenceType::Memory, destination.reference, state);
	}

	fn handle_memory_init(&mut self, graph: &mut DataFlowGraph, memory_init: MemoryInit) {
		let MemoryInit {
			destination,
			source,
			size,
		} = memory_init;

		let state = graph.add_memory_init(
			self.load_location(ReferenceType::Memory, destination),
			self.load_location(ReferenceType::Data, source),
			self.locals[usize::from(size)],
		);

		self.dependencies
			.set(ReferenceType::Memory, destination.reference, state);
	}

	fn handle_data_drop(&mut self, graph: &mut DataFlowGraph, data_drop: DataDrop) {
		let DataDrop { source } = data_drop;

		let state = self.dependencies.get(ReferenceType::Data, source);
		let state = graph.add_data_drop(state);

		self.dependencies.set(ReferenceType::Data, source, state);
	}

	fn handle_instruction(&mut self, graph: &mut DataFlowGraph, instruction: Instruction) {
		match instruction {
			Instruction::LocalSet(local_set) => self.handle_local_set(local_set),
			Instruction::LocalBranch(local_branch) => self.handle_local_branch(local_branch),
			Instruction::I32Constant(i32_constant) => self.handle_i32_constant(graph, i32_constant),
			Instruction::I64Constant(i64_constant) => self.handle_i64_constant(graph, i64_constant),
			Instruction::F32Constant(f32_constant) => self.handle_f32_constant(graph, f32_constant),
			Instruction::F64Constant(f64_constant) => self.handle_f64_constant(graph, f64_constant),
			Instruction::RefIsNull(ref_is_null) => self.handle_ref_is_null(graph, ref_is_null),
			Instruction::RefNull(ref_null) => self.handle_ref_null(graph, ref_null),
			Instruction::RefFunction(ref_function) => self.handle_ref_function(graph, ref_function),
			Instruction::Call(call) => self.handle_call(graph, call),
			Instruction::Unreachable => self.handle_unreachable(graph),
			Instruction::IntegerUnaryOperation(integer_unary_operation) => {
				self.handle_integer_unary_operation(graph, integer_unary_operation);
			}
			Instruction::IntegerBinaryOperation(integer_binary_operation) => {
				self.handle_integer_binary_operation(graph, integer_binary_operation);
			}
			Instruction::IntegerCompareOperation(integer_compare_operation) => {
				self.handle_integer_compare_operation(graph, integer_compare_operation);
			}
			Instruction::IntegerNarrow(integer_narrow) => {
				self.handle_integer_narrow(graph, integer_narrow);
			}
			Instruction::IntegerWiden(integer_widen) => {
				self.handle_integer_widen(graph, integer_widen);
			}
			Instruction::IntegerExtend(integer_extend) => {
				self.handle_integer_extend(graph, integer_extend);
			}
			Instruction::IntegerConvertToNumber(integer_convert_to_number) => {
				self.handle_integer_convert_to_number(graph, integer_convert_to_number);
			}
			Instruction::IntegerTransmuteToNumber(integer_transmute_to_number) => {
				self.handle_integer_transmute_to_number(graph, integer_transmute_to_number);
			}
			Instruction::NumberUnaryOperation(number_unary_operation) => {
				self.handle_number_unary_operation(graph, number_unary_operation);
			}
			Instruction::NumberBinaryOperation(number_binary_operation) => {
				self.handle_number_binary_operation(graph, number_binary_operation);
			}
			Instruction::NumberCompareOperation(number_compare_operation) => {
				self.handle_number_compare_operation(graph, number_compare_operation);
			}
			Instruction::NumberNarrow(number_narrow) => {
				self.handle_number_narrow(graph, number_narrow);
			}
			Instruction::NumberWiden(number_widen) => self.handle_number_widen(graph, number_widen),
			Instruction::NumberTruncateToInteger(number_truncate_to_integer) => {
				self.handle_number_truncate_to_integer(graph, number_truncate_to_integer);
			}
			Instruction::NumberTransmuteToInteger(number_transmute_to_integer) => {
				self.handle_number_transmute_to_integer(graph, number_transmute_to_integer);
			}
			Instruction::GlobalGet(global_get) => self.handle_global_get(graph, global_get),
			Instruction::GlobalSet(global_set) => self.handle_global_set(graph, global_set),
			Instruction::TableGet(table_get) => self.handle_table_get(graph, table_get),
			Instruction::TableSet(table_set) => self.handle_table_set(graph, table_set),
			Instruction::TableSize(table_size) => self.handle_table_size(graph, table_size),
			Instruction::TableGrow(table_grow) => self.handle_table_grow(graph, table_grow),
			Instruction::TableFill(table_fill) => self.handle_table_fill(graph, table_fill),
			Instruction::TableCopy(table_copy) => self.handle_table_copy(graph, table_copy),
			Instruction::TableInit(table_init) => self.handle_table_init(graph, table_init),
			Instruction::ElementsDrop(elements_drop) => {
				self.handle_elements_drop(graph, elements_drop);
			}
			Instruction::MemoryLoad(memory_load) => self.handle_memory_load(graph, memory_load),
			Instruction::MemoryStore(memory_store) => self.handle_memory_store(graph, memory_store),
			Instruction::MemorySize(memory_size) => self.handle_memory_size(graph, memory_size),
			Instruction::MemoryGrow(memory_grow) => self.handle_memory_grow(graph, memory_grow),
			Instruction::MemoryFill(memory_fill) => self.handle_memory_fill(graph, memory_fill),
			Instruction::MemoryCopy(memory_copy) => self.handle_memory_copy(graph, memory_copy),
			Instruction::MemoryInit(memory_init) => self.handle_memory_init(graph, memory_init),
			Instruction::DataDrop(data_drop) => self.handle_data_drop(graph, data_drop),
		}
	}

	pub fn run(&mut self, graph: &mut DataFlowGraph, instructions: &[Instruction]) {
		for &instruction in instructions {
			self.handle_instruction(graph, instruction);
		}
	}
}
