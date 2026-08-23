#![expect(
	clippy::too_many_arguments,
	reason = "control-flow instruction constructors mirror their struct fields"
)]

use core::{iter, mem};

use list::resizable::Resizable;

use ir_graph::operation::{ExtendType, LoadType, StoreType, integer, number};
use web_assembly_control_flow::{
	BasicBlock, ControlFlowGraph,
	instruction::{
		Call, DataDrop, ElementsDrop, F32Constant, F64Constant, GlobalGet, GlobalSet, I32Constant,
		I64Constant, Instruction, IntegerBinaryOperation, IntegerCompareOperation,
		IntegerConvertToNumber, IntegerExtend, IntegerNarrow, IntegerTransmuteToNumber,
		IntegerUnaryOperation, IntegerWiden, LocalBranch, LocalSet, Location, MemoryCopy,
		MemoryFill, MemoryGrow, MemoryInit, MemoryLoad, MemorySize, MemoryStore,
		NumberBinaryOperation, NumberCompareOperation, NumberNarrow, NumberTransmuteToInteger,
		NumberTruncateToInteger, NumberUnaryOperation, NumberWiden, RefFunction, RefIsNull,
		RefNull, TableCopy, TableFill, TableGet, TableGrow, TableInit, TableSet, TableSize,
	},
};

use super::{
	DECODER_SCRATCH,
	operand_stack::{ControlLevel, ControlLevelKind, PendingJump},
};

fn fill_predecessors(basic_blocks: &mut [BasicBlock]) {
	for predecessor_usize in 0..basic_blocks.len() {
		let predecessor = predecessor_usize.try_into().unwrap();
		let successors = mem::take(&mut basic_blocks[predecessor_usize].successors);

		for &successor in &successors {
			let successor_usize = usize::from(successor);

			basic_blocks[successor_usize].predecessors.push(predecessor);
		}

		basic_blocks[predecessor_usize].successors = successors;
	}
}

pub struct BlockBuilder {
	graph: ControlFlowGraph,
}

impl BlockBuilder {
	pub const fn new() -> Self {
		Self {
			graph: ControlFlowGraph::new(),
		}
	}

	pub fn clear(&mut self) {
		self.graph.instructions.clear();
		self.graph.basic_blocks.clear();
	}

	pub fn finish(&mut self) {
		fill_predecessors(&mut self.graph.basic_blocks);
	}

	pub const fn graph(&self) -> &ControlFlowGraph {
		&self.graph
	}

	pub const fn graph_mut(&mut self) -> &mut ControlFlowGraph {
		&mut self.graph
	}

	pub fn add_basic_block(&mut self, successors: usize) -> u16 {
		let basic_blocks = self.graph.basic_blocks.len().try_into().unwrap();
		let instructions = self.graph.instructions.len().try_into().unwrap();
		let start = self.graph.basic_blocks.last().map_or(0, |block| block.end);

		self.graph.basic_blocks.push(BasicBlock {
			predecessors: Resizable::new(),
			successors: iter::repeat_n(basic_blocks + 1, successors).collect(),
			start,
			end: instructions,
		});

		basic_blocks
	}

	pub fn add_local_set(&mut self, destination: u16, source: u16) {
		let local_set = Instruction::LocalSet(LocalSet {
			destination,
			source,
		});

		self.graph.instructions.push(local_set);
	}

	fn add_locals_set(&mut self, destination: u16, source: u16, count: u16) {
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

		self.graph.instructions.push(instruction);

		self.add_basic_block(successors)
	}

	pub fn add_if<OnFalse, OnTrue>(&mut self, condition: u16, on_false: OnFalse, on_true: OnTrue)
	where
		OnFalse: FnOnce(&mut Self),
		OnTrue: FnOnce(&mut Self),
	{
		let condition_id = self.add_local_branch(condition, 2);

		on_false(self);

		let false_id = self.add_basic_block(1);

		on_true(self);

		let true_id = self.add_basic_block(1);

		self.set_jump_destination(condition_id, 0, condition_id + 1);
		self.set_jump_destination(condition_id, 1, false_id + 1);
		self.set_jump_destination(false_id, 0, true_id + 1);
	}

	pub fn add_select(&mut self, destination: u16, condition: u16, on_false: u16, on_true: u16) {
		self.add_i32_compare_constant(
			DECODER_SCRATCH,
			condition,
			0,
			integer::CompareOperator::NotEqual,
		);

		self.add_if(
			DECODER_SCRATCH,
			|this| this.add_local_set(destination, on_false),
			|this| this.add_local_set(destination, on_true),
		);
	}

	fn try_add_stack_adjustment(&mut self, base: u16, top: u16, count: u16) -> bool {
		let source = top.wrapping_sub(count);

		if count == 0 || base == source || top == u16::MAX {
			return false;
		}

		self.add_locals_set(base, source, count);

		true
	}

	pub fn add_i32_constant(&mut self, destination: u16, data: i32) {
		let instruction = Instruction::I32Constant(I32Constant { destination, data });

		self.graph.instructions.push(instruction);
	}

	pub fn add_i64_constant(&mut self, destination: u16, data: i64) {
		let instruction = Instruction::I64Constant(I64Constant { destination, data });

		self.graph.instructions.push(instruction);
	}

	pub fn add_f32_constant(&mut self, destination: u16, data: f32) {
		let instruction = Instruction::F32Constant(F32Constant { destination, data });

		self.graph.instructions.push(instruction);
	}

	pub fn add_f64_constant(&mut self, destination: u16, data: f64) {
		let instruction = Instruction::F64Constant(F64Constant { destination, data });

		self.graph.instructions.push(instruction);
	}

	pub fn add_unreachable(&mut self) -> u16 {
		self.graph.instructions.push(Instruction::Unreachable);

		self.add_basic_block(1)
	}

	pub fn add_call(&mut self, destinations: (u16, u16), sources: (u16, u16), function: u16) {
		let instruction = Instruction::Call(Call {
			destinations,
			sources,
			function,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_ref_is_null(&mut self, destination: u16, source: u16) {
		let instruction = Instruction::RefIsNull(RefIsNull {
			destination,
			source,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_ref_null(&mut self, destination: u16) {
		let instruction = Instruction::RefNull(RefNull { destination });

		self.graph.instructions.push(instruction);
	}

	pub fn add_ref_function(&mut self, destination: u16, function: u16) {
		let instruction = Instruction::RefFunction(RefFunction {
			destination,
			function,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_integer_unary_operation(
		&mut self,
		destination: u16,
		source: u16,
		kind: integer::Type,
		operator: integer::UnaryOperator,
	) {
		let instruction = Instruction::IntegerUnaryOperation(IntegerUnaryOperation {
			destination,
			source,
			kind,
			operator,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_integer_binary_operation(
		&mut self,
		destination: u16,
		lhs: u16,
		rhs: u16,
		kind: integer::Type,
		operator: integer::BinaryOperator,
	) {
		let instruction = Instruction::IntegerBinaryOperation(IntegerBinaryOperation {
			destination,
			lhs,
			rhs,
			kind,
			operator,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_integer_compare_operation(
		&mut self,
		destination: u16,
		lhs: u16,
		rhs: u16,
		kind: integer::Type,
		operator: integer::CompareOperator,
	) {
		let instruction = Instruction::IntegerCompareOperation(IntegerCompareOperation {
			destination,
			lhs,
			rhs,
			kind,
			operator,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_i32_compare_constant(
		&mut self,
		destination: u16,
		lhs: u16,
		rhs: i32,
		operator: integer::CompareOperator,
	) {
		self.add_i32_constant(DECODER_SCRATCH, rhs);
		self.add_integer_compare_operation(
			destination,
			lhs,
			DECODER_SCRATCH,
			integer::Type::I32,
			operator,
		);
	}

	pub fn add_integer_narrow(&mut self, destination: u16, source: u16) {
		let instruction = Instruction::IntegerNarrow(IntegerNarrow {
			destination,
			source,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_integer_widen(&mut self, destination: u16, source: u16) {
		let instruction = Instruction::IntegerWiden(IntegerWiden {
			destination,
			source,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_integer_extend(&mut self, destination: u16, source: u16, kind: ExtendType) {
		let instruction = Instruction::IntegerExtend(IntegerExtend {
			destination,
			source,
			kind,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_integer_convert_to_number(
		&mut self,
		destination: u16,
		source: u16,
		is_signed: bool,
		to: number::Type,
		from: integer::Type,
	) {
		let instruction = Instruction::IntegerConvertToNumber(IntegerConvertToNumber {
			destination,
			source,
			is_signed,
			to,
			from,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_integer_transmute_to_number(
		&mut self,
		destination: u16,
		source: u16,
		from: integer::Type,
	) {
		let instruction = Instruction::IntegerTransmuteToNumber(IntegerTransmuteToNumber {
			destination,
			source,
			from,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_number_unary_operation(
		&mut self,
		destination: u16,
		source: u16,
		kind: number::Type,
		operator: number::UnaryOperator,
	) {
		let instruction = Instruction::NumberUnaryOperation(NumberUnaryOperation {
			destination,
			source,
			kind,
			operator,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_number_binary_operation(
		&mut self,
		destination: u16,
		lhs: u16,
		rhs: u16,
		kind: number::Type,
		operator: number::BinaryOperator,
	) {
		let instruction = Instruction::NumberBinaryOperation(NumberBinaryOperation {
			destination,
			lhs,
			rhs,
			kind,
			operator,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_number_compare_operation(
		&mut self,
		destination: u16,
		lhs: u16,
		rhs: u16,
		kind: number::Type,
		operator: number::CompareOperator,
	) {
		let instruction = Instruction::NumberCompareOperation(NumberCompareOperation {
			destination,
			lhs,
			rhs,
			kind,
			operator,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_number_narrow(&mut self, destination: u16, source: u16) {
		let instruction = Instruction::NumberNarrow(NumberNarrow {
			destination,
			source,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_number_widen(&mut self, destination: u16, source: u16) {
		let instruction = Instruction::NumberWiden(NumberWiden {
			destination,
			source,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_number_truncate_to_integer(
		&mut self,
		destination: u16,
		source: u16,
		is_signed: bool,
		is_saturating: bool,
		to: integer::Type,
		from: number::Type,
	) {
		let instruction = Instruction::NumberTruncateToInteger(NumberTruncateToInteger {
			destination,
			source,
			is_signed,
			is_saturating,
			to,
			from,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_number_transmute_to_integer(
		&mut self,
		destination: u16,
		source: u16,
		from: number::Type,
	) {
		let instruction = Instruction::NumberTransmuteToInteger(NumberTransmuteToInteger {
			destination,
			source,
			from,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_global_get(&mut self, destination: u16, source: u16) {
		let instruction = Instruction::GlobalGet(GlobalGet {
			destination,
			source,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_global_set(&mut self, destination: u16, source: u16) {
		let instruction = Instruction::GlobalSet(GlobalSet {
			destination,
			source,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_table_get(&mut self, destination: u16, source: Location) {
		let instruction = Instruction::TableGet(TableGet {
			destination,
			source,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_table_set(&mut self, destination: Location, source: u16) {
		let instruction = Instruction::TableSet(TableSet {
			destination,
			source,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_table_size(&mut self, destination: u16, table: u16) {
		let instruction = Instruction::TableSize(TableSize { destination, table });

		self.graph.instructions.push(instruction);
	}

	pub fn add_table_grow(&mut self, destination: u16, table: u16, size: u16, initializer: u16) {
		let instruction = Instruction::TableGrow(TableGrow {
			destination,
			table,
			size,
			initializer,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_table_fill(&mut self, destination: Location, source: u16, size: u16) {
		let instruction = Instruction::TableFill(TableFill {
			destination,
			source,
			size,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_table_copy(&mut self, destination: Location, source: Location, size: u16) {
		let instruction = Instruction::TableCopy(TableCopy {
			destination,
			source,
			size,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_table_init(&mut self, destination: Location, source: Location, size: u16) {
		let instruction = Instruction::TableInit(TableInit {
			destination,
			source,
			size,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_elements_drop(&mut self, source: u16) {
		let instruction = Instruction::ElementsDrop(ElementsDrop { source });

		self.graph.instructions.push(instruction);
	}

	pub fn apply_memory_offset(&mut self, destination: u16, offset: u64) {
		if offset == 0 {
			return;
		}

		let raw_offset = u32::try_from(offset).unwrap();
		let signed_offset = i32::from_ne_bytes(raw_offset.to_ne_bytes());

		self.add_i32_constant(DECODER_SCRATCH, signed_offset);
		self.add_integer_binary_operation(
			destination,
			destination,
			DECODER_SCRATCH,
			integer::Type::I32,
			integer::BinaryOperator::Add,
		);
	}

	pub fn add_memory_load(&mut self, destination: u16, source: Location, kind: LoadType) {
		let instruction = Instruction::MemoryLoad(MemoryLoad {
			destination,
			source,
			kind,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_memory_store(&mut self, destination: Location, source: u16, kind: StoreType) {
		let instruction = Instruction::MemoryStore(MemoryStore {
			destination,
			source,
			kind,
		});

		self.graph.instructions.push(instruction);
	}

	fn convert_pages_to_bytes(&mut self, destination: u16, source: u16) {
		self.add_i32_constant(DECODER_SCRATCH, MemorySize::PAGE_SIZE.try_into().unwrap());
		self.add_integer_binary_operation(
			destination,
			source,
			DECODER_SCRATCH,
			integer::Type::I32,
			integer::BinaryOperator::Multiply,
		);
	}

	fn convert_bytes_to_pages(&mut self, destination: u16, source: u16) {
		self.add_i32_constant(DECODER_SCRATCH, MemorySize::PAGE_SIZE.try_into().unwrap());
		self.add_integer_binary_operation(
			destination,
			source,
			DECODER_SCRATCH,
			integer::Type::I32,
			integer::BinaryOperator::Divide { is_signed: false },
		);
	}

	fn add_memory_size(&mut self, destination: u16, memory: u16) {
		let instruction = Instruction::MemorySize(MemorySize {
			destination,
			memory,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_paged_memory_size(&mut self, destination: u16, memory: u16) {
		self.add_memory_size(destination, memory);
		self.convert_bytes_to_pages(destination, destination);
	}

	fn add_memory_grow(&mut self, destination: u16, memory: u16, size: u16) {
		let instruction = Instruction::MemoryGrow(MemoryGrow {
			destination,
			memory,
			size,
		});

		self.graph.instructions.push(instruction);
	}

	fn add_sized_memory_grow(&mut self, destination: u16, memory: u16, size: u16) {
		self.convert_pages_to_bytes(DECODER_SCRATCH, size);

		self.add_memory_grow(destination, memory, DECODER_SCRATCH);
		self.add_i32_compare_constant(
			DECODER_SCRATCH,
			destination,
			-1,
			integer::CompareOperator::Equal,
		);

		self.add_if(
			DECODER_SCRATCH,
			|this| {
				this.convert_bytes_to_pages(destination, destination);
			},
			|_| {},
		);
	}

	pub fn add_paged_memory_grow(&mut self, destination: u16, memory: u16, size: u16) {
		let page_limit = MemorySize::PAGE_LIMIT.try_into().unwrap();

		self.add_i32_compare_constant(
			DECODER_SCRATCH,
			size,
			page_limit,
			integer::CompareOperator::LessThanEqual { is_signed: false },
		);

		self.add_if(
			DECODER_SCRATCH,
			|this| this.add_i32_constant(destination, -1),
			|this| this.add_sized_memory_grow(destination, memory, size),
		);
	}

	pub fn add_memory_fill(&mut self, destination: Location, byte: u16, size: u16) {
		let instruction = Instruction::MemoryFill(MemoryFill {
			destination,
			byte,
			size,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_memory_copy(&mut self, destination: Location, source: Location, size: u16) {
		let instruction = Instruction::MemoryCopy(MemoryCopy {
			destination,
			source,
			size,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_memory_init(&mut self, destination: Location, source: Location, size: u16) {
		let instruction = Instruction::MemoryInit(MemoryInit {
			destination,
			source,
			size,
		});

		self.graph.instructions.push(instruction);
	}

	pub fn add_data_drop(&mut self, source: u16) {
		let instruction = Instruction::DataDrop(DataDrop { source });

		self.graph.instructions.push(instruction);
	}

	pub fn set_jump_destination(
		&mut self,
		source_block: u16,
		successor_index: u16,
		destination_block: u16,
	) {
		let source_index = usize::from(source_block);
		let successor_index = usize::from(successor_index);

		self.graph.basic_blocks[source_index].successors[successor_index] = destination_block;
	}

	fn set_jump_destinations(&mut self, destination_block: u16, pending_jumps: &[PendingJump]) {
		for &PendingJump {
			source_block,
			successor_index,
			..
		} in pending_jumps
		{
			self.set_jump_destination(source_block, successor_index, destination_block);
		}
	}

	fn add_jump_adjustments(
		&mut self,
		base: u16,
		parameters: u16,
		pending_jumps: &mut [PendingJump],
	) {
		for PendingJump {
			stack_top,
			source_block,
			successor_index,
		} in pending_jumps
		{
			if !self.try_add_stack_adjustment(base, *stack_top, parameters) {
				continue;
			}

			let new_destination = self.add_basic_block(1);

			self.set_jump_destination(*source_block, *successor_index, new_destination);

			*source_block = new_destination;
			*successor_index = 0;
		}
	}

	pub fn handle_level(&mut self, level: ControlLevel, top: u16) {
		let ControlLevel {
			parameters,
			results,
			base,
			kind,
			mut pending_jumps,
		} = level;

		self.try_add_stack_adjustment(base, top, results);

		let fallthrough_exit = self.add_basic_block(1);

		match kind {
			ControlLevelKind::Forward => {
				self.add_jump_adjustments(base, results, &mut pending_jumps);

				let next_destination = self.graph.basic_blocks.len().try_into().unwrap();

				self.set_jump_destinations(next_destination, &pending_jumps);
			}
			ControlLevelKind::Loop { entry } => {
				self.add_jump_adjustments(base, parameters, &mut pending_jumps);
				self.set_jump_destinations(entry, &pending_jumps);
			}
		}

		let next_block = self.graph.basic_blocks.len().try_into().unwrap();

		self.set_jump_destination(fallthrough_exit, 0, next_block);
	}
}
