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

	fn handle_local_set(&mut self, instruction: LocalSet) {
		let LocalSet {
			destination,
			source,
		} = instruction;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_local_branch(&mut self, instruction: LocalBranch) {
		let LocalBranch { source } = instruction;

		self.read_local(source);
	}

	fn handle_i32_constant(&mut self, instruction: I32Constant) {
		let I32Constant { destination, .. } = instruction;

		self.write_local(destination);
	}

	fn handle_i64_constant(&mut self, instruction: I64Constant) {
		let I64Constant { destination, .. } = instruction;

		self.write_local(destination);
	}

	fn handle_f32_constant(&mut self, instruction: F32Constant) {
		let F32Constant { destination, .. } = instruction;

		self.write_local(destination);
	}

	fn handle_f64_constant(&mut self, instruction: F64Constant) {
		let F64Constant { destination, .. } = instruction;

		self.write_local(destination);
	}

	fn handle_ref_is_null(&mut self, instruction: RefIsNull) {
		let RefIsNull {
			destination,
			source,
		} = instruction;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_ref_null(&mut self, instruction: RefNull) {
		let RefNull { destination } = instruction;

		self.write_local(destination);
	}

	fn handle_ref_function(&mut self, instruction: RefFunction) {
		let RefFunction { destination, .. } = instruction;

		self.write_local(destination);
	}

	fn handle_call(&mut self, instruction: Call) {
		let Call {
			destinations,
			sources,
			function,
		} = instruction;

		for destination in destinations.0..destinations.1 {
			self.write_local(destination);
		}

		for source in sources.0..sources.1 {
			self.read_local(source);
		}

		self.read_local(function);
	}

	fn handle_integer_unary_operation(&mut self, instruction: IntegerUnaryOperation) {
		let IntegerUnaryOperation {
			destination,
			source,
			..
		} = instruction;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_integer_binary_operation(&mut self, instruction: IntegerBinaryOperation) {
		let IntegerBinaryOperation {
			destination,
			lhs,
			rhs,
			..
		} = instruction;

		self.write_local(destination);
		self.read_local(lhs);
		self.read_local(rhs);
	}

	fn handle_integer_compare_operation(&mut self, instruction: IntegerCompareOperation) {
		let IntegerCompareOperation {
			destination,
			lhs,
			rhs,
			..
		} = instruction;

		self.write_local(destination);
		self.read_local(lhs);
		self.read_local(rhs);
	}

	fn handle_integer_narrow(&mut self, instruction: IntegerNarrow) {
		let IntegerNarrow {
			destination,
			source,
		} = instruction;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_integer_widen(&mut self, instruction: IntegerWiden) {
		let IntegerWiden {
			destination,
			source,
		} = instruction;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_integer_extend(&mut self, instruction: IntegerExtend) {
		let IntegerExtend {
			destination,
			source,
			..
		} = instruction;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_integer_convert_to_number(&mut self, instruction: IntegerConvertToNumber) {
		let IntegerConvertToNumber {
			destination,
			source,
			..
		} = instruction;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_integer_transmute_to_number(&mut self, instruction: IntegerTransmuteToNumber) {
		let IntegerTransmuteToNumber {
			destination,
			source,
			..
		} = instruction;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_number_unary_operation(&mut self, instruction: NumberUnaryOperation) {
		let NumberUnaryOperation {
			destination,
			source,
			..
		} = instruction;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_number_binary_operation(&mut self, instruction: NumberBinaryOperation) {
		let NumberBinaryOperation {
			destination,
			lhs,
			rhs,
			..
		} = instruction;

		self.write_local(destination);
		self.read_local(lhs);
		self.read_local(rhs);
	}

	fn handle_number_compare_operation(&mut self, instruction: NumberCompareOperation) {
		let NumberCompareOperation {
			destination,
			lhs,
			rhs,
			..
		} = instruction;

		self.write_local(destination);
		self.read_local(lhs);
		self.read_local(rhs);
	}

	fn handle_number_narrow(&mut self, instruction: NumberNarrow) {
		let NumberNarrow {
			destination,
			source,
		} = instruction;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_number_widen(&mut self, instruction: NumberWiden) {
		let NumberWiden {
			destination,
			source,
		} = instruction;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_number_truncate_to_integer(&mut self, instruction: NumberTruncateToInteger) {
		let NumberTruncateToInteger {
			destination,
			source,
			..
		} = instruction;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_number_transmute_to_integer(&mut self, instruction: NumberTransmuteToInteger) {
		let NumberTransmuteToInteger {
			destination,
			source,
			..
		} = instruction;

		self.write_local(destination);
		self.read_local(source);
	}

	fn handle_global_get(&mut self, instruction: GlobalGet) {
		let GlobalGet { destination, .. } = instruction;

		self.write_local(destination);
	}

	fn handle_global_set(&mut self, instruction: GlobalSet) {
		let GlobalSet { source, .. } = instruction;

		self.read_local(source);
	}

	fn handle_table_get(&mut self, instruction: TableGet) {
		let TableGet {
			destination,
			source,
		} = instruction;

		self.write_local(destination);
		self.read_local(source.offset);
	}

	fn handle_table_set(&mut self, instruction: TableSet) {
		let TableSet {
			destination,
			source,
		} = instruction;

		self.read_local(destination.offset);
		self.read_local(source);
	}

	fn handle_table_size(&mut self, instruction: TableSize) {
		let TableSize { destination, .. } = instruction;

		self.write_local(destination);
	}

	fn handle_table_grow(&mut self, instruction: TableGrow) {
		let TableGrow {
			destination,
			size,
			initializer,
			..
		} = instruction;

		self.write_local(destination);
		self.read_local(size);
		self.read_local(initializer);
	}

	fn handle_table_fill(&mut self, instruction: TableFill) {
		let TableFill {
			destination,
			source,
			size,
		} = instruction;

		self.read_local(destination.offset);
		self.read_local(source);
		self.read_local(size);
	}

	fn handle_table_copy(&mut self, instruction: TableCopy) {
		let TableCopy {
			destination,
			source,
			size,
		} = instruction;

		self.read_local(destination.offset);
		self.read_local(source.offset);
		self.read_local(size);
	}

	fn handle_table_init(&mut self, instruction: TableInit) {
		let TableInit {
			destination,
			source,
			size,
		} = instruction;

		self.read_local(destination.offset);
		self.read_local(source.offset);
		self.read_local(size);
	}

	fn handle_memory_load(&mut self, instruction: MemoryLoad) {
		let MemoryLoad {
			destination,
			source,
			..
		} = instruction;

		self.write_local(destination);
		self.read_local(source.offset);
	}

	fn handle_memory_store(&mut self, instruction: MemoryStore) {
		let MemoryStore {
			destination,
			source,
			..
		} = instruction;

		self.read_local(destination.offset);
		self.read_local(source);
	}

	fn handle_memory_size(&mut self, instruction: MemorySize) {
		let MemorySize { destination, .. } = instruction;

		self.write_local(destination);
	}

	fn handle_memory_grow(&mut self, instruction: MemoryGrow) {
		let MemoryGrow {
			destination, size, ..
		} = instruction;

		self.write_local(destination);
		self.read_local(size);
	}

	fn handle_memory_fill(&mut self, instruction: MemoryFill) {
		let MemoryFill {
			destination,
			byte,
			size,
		} = instruction;

		self.read_local(destination.offset);
		self.read_local(byte);
		self.read_local(size);
	}

	fn handle_memory_copy(&mut self, instruction: MemoryCopy) {
		let MemoryCopy {
			destination,
			source,
			size,
		} = instruction;

		self.read_local(destination.offset);
		self.read_local(source.offset);
		self.read_local(size);
	}

	fn handle_memory_init(&mut self, instruction: MemoryInit) {
		let MemoryInit {
			destination,
			source,
			size,
		} = instruction;

		self.read_local(destination.offset);
		self.read_local(source.offset);
		self.read_local(size);
	}

	fn handle_instruction(&mut self, instruction: Instruction) {
		match instruction {
			Instruction::Unreachable | Instruction::ElementsDrop(_) | Instruction::DataDrop(_) => {}
			Instruction::LocalSet(instruction) => self.handle_local_set(instruction),
			Instruction::LocalBranch(instruction) => self.handle_local_branch(instruction),
			Instruction::I32Constant(instruction) => self.handle_i32_constant(instruction),
			Instruction::I64Constant(instruction) => self.handle_i64_constant(instruction),
			Instruction::F32Constant(instruction) => self.handle_f32_constant(instruction),
			Instruction::F64Constant(instruction) => self.handle_f64_constant(instruction),
			Instruction::RefIsNull(instruction) => self.handle_ref_is_null(instruction),
			Instruction::RefNull(instruction) => self.handle_ref_null(instruction),
			Instruction::RefFunction(instruction) => self.handle_ref_function(instruction),
			Instruction::Call(instruction) => self.handle_call(instruction),
			Instruction::IntegerUnaryOperation(instruction) => {
				self.handle_integer_unary_operation(instruction);
			}
			Instruction::IntegerBinaryOperation(instruction) => {
				self.handle_integer_binary_operation(instruction);
			}
			Instruction::IntegerCompareOperation(instruction) => {
				self.handle_integer_compare_operation(instruction);
			}
			Instruction::IntegerNarrow(instruction) => self.handle_integer_narrow(instruction),
			Instruction::IntegerWiden(instruction) => self.handle_integer_widen(instruction),
			Instruction::IntegerExtend(instruction) => self.handle_integer_extend(instruction),
			Instruction::IntegerConvertToNumber(instruction) => {
				self.handle_integer_convert_to_number(instruction);
			}
			Instruction::IntegerTransmuteToNumber(instruction) => {
				self.handle_integer_transmute_to_number(instruction);
			}
			Instruction::NumberUnaryOperation(instruction) => {
				self.handle_number_unary_operation(instruction);
			}
			Instruction::NumberBinaryOperation(instruction) => {
				self.handle_number_binary_operation(instruction);
			}
			Instruction::NumberCompareOperation(instruction) => {
				self.handle_number_compare_operation(instruction);
			}
			Instruction::NumberNarrow(instruction) => self.handle_number_narrow(instruction),
			Instruction::NumberWiden(instruction) => self.handle_number_widen(instruction),
			Instruction::NumberTruncateToInteger(instruction) => {
				self.handle_number_truncate_to_integer(instruction);
			}
			Instruction::NumberTransmuteToInteger(instruction) => {
				self.handle_number_transmute_to_integer(instruction);
			}
			Instruction::GlobalGet(instruction) => self.handle_global_get(instruction),
			Instruction::GlobalSet(instruction) => self.handle_global_set(instruction),
			Instruction::TableGet(instruction) => self.handle_table_get(instruction),
			Instruction::TableSet(instruction) => self.handle_table_set(instruction),
			Instruction::TableSize(instruction) => self.handle_table_size(instruction),
			Instruction::TableGrow(instruction) => self.handle_table_grow(instruction),
			Instruction::TableFill(instruction) => self.handle_table_fill(instruction),
			Instruction::TableCopy(instruction) => self.handle_table_copy(instruction),
			Instruction::TableInit(instruction) => self.handle_table_init(instruction),
			Instruction::MemoryLoad(instruction) => self.handle_memory_load(instruction),
			Instruction::MemoryStore(instruction) => self.handle_memory_store(instruction),
			Instruction::MemorySize(instruction) => self.handle_memory_size(instruction),
			Instruction::MemoryGrow(instruction) => self.handle_memory_grow(instruction),
			Instruction::MemoryFill(instruction) => self.handle_memory_fill(instruction),
			Instruction::MemoryCopy(instruction) => self.handle_memory_copy(instruction),
			Instruction::MemoryInit(instruction) => self.handle_memory_init(instruction),
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
