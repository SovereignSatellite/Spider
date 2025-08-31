use alloc::vec::Vec;
use control_flow_graph::{
	instruction::{
		Call, DataDrop, ElementsDrop, ExtendType, F32Constant, F64Constant, GlobalGet, GlobalSet,
		I32Constant, I64Constant, Instruction, IntegerBinaryOperation, IntegerBinaryOperator,
		IntegerCompareOperation, IntegerCompareOperator, IntegerConvertToNumber, IntegerExtend,
		IntegerNarrow, IntegerTransmuteToNumber, IntegerType, IntegerUnaryOperation,
		IntegerUnaryOperator, IntegerWiden, LoadType, LocalBranch, LocalSet, Location, MemoryCopy,
		MemoryFill, MemoryGrow, MemoryInit, MemoryLoad, MemorySize, MemoryStore,
		NumberBinaryOperation, NumberBinaryOperator, NumberCompareOperation, NumberCompareOperator,
		NumberNarrow, NumberTransmuteToInteger, NumberTruncateToInteger, NumberType,
		NumberUnaryOperation, NumberUnaryOperator, NumberWiden, RefFunction, RefIsNull, RefNull,
		StoreType, TableCopy, TableFill, TableGet, TableGrow, TableInit, TableSet, TableSize,
	},
	BasicBlock, ControlFlowGraph,
};
use list::resizable::Resizable;

use crate::stack_builder::{Jump, Level};

fn fill_predecessors(basic_blocks: &mut [BasicBlock]) {
	for predecessor_usize in 0..basic_blocks.len() {
		let predecessor = predecessor_usize.try_into().unwrap();
		let successors = core::mem::take(&mut basic_blocks[predecessor_usize].successors);

		for &successor in &successors {
			let successor_usize = usize::from(successor);

			basic_blocks[successor_usize].predecessors.push(predecessor);
		}

		basic_blocks[predecessor_usize].successors = successors;
	}
}

pub struct CodeBuilder {
	instructions: Vec<Instruction>,
	basic_blocks: Vec<BasicBlock>,

	position: u32,
}

impl CodeBuilder {
	pub const fn new() -> Self {
		Self {
			instructions: Vec::new(),
			basic_blocks: Vec::new(),

			position: 0,
		}
	}

	pub fn clear(&mut self) {
		self.instructions.clear();
		self.basic_blocks.clear();

		self.position = 0;
	}

	pub fn swap_contents(&mut self, graph: &mut ControlFlowGraph) {
		let ControlFlowGraph {
			instructions,
			basic_blocks,
		} = graph;

		fill_predecessors(&mut self.basic_blocks);

		core::mem::swap(&mut self.instructions, instructions);
		core::mem::swap(&mut self.basic_blocks, basic_blocks);
	}

	pub fn add_basic_block(&mut self, successors: usize) -> u16 {
		let basic_blocks = self.basic_blocks.len().try_into().unwrap();
		let instructions = self.instructions.len().try_into().unwrap();

		self.basic_blocks.push(BasicBlock {
			predecessors: Resizable::new(),
			successors: core::iter::repeat_n(basic_blocks + 1, successors).collect(),

			start: self.position,
			end: instructions,
		});

		self.position = instructions;

		basic_blocks
	}

	pub fn add_local_set(&mut self, destination: u16, source: u16) {
		let local_set = Instruction::LocalSet(LocalSet {
			destination,
			source,
		});

		self.instructions.push(local_set);
	}

	pub fn add_locals_set(&mut self, destination: u16, source: u16, count: u16) {
		if destination <= source {
			for offset in 0..count {
				self.add_local_set(destination + offset, source + offset);
			}
		} else {
			for offset in (0..count).rev() {
				self.add_local_set(destination + offset, source + offset);
			}
		}
	}

	pub fn add_local_branch(&mut self, source: u16, successors: usize) -> u16 {
		let branch = Instruction::LocalBranch(LocalBranch { source });

		self.instructions.push(branch);

		self.add_basic_block(successors)
	}

	pub fn try_add_stack_adjustment(&mut self, base: u16, top: u16, count: u16) -> bool {
		let source = top.wrapping_sub(count);

		if base == source || top == u16::MAX {
			return false;
		}

		self.add_locals_set(base, source, count);

		true
	}

	pub fn add_i32_constant(&mut self, destination: u16, data: i32) {
		let i32 = Instruction::I32Constant(I32Constant { destination, data });

		self.instructions.push(i32);
	}

	pub fn add_i64_constant(&mut self, destination: u16, data: i64) {
		let i64 = Instruction::I64Constant(I64Constant { destination, data });

		self.instructions.push(i64);
	}

	pub fn add_f32_constant(&mut self, destination: u16, data: f32) {
		let f32 = Instruction::F32Constant(F32Constant { destination, data });

		self.instructions.push(f32);
	}

	pub fn add_f64_constant(&mut self, destination: u16, data: f64) {
		let f64 = Instruction::F64Constant(F64Constant { destination, data });

		self.instructions.push(f64);
	}

	pub fn add_unreachable(&mut self) -> u16 {
		self.instructions.push(Instruction::Unreachable);

		self.add_basic_block(1)
	}

	pub fn add_call(&mut self, destinations: (u16, u16), sources: (u16, u16), function: u16) {
		let call = Instruction::Call(Call {
			destinations,
			sources,
			function,
		});

		self.instructions.push(call);
	}

	pub fn add_ref_is_null(&mut self, destination: u16, source: u16) {
		let ref_is_null = Instruction::RefIsNull(RefIsNull {
			destination,
			source,
		});

		self.instructions.push(ref_is_null);
	}

	pub fn add_ref_null(&mut self, destination: u16) {
		let ref_null = Instruction::RefNull(RefNull { destination });

		self.instructions.push(ref_null);
	}

	pub fn add_ref_function(&mut self, destination: u16, function: u16) {
		let ref_function = Instruction::RefFunction(RefFunction {
			destination,
			function,
		});

		self.instructions.push(ref_function);
	}

	pub fn add_integer_unary_operation(
		&mut self,
		destination: u16,
		source: u16,
		r#type: IntegerType,
		operator: IntegerUnaryOperator,
	) {
		let integer_unary_operation = Instruction::IntegerUnaryOperation(IntegerUnaryOperation {
			destination,
			source,
			r#type,
			operator,
		});

		self.instructions.push(integer_unary_operation);
	}

	pub fn add_integer_binary_operation(
		&mut self,
		destination: u16,
		lhs: u16,
		rhs: u16,
		r#type: IntegerType,
		operator: IntegerBinaryOperator,
	) {
		let integer_binary_operation =
			Instruction::IntegerBinaryOperation(IntegerBinaryOperation {
				destination,
				lhs,
				rhs,
				r#type,
				operator,
			});

		self.instructions.push(integer_binary_operation);
	}

	pub fn add_integer_compare_operation(
		&mut self,
		destination: u16,
		lhs: u16,
		rhs: u16,
		r#type: IntegerType,
		operator: IntegerCompareOperator,
	) {
		let integer_compare_operation =
			Instruction::IntegerCompareOperation(IntegerCompareOperation {
				destination,
				lhs,
				rhs,
				r#type,
				operator,
			});

		self.instructions.push(integer_compare_operation);
	}

	pub fn add_integer_narrow(&mut self, destination: u16, source: u16) {
		let integer_narrow = Instruction::IntegerNarrow(IntegerNarrow {
			destination,
			source,
		});

		self.instructions.push(integer_narrow);
	}

	pub fn add_integer_widen(&mut self, destination: u16, source: u16) {
		let integer_widen = Instruction::IntegerWiden(IntegerWiden {
			destination,
			source,
		});

		self.instructions.push(integer_widen);
	}

	pub fn add_integer_extend(&mut self, destination: u16, source: u16, r#type: ExtendType) {
		let integer_extend = Instruction::IntegerExtend(IntegerExtend {
			destination,
			source,
			r#type,
		});

		self.instructions.push(integer_extend);
	}

	pub fn add_integer_convert_to_number(
		&mut self,
		destination: u16,
		source: u16,
		signed: bool,
		to: NumberType,
		from: IntegerType,
	) {
		let integer_convert_to_number =
			Instruction::IntegerConvertToNumber(IntegerConvertToNumber {
				destination,
				source,
				signed,
				to,
				from,
			});

		self.instructions.push(integer_convert_to_number);
	}

	pub fn add_integer_transmute_to_number(
		&mut self,
		destination: u16,
		source: u16,
		from: IntegerType,
	) {
		let integer_transmute_to_number =
			Instruction::IntegerTransmuteToNumber(IntegerTransmuteToNumber {
				destination,
				source,
				from,
			});

		self.instructions.push(integer_transmute_to_number);
	}

	pub fn add_number_unary_operation(
		&mut self,
		destination: u16,
		source: u16,
		r#type: NumberType,
		operator: NumberUnaryOperator,
	) {
		let number_unary_operation = Instruction::NumberUnaryOperation(NumberUnaryOperation {
			destination,
			source,
			r#type,
			operator,
		});

		self.instructions.push(number_unary_operation);
	}

	pub fn add_number_binary_operation(
		&mut self,
		destination: u16,
		lhs: u16,
		rhs: u16,
		r#type: NumberType,
		operator: NumberBinaryOperator,
	) {
		let number_binary_operation = Instruction::NumberBinaryOperation(NumberBinaryOperation {
			destination,
			lhs,
			rhs,
			r#type,
			operator,
		});

		self.instructions.push(number_binary_operation);
	}

	pub fn add_number_compare_operation(
		&mut self,
		destination: u16,
		lhs: u16,
		rhs: u16,
		r#type: NumberType,
		operator: NumberCompareOperator,
	) {
		let number_compare_operation =
			Instruction::NumberCompareOperation(NumberCompareOperation {
				destination,
				lhs,
				rhs,
				r#type,
				operator,
			});

		self.instructions.push(number_compare_operation);
	}

	pub fn add_number_narrow(&mut self, destination: u16, source: u16) {
		let number_narrow = Instruction::NumberNarrow(NumberNarrow {
			destination,
			source,
		});

		self.instructions.push(number_narrow);
	}

	pub fn add_number_widen(&mut self, destination: u16, source: u16) {
		let number_widen = Instruction::NumberWiden(NumberWiden {
			destination,
			source,
		});

		self.instructions.push(number_widen);
	}

	pub fn add_number_truncate_to_integer(
		&mut self,
		destination: u16,
		source: u16,
		signed: bool,
		saturate: bool,
		to: IntegerType,
		from: NumberType,
	) {
		let number_truncate_to_integer =
			Instruction::NumberTruncateToInteger(NumberTruncateToInteger {
				destination,
				source,
				signed,
				saturate,
				to,
				from,
			});

		self.instructions.push(number_truncate_to_integer);
	}

	pub fn add_number_transmute_to_integer(
		&mut self,
		destination: u16,
		source: u16,
		from: NumberType,
	) {
		let number_transmute_to_integer =
			Instruction::NumberTransmuteToInteger(NumberTransmuteToInteger {
				destination,
				source,
				from,
			});

		self.instructions.push(number_transmute_to_integer);
	}

	pub fn add_global_get(&mut self, destination: u16, source: u16) {
		let global_get = Instruction::GlobalGet(GlobalGet {
			destination,
			source,
		});

		self.instructions.push(global_get);
	}

	pub fn add_global_set(&mut self, destination: u16, source: u16) {
		let global_set = Instruction::GlobalSet(GlobalSet {
			destination,
			source,
		});

		self.instructions.push(global_set);
	}

	pub fn add_table_get(&mut self, destination: u16, source: Location) {
		let table_get = Instruction::TableGet(TableGet {
			destination,
			source,
		});

		self.instructions.push(table_get);
	}

	pub fn add_table_set(&mut self, destination: Location, source: u16) {
		let table_set = Instruction::TableSet(TableSet {
			destination,
			source,
		});

		self.instructions.push(table_set);
	}

	pub fn add_table_size(&mut self, reference: u16, destination: u16) {
		let table_size = Instruction::TableSize(TableSize {
			reference,
			destination,
		});

		self.instructions.push(table_size);
	}

	pub fn add_table_grow(
		&mut self,
		reference: u16,
		destination: u16,
		size: u16,
		initializer: u16,
	) {
		let table_grow = Instruction::TableGrow(TableGrow {
			reference,
			destination,
			size,
			initializer,
		});

		self.instructions.push(table_grow);
	}

	pub fn add_table_fill(&mut self, destination: Location, source: u16, size: u16) {
		let table_fill = Instruction::TableFill(TableFill {
			destination,
			source,
			size,
		});

		self.instructions.push(table_fill);
	}

	pub fn add_table_copy(&mut self, destination: Location, source: Location, size: u16) {
		let table_copy = Instruction::TableCopy(TableCopy {
			destination,
			source,
			size,
		});

		self.instructions.push(table_copy);
	}

	pub fn add_table_init(&mut self, destination: Location, source: Location, size: u16) {
		let table_init = Instruction::TableInit(TableInit {
			destination,
			source,
			size,
		});

		self.instructions.push(table_init);
	}

	pub fn add_elements_drop(&mut self, source: u16) {
		let elements_drop = Instruction::ElementsDrop(ElementsDrop { source });

		self.instructions.push(elements_drop);
	}

	pub fn add_memory_load(&mut self, destination: u16, source: Location, r#type: LoadType) {
		let memory_load = Instruction::MemoryLoad(MemoryLoad {
			destination,
			source,
			r#type,
		});

		self.instructions.push(memory_load);
	}

	pub fn add_memory_store(&mut self, destination: Location, source: u16, r#type: StoreType) {
		let memory_store = Instruction::MemoryStore(MemoryStore {
			destination,
			source,
			r#type,
		});

		self.instructions.push(memory_store);
	}

	pub fn add_memory_size(&mut self, reference: u16, destination: u16) {
		let memory_size = Instruction::MemorySize(MemorySize {
			reference,
			destination,
		});

		self.instructions.push(memory_size);
	}

	pub fn add_memory_grow(&mut self, reference: u16, destination: u16, size: u16) {
		let memory_grow = Instruction::MemoryGrow(MemoryGrow {
			reference,
			destination,
			size,
		});

		self.instructions.push(memory_grow);
	}

	pub fn add_memory_fill(&mut self, destination: Location, byte: u16, size: u16) {
		let memory_fill = Instruction::MemoryFill(MemoryFill {
			destination,
			byte,
			size,
		});

		self.instructions.push(memory_fill);
	}

	pub fn add_memory_copy(&mut self, destination: Location, source: Location, size: u16) {
		let memory_copy = Instruction::MemoryCopy(MemoryCopy {
			destination,
			source,
			size,
		});

		self.instructions.push(memory_copy);
	}

	pub fn add_memory_init(&mut self, destination: Location, source: Location, size: u16) {
		let memory_init = Instruction::MemoryInit(MemoryInit {
			destination,
			source,
			size,
		});

		self.instructions.push(memory_init);
	}

	pub fn add_data_drop(&mut self, source: u16) {
		let data_drop = Instruction::DataDrop(DataDrop { source });

		self.instructions.push(data_drop);
	}

	pub fn set_jump_destination(&mut self, source: u16, branch: u16, destination: u16) {
		let source = usize::from(source);
		let branch = usize::from(branch);

		self.basic_blocks[source].successors[branch] = destination;
	}

	fn set_jump_destinations(&mut self, destination: u16, jumps: &[Jump]) {
		for &Jump { source, branch, .. } in jumps {
			self.set_jump_destination(source, branch, destination);
		}
	}

	fn add_jump_adjustments(&mut self, base: u16, parameters: u16, jumps: &mut [Jump]) {
		for Jump {
			stack,
			source,
			branch,
		} in jumps
		{
			if !self.try_add_stack_adjustment(base, *stack, parameters) {
				continue;
			}

			let destination = self.add_basic_block(1);

			self.set_jump_destination(*source, *branch, destination);

			*source = destination;
			*branch = 0;
		}
	}

	pub fn handle_level(&mut self, level: Level, top: u16) {
		let Level {
			parameters,
			results,
			base,
			destination,
			mut jumps,
		} = level;

		self.try_add_stack_adjustment(base, top, results);

		let exit = self.add_basic_block(1);

		// Levels with destinations need to point to it, while levels
		// without it simply defer to the next basic block after all
		// adjustments have been completed.
		if let Some(destination) = destination {
			self.add_jump_adjustments(base, parameters, &mut jumps);
			self.set_jump_destinations(destination, &jumps);
		} else {
			self.add_jump_adjustments(base, results, &mut jumps);

			let destination = self.basic_blocks.len().try_into().unwrap();

			self.set_jump_destinations(destination, &jumps);
		}

		// The base case always falls through to the next basic block.
		let destination = self.basic_blocks.len().try_into().unwrap();

		self.set_jump_destination(exit, 0, destination);
	}
}
