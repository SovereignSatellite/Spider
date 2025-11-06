use alloc::vec::Vec;
use control_flow_graph::{
	BasicBlock, ControlFlowGraph,
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
		let instruction = Instruction::LocalBranch(LocalBranch { source });

		self.instructions.push(instruction);

		self.add_basic_block(successors)
	}

	pub fn add_select(&mut self, destination: u16, condition: u16, on_false: u16, on_true: u16) {
		let condition = self.add_local_branch(condition, 2);

		self.add_local_set(destination, on_false);

		let on_false = self.add_basic_block(1);

		self.add_local_set(destination, on_true);

		let on_true = self.add_basic_block(1);

		self.set_jump_destination(on_false, 0, on_true + 1);
		self.set_jump_destination(condition, 0, on_false);
		self.set_jump_destination(condition, 1, on_true);
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
		let instruction = Instruction::I32Constant(I32Constant { destination, data });

		self.instructions.push(instruction);
	}

	pub fn add_i64_constant(&mut self, destination: u16, data: i64) {
		let instruction = Instruction::I64Constant(I64Constant { destination, data });

		self.instructions.push(instruction);
	}

	pub fn add_f32_constant(&mut self, destination: u16, data: f32) {
		let instruction = Instruction::F32Constant(F32Constant { destination, data });

		self.instructions.push(instruction);
	}

	pub fn add_f64_constant(&mut self, destination: u16, data: f64) {
		let instruction = Instruction::F64Constant(F64Constant { destination, data });

		self.instructions.push(instruction);
	}

	pub fn add_unreachable(&mut self) -> u16 {
		self.instructions.push(Instruction::Unreachable);

		self.add_basic_block(1)
	}

	pub fn add_call(&mut self, destinations: (u16, u16), sources: (u16, u16), function: u16) {
		let instruction = Instruction::Call(Call {
			destinations,
			sources,
			function,
		});

		self.instructions.push(instruction);
	}

	pub fn add_ref_is_null(&mut self, destination: u16, source: u16) {
		let instruction = Instruction::RefIsNull(RefIsNull {
			destination,
			source,
		});

		self.instructions.push(instruction);
	}

	pub fn add_ref_null(&mut self, destination: u16) {
		let instruction = Instruction::RefNull(RefNull { destination });

		self.instructions.push(instruction);
	}

	pub fn add_ref_function(&mut self, destination: u16, function: u16) {
		let instruction = Instruction::RefFunction(RefFunction {
			destination,
			function,
		});

		self.instructions.push(instruction);
	}

	pub fn add_integer_unary_operation(
		&mut self,
		destination: u16,
		source: u16,
		r#type: IntegerType,
		operator: IntegerUnaryOperator,
	) {
		let instruction = Instruction::IntegerUnaryOperation(IntegerUnaryOperation {
			destination,
			source,
			r#type,
			operator,
		});

		self.instructions.push(instruction);
	}

	pub fn add_integer_binary_operation(
		&mut self,
		destination: u16,
		lhs: u16,
		rhs: u16,
		r#type: IntegerType,
		operator: IntegerBinaryOperator,
	) {
		let instruction = Instruction::IntegerBinaryOperation(IntegerBinaryOperation {
			destination,
			lhs,
			rhs,
			r#type,
			operator,
		});

		self.instructions.push(instruction);
	}

	pub fn add_integer_compare_operation(
		&mut self,
		destination: u16,
		lhs: u16,
		rhs: u16,
		r#type: IntegerType,
		operator: IntegerCompareOperator,
	) {
		let instruction = Instruction::IntegerCompareOperation(IntegerCompareOperation {
			destination,
			lhs,
			rhs,
			r#type,
			operator,
		});

		self.instructions.push(instruction);
	}

	pub fn add_integer_narrow(&mut self, destination: u16, source: u16) {
		let instruction = Instruction::IntegerNarrow(IntegerNarrow {
			destination,
			source,
		});

		self.instructions.push(instruction);
	}

	pub fn add_integer_widen(&mut self, destination: u16, source: u16) {
		let instruction = Instruction::IntegerWiden(IntegerWiden {
			destination,
			source,
		});

		self.instructions.push(instruction);
	}

	pub fn add_integer_extend(&mut self, destination: u16, source: u16, r#type: ExtendType) {
		let instruction = Instruction::IntegerExtend(IntegerExtend {
			destination,
			source,
			r#type,
		});

		self.instructions.push(instruction);
	}

	pub fn add_integer_convert_to_number(
		&mut self,
		destination: u16,
		source: u16,
		signed: bool,
		to: NumberType,
		from: IntegerType,
	) {
		let instruction = Instruction::IntegerConvertToNumber(IntegerConvertToNumber {
			destination,
			source,
			signed,
			to,
			from,
		});

		self.instructions.push(instruction);
	}

	pub fn add_integer_transmute_to_number(
		&mut self,
		destination: u16,
		source: u16,
		from: IntegerType,
	) {
		let instruction = Instruction::IntegerTransmuteToNumber(IntegerTransmuteToNumber {
			destination,
			source,
			from,
		});

		self.instructions.push(instruction);
	}

	pub fn add_number_unary_operation(
		&mut self,
		destination: u16,
		source: u16,
		r#type: NumberType,
		operator: NumberUnaryOperator,
	) {
		let instruction = Instruction::NumberUnaryOperation(NumberUnaryOperation {
			destination,
			source,
			r#type,
			operator,
		});

		self.instructions.push(instruction);
	}

	pub fn add_number_binary_operation(
		&mut self,
		destination: u16,
		lhs: u16,
		rhs: u16,
		r#type: NumberType,
		operator: NumberBinaryOperator,
	) {
		let instruction = Instruction::NumberBinaryOperation(NumberBinaryOperation {
			destination,
			lhs,
			rhs,
			r#type,
			operator,
		});

		self.instructions.push(instruction);
	}

	pub fn add_number_compare_operation(
		&mut self,
		destination: u16,
		lhs: u16,
		rhs: u16,
		r#type: NumberType,
		operator: NumberCompareOperator,
	) {
		let instruction = Instruction::NumberCompareOperation(NumberCompareOperation {
			destination,
			lhs,
			rhs,
			r#type,
			operator,
		});

		self.instructions.push(instruction);
	}

	pub fn add_number_narrow(&mut self, destination: u16, source: u16) {
		let instruction = Instruction::NumberNarrow(NumberNarrow {
			destination,
			source,
		});

		self.instructions.push(instruction);
	}

	pub fn add_number_widen(&mut self, destination: u16, source: u16) {
		let instruction = Instruction::NumberWiden(NumberWiden {
			destination,
			source,
		});

		self.instructions.push(instruction);
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
		let instruction = Instruction::NumberTruncateToInteger(NumberTruncateToInteger {
			destination,
			source,
			signed,
			saturate,
			to,
			from,
		});

		self.instructions.push(instruction);
	}

	pub fn add_number_transmute_to_integer(
		&mut self,
		destination: u16,
		source: u16,
		from: NumberType,
	) {
		let instruction = Instruction::NumberTransmuteToInteger(NumberTransmuteToInteger {
			destination,
			source,
			from,
		});

		self.instructions.push(instruction);
	}

	pub fn add_global_get(&mut self, destination: u16, source: u16) {
		let instruction = Instruction::GlobalGet(GlobalGet {
			destination,
			source,
		});

		self.instructions.push(instruction);
	}

	pub fn add_global_set(&mut self, destination: u16, source: u16) {
		let instruction = Instruction::GlobalSet(GlobalSet {
			destination,
			source,
		});

		self.instructions.push(instruction);
	}

	pub fn add_table_get(&mut self, destination: u16, source: Location) {
		let instruction = Instruction::TableGet(TableGet {
			destination,
			source,
		});

		self.instructions.push(instruction);
	}

	pub fn add_table_set(&mut self, destination: Location, source: u16) {
		let instruction = Instruction::TableSet(TableSet {
			destination,
			source,
		});

		self.instructions.push(instruction);
	}

	pub fn add_table_size(&mut self, destination: u16, table: u16) {
		let instruction = Instruction::TableSize(TableSize { destination, table });

		self.instructions.push(instruction);
	}

	pub fn add_table_grow(&mut self, destination: u16, table: u16, size: u16, initializer: u16) {
		let instruction = Instruction::TableGrow(TableGrow {
			destination,
			table,
			size,
			initializer,
		});

		self.instructions.push(instruction);
	}

	pub fn add_table_fill(&mut self, destination: Location, source: u16, size: u16) {
		let instruction = Instruction::TableFill(TableFill {
			destination,
			source,
			size,
		});

		self.instructions.push(instruction);
	}

	pub fn add_table_copy(&mut self, destination: Location, source: Location, size: u16) {
		let instruction = Instruction::TableCopy(TableCopy {
			destination,
			source,
			size,
		});

		self.instructions.push(instruction);
	}

	pub fn add_table_init(&mut self, destination: Location, source: Location, size: u16) {
		let instruction = Instruction::TableInit(TableInit {
			destination,
			source,
			size,
		});

		self.instructions.push(instruction);
	}

	pub fn add_elements_drop(&mut self, source: u16) {
		let instruction = Instruction::ElementsDrop(ElementsDrop { source });

		self.instructions.push(instruction);
	}

	pub fn add_memory_load(&mut self, destination: u16, source: Location, r#type: LoadType) {
		let instruction = Instruction::MemoryLoad(MemoryLoad {
			destination,
			source,
			r#type,
		});

		self.instructions.push(instruction);
	}

	pub fn add_memory_store(&mut self, destination: Location, source: u16, r#type: StoreType) {
		let instruction = Instruction::MemoryStore(MemoryStore {
			destination,
			source,
			r#type,
		});

		self.instructions.push(instruction);
	}

	pub fn add_memory_size(&mut self, destination: u16, memory: u16) {
		let instruction = Instruction::MemorySize(MemorySize {
			destination,
			memory,
		});

		self.instructions.push(instruction);
	}

	pub fn add_memory_grow(&mut self, destination: u16, memory: u16, size: u16) {
		let instruction = Instruction::MemoryGrow(MemoryGrow {
			destination,
			memory,
			size,
		});

		self.instructions.push(instruction);
	}

	pub fn add_memory_fill(&mut self, destination: Location, byte: u16, size: u16) {
		let instruction = Instruction::MemoryFill(MemoryFill {
			destination,
			byte,
			size,
		});

		self.instructions.push(instruction);
	}

	pub fn add_memory_copy(&mut self, destination: Location, source: Location, size: u16) {
		let instruction = Instruction::MemoryCopy(MemoryCopy {
			destination,
			source,
			size,
		});

		self.instructions.push(instruction);
	}

	pub fn add_memory_init(&mut self, destination: Location, source: Location, size: u16) {
		let instruction = Instruction::MemoryInit(MemoryInit {
			destination,
			source,
			size,
		});

		self.instructions.push(instruction);
	}

	pub fn add_data_drop(&mut self, source: u16) {
		let instruction = Instruction::DataDrop(DataDrop { source });

		self.instructions.push(instruction);
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
