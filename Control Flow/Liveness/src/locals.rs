use alloc::vec::Vec;
use control_flow_graph::{
	ControlFlowGraph,
	instruction::{
		Call, F32Constant, F64Constant, GlobalGet, GlobalSet, I32Constant, I64Constant,
		Instruction, IntegerBinaryOperation, IntegerCompareOperation, IntegerConvertToNumber,
		IntegerExtend, IntegerNarrow, IntegerTransmuteToNumber, IntegerUnaryOperation,
		IntegerWiden, LocalBranch, LocalSet, MemoryCopy, MemoryFill, MemoryGrow, MemoryInit,
		MemoryLoad, MemorySize, MemoryStore, Name, NumberBinaryOperation, NumberCompareOperation,
		NumberNarrow, NumberTransmuteToInteger, NumberTruncateToInteger, NumberUnaryOperation,
		NumberWiden, RefFunction, RefIsNull, RefNull, TableCopy, TableFill, TableGet, TableGrow,
		TableInit, TableSet, TableSize,
	},
};
use set::{Set, Slice};

pub struct Locals {
	locals: Vec<u16>,
	ranges: Vec<(u32, u32)>,
}

impl Locals {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			locals: Vec::new(),
			ranges: Vec::new(),
		}
	}

	#[must_use]
	pub fn get(&self, id: u16) -> &[u16] {
		let (start, end) = self.ranges[usize::from(id)];

		&self.locals[start.try_into().unwrap()..end.try_into().unwrap()]
	}

	pub fn get_union<I: IntoIterator<Item = u16>>(&self, ids: I, successors: &mut Vec<u16>) {
		successors.clear();

		for id in ids {
			successors.extend(self.get(id));
		}

		successors.sort_unstable();
		successors.dedup();
	}

	fn set_len(&mut self, len: usize) {
		self.locals.clear();
		self.ranges.clear();
		self.ranges.resize(len, (0, 0));
	}

	fn insert(&mut self, id: u16, set: Slice) {
		let start = self.locals.len().try_into().unwrap();
		let iter = set.ascending().map(|index| u16::try_from(index).unwrap());

		self.locals.extend(iter);
		self.ranges[usize::from(id)] = (start, self.locals.len().try_into().unwrap());
	}
}

impl Default for Locals {
	fn default() -> Self {
		Self::new()
	}
}

pub struct LocalTracker {
	reads: Set,
	count: u16,
}

impl LocalTracker {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			reads: Set::new(),
			count: 0,
		}
	}

	fn read_local(&mut self, local: u16) {
		self.count = self.count.max(local + 1);

		self.reads.grow_insert(local.into());
	}

	fn write_local(&mut self, local: u16) {
		self.count = self.count.max(local + 1);

		self.reads.remove(local.into());
	}

	fn read_successors_in(&mut self, locals: &Locals, graph: &ControlFlowGraph, id: u16) {
		for successor in graph.successors_acyclic(id) {
			self.read_other_in(locals, successor);
		}
	}

	fn read_other_in(&mut self, locals: &Locals, id: u16) {
		let live = locals.get(id).iter().copied().map(usize::from);

		self.reads.extend(live);
	}

	fn handle_start(&mut self, locals: &mut Locals, graph: &ControlFlowGraph, id: u16) {
		let should_store = if graph.find_repeat_end(id).is_some() {
			self.read_other_in(locals, id);

			true
		} else {
			graph.is_branch_end(id) || graph.find_branch_start(id).is_some()
		};

		if should_store {
			locals.insert(id, self.reads.as_slice());
		}
	}

	fn handle_end(&mut self, locals: &mut Locals, graph: &ControlFlowGraph, id: u16) {
		if graph.is_branch_start(id) {
			self.read_successors_in(locals, graph, id);

			return;
		}

		if let Some(end) = graph.find_branch_end(id) {
			self.reads.clear();
			self.read_other_in(locals, end);
		}

		if let Some(start) = graph.find_repeat_start(id) {
			self.read_other_in(locals, start);

			locals.insert(start, self.reads.as_slice());
		}
	}

	fn handle_local_set(&mut self, local_set: LocalSet) {
		let LocalSet {
			destination,
			source,
		} = local_set;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_local_branch(&mut self, local_branch: LocalBranch) {
		let LocalBranch { source } = local_branch;

		self.read_local(source);
	}

	fn handle_i32_constant(&mut self, i32_constant: I32Constant) {
		let I32Constant {
			destination,
			data: _,
		} = i32_constant;

		self.write_local(destination);
	}

	fn handle_i64_constant(&mut self, i64_constant: I64Constant) {
		let I64Constant {
			destination,
			data: _,
		} = i64_constant;

		self.write_local(destination);
	}

	fn handle_f32_constant(&mut self, f32_constant: F32Constant) {
		let F32Constant {
			destination,
			data: _,
		} = f32_constant;

		self.write_local(destination);
	}

	fn handle_f64_constant(&mut self, f64_constant: F64Constant) {
		let F64Constant {
			destination,
			data: _,
		} = f64_constant;

		self.write_local(destination);
	}

	fn handle_ref_is_null(&mut self, ref_is_null: RefIsNull) {
		let RefIsNull {
			destination,
			source,
		} = ref_is_null;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_ref_null(&mut self, ref_null: RefNull) {
		let RefNull { destination } = ref_null;

		self.write_local(destination);
	}

	fn handle_ref_function(&mut self, ref_function: RefFunction) {
		let RefFunction {
			destination,
			function: _,
		} = ref_function;

		self.write_local(destination);
	}

	fn handle_call(&mut self, call: Call) {
		let Call {
			destinations,
			sources,
			function,
		} = call;

		for destination in destinations.0..destinations.1 {
			self.write_local(destination);
		}

		for source in sources.0..sources.1 {
			self.read_local(source);
		}

		self.read_local(function);
	}

	fn handle_integer_unary_operation(&mut self, integer_unary_operation: IntegerUnaryOperation) {
		let IntegerUnaryOperation {
			destination,
			source,
			r#type: _,
			operator: _,
		} = integer_unary_operation;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_integer_binary_operation(
		&mut self,
		integer_binary_operation: IntegerBinaryOperation,
	) {
		let IntegerBinaryOperation {
			destination,
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = integer_binary_operation;

		self.write_local(destination);
		self.read_local(lhs);
		self.read_local(rhs);
	}

	fn handle_integer_compare_operation(
		&mut self,
		integer_compare_operation: IntegerCompareOperation,
	) {
		let IntegerCompareOperation {
			destination,
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = integer_compare_operation;

		self.write_local(destination);
		self.read_local(lhs);
		self.read_local(rhs);
	}

	fn handle_integer_narrow(&mut self, integer_narrow: IntegerNarrow) {
		let IntegerNarrow {
			destination,
			source,
		} = integer_narrow;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_integer_widen(&mut self, integer_widen: IntegerWiden) {
		let IntegerWiden {
			destination,
			source,
		} = integer_widen;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_integer_extend(&mut self, integer_extend: IntegerExtend) {
		let IntegerExtend {
			destination,
			source,
			r#type: _,
		} = integer_extend;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_integer_convert_to_number(
		&mut self,
		integer_convert_to_number: IntegerConvertToNumber,
	) {
		let IntegerConvertToNumber {
			destination,
			source,
			signed: _,
			to: _,
			from: _,
		} = integer_convert_to_number;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_integer_transmute_to_number(
		&mut self,
		integer_transmute_to_number: IntegerTransmuteToNumber,
	) {
		let IntegerTransmuteToNumber {
			destination,
			source,
			from: _,
		} = integer_transmute_to_number;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_number_unary_operation(&mut self, number_unary_operation: NumberUnaryOperation) {
		let NumberUnaryOperation {
			destination,
			source,
			r#type: _,
			operator: _,
		} = number_unary_operation;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_number_binary_operation(&mut self, number_binary_operation: NumberBinaryOperation) {
		let NumberBinaryOperation {
			destination,
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = number_binary_operation;

		self.write_local(destination);
		self.read_local(lhs);
		self.read_local(rhs);
	}

	fn handle_number_compare_operation(
		&mut self,
		number_compare_operation: NumberCompareOperation,
	) {
		let NumberCompareOperation {
			destination,
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = number_compare_operation;

		self.write_local(destination);
		self.read_local(lhs);
		self.read_local(rhs);
	}

	fn handle_number_narrow(&mut self, number_narrow: NumberNarrow) {
		let NumberNarrow {
			destination,
			source,
		} = number_narrow;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_number_widen(&mut self, number_widen: NumberWiden) {
		let NumberWiden {
			destination,
			source,
		} = number_widen;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_number_truncate_to_integer(
		&mut self,
		number_truncate_to_integer: NumberTruncateToInteger,
	) {
		let NumberTruncateToInteger {
			destination,
			source,
			signed: _,
			saturate: _,
			to: _,
			from: _,
		} = number_truncate_to_integer;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_number_transmute_to_integer(
		&mut self,
		number_transmute_to_integer: NumberTransmuteToInteger,
	) {
		let NumberTransmuteToInteger {
			destination,
			source,
			from: _,
		} = number_transmute_to_integer;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_global_get(&mut self, global_get: GlobalGet) {
		let GlobalGet {
			destination,
			source: _,
		} = global_get;

		self.write_local(destination);
	}

	fn handle_global_set(&mut self, global_set: GlobalSet) {
		let GlobalSet {
			destination: _,
			source,
		} = global_set;

		self.read_local(source);
	}

	fn handle_table_get(&mut self, table_get: TableGet) {
		let TableGet {
			destination,
			source,
		} = table_get;

		self.write_local(destination);
		self.read_local(source.offset);
	}

	fn handle_table_set(&mut self, table_set: TableSet) {
		let TableSet {
			destination,
			source,
		} = table_set;

		self.read_local(destination.offset);
		self.read_local(source);
	}

	fn handle_table_size(&mut self, table_size: TableSize) {
		let TableSize {
			reference: _,
			destination,
		} = table_size;

		self.write_local(destination);
	}

	fn handle_table_grow(&mut self, table_grow: TableGrow) {
		let TableGrow {
			reference: _,
			destination,
			size,
			initializer,
		} = table_grow;

		self.write_local(destination);
		self.read_local(size);
		self.read_local(initializer);
	}

	fn handle_table_fill(&mut self, table_fill: TableFill) {
		let TableFill {
			destination,
			source,
			size,
		} = table_fill;

		self.read_local(destination.offset);
		self.read_local(source);
		self.read_local(size);
	}

	fn handle_table_copy(&mut self, table_copy: TableCopy) {
		let TableCopy {
			destination,
			source,
			size,
		} = table_copy;

		self.read_local(destination.offset);
		self.read_local(source.offset);
		self.read_local(size);
	}

	fn handle_table_init(&mut self, table_init: TableInit) {
		let TableInit {
			destination,
			source,
			size,
		} = table_init;

		self.read_local(destination.offset);
		self.read_local(source.offset);
		self.read_local(size);
	}

	fn handle_memory_load(&mut self, memory_load: MemoryLoad) {
		let MemoryLoad {
			destination,
			source,
			r#type: _,
		} = memory_load;

		self.write_local(destination);
		self.read_local(source.offset);
	}

	fn handle_memory_store(&mut self, memory_store: MemoryStore) {
		let MemoryStore {
			destination,
			source,
			r#type: _,
		} = memory_store;

		self.read_local(destination.offset);
		self.read_local(source);
	}

	fn handle_memory_size(&mut self, memory_size: MemorySize) {
		let MemorySize {
			reference: _,
			destination,
		} = memory_size;

		self.write_local(destination);
	}

	fn handle_memory_grow(&mut self, memory_grow: MemoryGrow) {
		let MemoryGrow {
			reference: _,
			destination,
			size,
		} = memory_grow;

		self.write_local(destination);
		self.read_local(size);
	}

	fn handle_memory_fill(&mut self, memory_fill: MemoryFill) {
		let MemoryFill {
			destination,
			byte,
			size,
		} = memory_fill;

		self.read_local(destination.offset);
		self.read_local(byte);
		self.read_local(size);
	}

	fn handle_memory_copy(&mut self, memory_copy: MemoryCopy) {
		let MemoryCopy {
			destination,
			source,
			size,
		} = memory_copy;

		self.read_local(destination.offset);
		self.read_local(source.offset);
		self.read_local(size);
	}

	fn handle_memory_init(&mut self, memory_init: MemoryInit) {
		let MemoryInit {
			destination,
			source,
			size,
		} = memory_init;

		self.read_local(destination.offset);
		self.read_local(source.offset);
		self.read_local(size);
	}

	fn handle_instruction(&mut self, instruction: Instruction) {
		match instruction {
			Instruction::Unreachable | Instruction::ElementsDrop(_) | Instruction::DataDrop(_) => {}
			Instruction::LocalSet(local_set) => self.handle_local_set(local_set),
			Instruction::LocalBranch(local_branch) => self.handle_local_branch(local_branch),
			Instruction::I32Constant(i32_constant) => self.handle_i32_constant(i32_constant),
			Instruction::I64Constant(i64_constant) => self.handle_i64_constant(i64_constant),
			Instruction::F32Constant(f32_constant) => self.handle_f32_constant(f32_constant),
			Instruction::F64Constant(f64_constant) => self.handle_f64_constant(f64_constant),
			Instruction::RefIsNull(ref_is_null) => self.handle_ref_is_null(ref_is_null),
			Instruction::RefNull(ref_null) => self.handle_ref_null(ref_null),
			Instruction::RefFunction(ref_function) => self.handle_ref_function(ref_function),
			Instruction::Call(call) => self.handle_call(call),
			Instruction::IntegerUnaryOperation(integer_unary_operation) => {
				self.handle_integer_unary_operation(integer_unary_operation);
			}
			Instruction::IntegerBinaryOperation(integer_binary_operation) => {
				self.handle_integer_binary_operation(integer_binary_operation);
			}
			Instruction::IntegerCompareOperation(integer_compare_operation) => {
				self.handle_integer_compare_operation(integer_compare_operation);
			}
			Instruction::IntegerNarrow(integer_narrow) => {
				self.handle_integer_narrow(integer_narrow);
			}
			Instruction::IntegerWiden(integer_widen) => self.handle_integer_widen(integer_widen),
			Instruction::IntegerExtend(integer_extend) => {
				self.handle_integer_extend(integer_extend);
			}
			Instruction::IntegerConvertToNumber(integer_convert_to_number) => {
				self.handle_integer_convert_to_number(integer_convert_to_number);
			}
			Instruction::IntegerTransmuteToNumber(integer_transmute_to_number) => {
				self.handle_integer_transmute_to_number(integer_transmute_to_number);
			}
			Instruction::NumberUnaryOperation(number_unary_operation) => {
				self.handle_number_unary_operation(number_unary_operation);
			}
			Instruction::NumberBinaryOperation(number_binary_operation) => {
				self.handle_number_binary_operation(number_binary_operation);
			}
			Instruction::NumberCompareOperation(number_compare_operation) => {
				self.handle_number_compare_operation(number_compare_operation);
			}
			Instruction::NumberNarrow(number_narrow) => self.handle_number_narrow(number_narrow),
			Instruction::NumberWiden(number_widen) => self.handle_number_widen(number_widen),
			Instruction::NumberTruncateToInteger(number_truncate_to_integer) => {
				self.handle_number_truncate_to_integer(number_truncate_to_integer);
			}
			Instruction::NumberTransmuteToInteger(number_transmute_to_integer) => {
				self.handle_number_transmute_to_integer(number_transmute_to_integer);
			}
			Instruction::GlobalGet(global_get) => self.handle_global_get(global_get),
			Instruction::GlobalSet(global_set) => self.handle_global_set(global_set),
			Instruction::TableGet(table_get) => self.handle_table_get(table_get),
			Instruction::TableSet(table_set) => self.handle_table_set(table_set),
			Instruction::TableSize(table_size) => self.handle_table_size(table_size),
			Instruction::TableGrow(table_grow) => self.handle_table_grow(table_grow),
			Instruction::TableFill(table_fill) => self.handle_table_fill(table_fill),
			Instruction::TableCopy(table_copy) => self.handle_table_copy(table_copy),
			Instruction::TableInit(table_init) => self.handle_table_init(table_init),
			Instruction::MemoryLoad(memory_load) => self.handle_memory_load(memory_load),
			Instruction::MemoryStore(memory_store) => self.handle_memory_store(memory_store),
			Instruction::MemorySize(memory_size) => self.handle_memory_size(memory_size),
			Instruction::MemoryGrow(memory_grow) => self.handle_memory_grow(memory_grow),
			Instruction::MemoryFill(memory_fill) => self.handle_memory_fill(memory_fill),
			Instruction::MemoryCopy(memory_copy) => self.handle_memory_copy(memory_copy),
			Instruction::MemoryInit(memory_init) => self.handle_memory_init(memory_init),
		}
	}

	fn handle_instructions(&mut self, instructions: &[Instruction]) {
		for &instruction in instructions.iter().rev() {
			self.handle_instruction(instruction);
		}
	}

	fn handle_all(&mut self, locals: &mut Locals, graph: &ControlFlowGraph, results: u16) {
		self.reads.clear();

		for result in 0..results {
			self.read_local(result + Name::COUNT);
		}

		for id in graph.block_ids().rev() {
			self.handle_end(locals, graph, id);
			self.handle_instructions(graph.instructions(id));
			self.handle_start(locals, graph, id);
		}
	}

	pub fn run(&mut self, locals: &mut Locals, graph: &ControlFlowGraph, results: u16) -> u16 {
		locals.set_len(graph.basic_blocks.len());

		self.count = 0;

		self.handle_all(locals, graph, results);

		if graph.has_repeats() {
			self.handle_all(locals, graph, results);
		}

		locals.insert(0, self.reads.as_slice());

		self.count
	}
}

impl Default for LocalTracker {
	fn default() -> Self {
		Self::new()
	}
}
