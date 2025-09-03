use control_flow_graph::{
	ControlFlowGraph,
	instruction::{
		ExtendType, IntegerBinaryOperator, IntegerCompareOperator, IntegerType,
		IntegerUnaryOperator, LoadType, Location, NumberBinaryOperator, NumberCompareOperator,
		NumberType, NumberUnaryOperator, StoreType,
	},
};
use wasmparser::{BlockType, BrTable, FuncType, Ieee32, Ieee64, MemArg, Operator, OperatorsReader};

use crate::{
	code_builder::CodeBuilder,
	stack_builder::{Jump, LOCAL_BASE, Level, SHARED_LOCAL, StackBuilder},
	types::Types,
};

pub struct BasicBlockBuilder {
	code_builder: CodeBuilder,
	stack_builder: StackBuilder,
}

impl BasicBlockBuilder {
	pub const fn new() -> Self {
		Self {
			code_builder: CodeBuilder::new(),
			stack_builder: StackBuilder::new(),
		}
	}

	fn add_local_branch(&mut self, successors: usize) -> u16 {
		let source = self.stack_builder.pull_local();

		self.code_builder.add_local_branch(source, successors)
	}

	fn handle_unreachable(&mut self) {
		let id = self.code_builder.add_unreachable();

		self.stack_builder.set_top(u16::MAX);
		self.stack_builder.jump_to_level(id, 0, 0);
	}

	fn handle_block(&mut self, types: &Types, block_type: BlockType) {
		self.stack_builder.push_level(types, block_type, None);
	}

	fn handle_loop(&mut self, types: &Types, block_type: BlockType) {
		let id = self.code_builder.add_basic_block(1) + 1;

		self.stack_builder.push_level(types, block_type, Some(id));
	}

	fn handle_if(&mut self, types: &Types, block_type: BlockType) {
		let id = self.add_local_branch(2);

		self.stack_builder.push_level(types, block_type, None);
		self.stack_builder.jump_to_depth(id, 0, 0);
	}

	fn handle_else(&mut self) {
		let Level {
			parameters,
			base,
			jumps,
			..
		} = self.stack_builder.peek_level_mut();

		let top = *base + *parameters;

		let Jump { source, branch, .. } = jumps.swap_remove(0);
		let skip = self.code_builder.add_basic_block(1);

		self.stack_builder.jump_to_depth(skip, 0, 0);

		// Patch the if statement to jump to the end of the else block
		self.code_builder
			.set_jump_destination(source, branch, skip + 1);

		self.stack_builder.set_top(top);
	}

	fn handle_end(&mut self) {
		let top = self.stack_builder.get_top();
		let level = self.stack_builder.pull_level();

		self.code_builder.handle_level(level, top);
	}

	fn handle_br(&mut self, relative_depth: u32) {
		let id = self.code_builder.add_basic_block(1);

		self.stack_builder.jump_to_depth(id, 0, relative_depth);
	}

	fn handle_br_if(&mut self, relative_depth: u32) {
		let id = self.add_local_branch(2);

		self.stack_builder.jump_to_depth(id, 1, relative_depth);
	}

	fn handle_br_table(&mut self, table: &BrTable) {
		let len = table.len().try_into().unwrap();
		let id = self.add_local_branch(len + 1);

		self.stack_builder.jump_to_depth(id, len, table.default());

		for (branch, relative_depth) in table.targets().map(Result::unwrap).enumerate() {
			self.stack_builder.jump_to_depth(id, branch, relative_depth);
		}
	}

	fn handle_return(&mut self) {
		let id = self.code_builder.add_basic_block(1);

		self.stack_builder.jump_to_level(id, 0, 0);
	}

	fn handle_call(&mut self, function: u32, r#type: &FuncType) {
		let function = function.try_into().unwrap();
		let (destinations, sources) = self.stack_builder.load_function_type(r#type);

		self.code_builder.add_ref_function(SHARED_LOCAL, function);
		self.code_builder
			.add_call(destinations, sources, SHARED_LOCAL);
	}

	fn handle_call_indirect(&mut self, table: u32, r#type: &FuncType) {
		let function = Location {
			reference: table.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};
		let (destinations, sources) = self.stack_builder.load_function_type(r#type);

		self.code_builder.add_table_get(SHARED_LOCAL, function);
		self.code_builder
			.add_call(destinations, sources, SHARED_LOCAL);
	}

	const fn handle_drop(&mut self) {
		let _item = self.stack_builder.pull_local();
	}

	fn handle_select(&mut self) {
		let condition = self.stack_builder.pull_local();
		let on_false = self.stack_builder.pull_local();
		let on_true = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		let condition = self.code_builder.add_local_branch(condition, 2);

		self.code_builder.add_local_set(destination, on_false);

		let on_false = self.code_builder.add_basic_block(1);

		self.code_builder.add_local_set(destination, on_true);

		let on_true = self.code_builder.add_basic_block(1);

		self.code_builder
			.set_jump_destination(on_false, 0, on_true + 1);

		self.code_builder
			.set_jump_destination(condition, 0, on_false);

		self.code_builder
			.set_jump_destination(condition, 1, on_true);
	}

	fn handle_local_get(&mut self, local: u32) {
		let destination = self.stack_builder.push_local();
		let source = LOCAL_BASE + u16::try_from(local).unwrap();

		self.code_builder.add_local_set(destination, source);
	}

	fn handle_local_set(&mut self, local: u32) {
		let destination = LOCAL_BASE + u16::try_from(local).unwrap();
		let source = self.stack_builder.pull_local();

		self.code_builder.add_local_set(destination, source);
	}

	fn handle_local_tee(&mut self, local: u32) {
		let destination = LOCAL_BASE + u16::try_from(local).unwrap();
		let source = self.stack_builder.pull_local();

		assert_eq!(
			self.stack_builder.push_local(),
			source,
			"local should remain the same"
		);

		self.code_builder.add_local_set(destination, source);
	}

	fn handle_global_get(&mut self, global: u32) {
		let destination = self.stack_builder.push_local();
		let source = global.try_into().unwrap();

		self.code_builder.add_global_get(destination, source);
	}

	fn handle_global_set(&mut self, global: u32) {
		let destination = global.try_into().unwrap();
		let source = self.stack_builder.pull_local();

		self.code_builder.add_global_set(destination, source);
	}

	fn add_memory_offset(&mut self, destination: u16, offset: u64) {
		if offset == 0 {
			return;
		}

		let offset = u32::try_from(offset).unwrap();
		let offset = i32::from_ne_bytes(offset.to_ne_bytes());

		self.code_builder.add_i32_constant(SHARED_LOCAL, offset);
		self.code_builder.add_integer_binary_operation(
			destination,
			destination,
			SHARED_LOCAL,
			IntegerType::I32,
			IntegerBinaryOperator::Add,
		);
	}

	fn handle_load(&mut self, info: MemArg, r#type: LoadType) {
		let source = Location {
			reference: info.memory.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};

		self.add_memory_offset(source.offset, info.offset);

		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_memory_load(destination, source, r#type);
	}

	fn handle_store(&mut self, info: MemArg, r#type: StoreType) {
		let source = self.stack_builder.pull_local();
		let destination = Location {
			reference: info.memory.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};

		self.add_memory_offset(destination.offset, info.offset);

		self.code_builder
			.add_memory_store(destination, source, r#type);
	}

	fn handle_memory_size(&mut self, memory: u32) {
		let reference = memory.try_into().unwrap();
		let destination = self.stack_builder.push_local();

		self.code_builder.add_memory_size(reference, destination);
	}

	fn handle_memory_grow(&mut self, memory: u32) {
		let reference = memory.try_into().unwrap();
		let size = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_memory_grow(reference, destination, size);
	}

	fn handle_i32_const(&mut self, data: i32) {
		let destination = self.stack_builder.push_local();

		self.code_builder.add_i32_constant(destination, data);
	}

	fn handle_i64_const(&mut self, data: i64) {
		let destination = self.stack_builder.push_local();

		self.code_builder.add_i64_constant(destination, data);
	}

	fn handle_f32_const(&mut self, data: Ieee32) {
		let destination = self.stack_builder.push_local();

		self.code_builder.add_f32_constant(destination, data.into());
	}

	fn handle_f64_const(&mut self, data: Ieee64) {
		let destination = self.stack_builder.push_local();

		self.code_builder.add_f64_constant(destination, data.into());
	}

	fn handle_integer_equals_zero(&mut self, r#type: IntegerType) {
		let lhs = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		match r#type {
			IntegerType::I32 => self.code_builder.add_i32_constant(SHARED_LOCAL, 0),
			IntegerType::I64 => self.code_builder.add_i64_constant(SHARED_LOCAL, 0),
		}

		self.code_builder.add_integer_compare_operation(
			destination,
			lhs,
			SHARED_LOCAL,
			r#type,
			IntegerCompareOperator::Equal,
		);
	}

	fn handle_integer_unary_operation(
		&mut self,
		r#type: IntegerType,
		operator: IntegerUnaryOperator,
	) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_integer_unary_operation(destination, source, r#type, operator);
	}

	fn handle_i32_unary(&mut self, operator: IntegerUnaryOperator) {
		self.handle_integer_unary_operation(IntegerType::I32, operator);
	}

	fn handle_i64_unary(&mut self, operator: IntegerUnaryOperator) {
		self.handle_integer_unary_operation(IntegerType::I64, operator);
	}

	fn handle_integer_compare_operation(
		&mut self,
		r#type: IntegerType,
		operator: IntegerCompareOperator,
	) {
		let rhs = self.stack_builder.pull_local();
		let lhs = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_integer_compare_operation(destination, lhs, rhs, r#type, operator);
	}

	fn handle_i32_compare(&mut self, operator: IntegerCompareOperator) {
		self.handle_integer_compare_operation(IntegerType::I32, operator);
	}

	fn handle_i64_compare(&mut self, operator: IntegerCompareOperator) {
		self.handle_integer_compare_operation(IntegerType::I64, operator);
	}

	fn handle_number_compare_operation(
		&mut self,
		r#type: NumberType,
		operator: NumberCompareOperator,
	) {
		let rhs = self.stack_builder.pull_local();
		let lhs = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_number_compare_operation(destination, lhs, rhs, r#type, operator);
	}

	fn handle_f32_compare(&mut self, operator: NumberCompareOperator) {
		self.handle_number_compare_operation(NumberType::F32, operator);
	}

	fn handle_f64_compare(&mut self, operator: NumberCompareOperator) {
		self.handle_number_compare_operation(NumberType::F64, operator);
	}

	fn handle_integer_binary_operation(
		&mut self,
		r#type: IntegerType,
		operator: IntegerBinaryOperator,
	) {
		let rhs = self.stack_builder.pull_local();
		let lhs = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_integer_binary_operation(destination, lhs, rhs, r#type, operator);
	}

	fn handle_i32_binary(&mut self, operator: IntegerBinaryOperator) {
		self.handle_integer_binary_operation(IntegerType::I32, operator);
	}

	fn handle_i64_binary(&mut self, operator: IntegerBinaryOperator) {
		self.handle_integer_binary_operation(IntegerType::I64, operator);
	}

	fn handle_number_unary_operation(&mut self, r#type: NumberType, operator: NumberUnaryOperator) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_number_unary_operation(destination, source, r#type, operator);
	}

	fn handle_f32_unary(&mut self, operator: NumberUnaryOperator) {
		self.handle_number_unary_operation(NumberType::F32, operator);
	}

	fn handle_f64_unary(&mut self, operator: NumberUnaryOperator) {
		self.handle_number_unary_operation(NumberType::F64, operator);
	}

	fn handle_number_binary_operation(
		&mut self,
		r#type: NumberType,
		operator: NumberBinaryOperator,
	) {
		let rhs = self.stack_builder.pull_local();
		let lhs = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_number_binary_operation(destination, lhs, rhs, r#type, operator);
	}

	fn handle_f32_binary(&mut self, operator: NumberBinaryOperator) {
		self.handle_number_binary_operation(NumberType::F32, operator);
	}

	fn handle_f64_binary(&mut self, operator: NumberBinaryOperator) {
		self.handle_number_binary_operation(NumberType::F64, operator);
	}

	fn handle_integer_narrow(&mut self) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder.add_integer_narrow(destination, source);
	}

	fn handle_number_truncate(
		&mut self,
		signed: bool,
		saturate: bool,
		to: IntegerType,
		from: NumberType,
	) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder.add_number_truncate_to_integer(
			destination,
			source,
			signed,
			saturate,
			to,
			from,
		);
	}

	fn handle_f32_truncate(&mut self, signed: bool, to: IntegerType) {
		self.handle_number_truncate(signed, false, to, NumberType::F32);
	}

	fn handle_f64_truncate(&mut self, signed: bool, to: IntegerType) {
		self.handle_number_truncate(signed, false, to, NumberType::F64);
	}

	fn handle_integer_widen(&mut self, signed: bool) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder.add_integer_widen(destination, source);

		if signed {
			self.code_builder
				.add_integer_extend(destination, destination, ExtendType::I64_S32);
		}
	}

	fn handle_integer_convert_to_number(
		&mut self,
		signed: bool,
		to: NumberType,
		from: IntegerType,
	) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_integer_convert_to_number(destination, source, signed, to, from);
	}

	fn handle_i32_convert_to_number(&mut self, signed: bool, to: NumberType) {
		self.handle_integer_convert_to_number(signed, to, IntegerType::I32);
	}

	fn handle_i64_convert_to_number(&mut self, signed: bool, to: NumberType) {
		self.handle_integer_convert_to_number(signed, to, IntegerType::I64);
	}

	fn handle_number_narrow(&mut self) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder.add_number_narrow(destination, source);
	}

	fn handle_number_widen(&mut self) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder.add_number_widen(destination, source);
	}

	fn handle_number_reinterpret(&mut self, from: NumberType) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_number_transmute_to_integer(destination, source, from);
	}

	fn handle_integer_reinterpret(&mut self, from: IntegerType) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_integer_transmute_to_number(destination, source, from);
	}

	fn handle_integer_extend(&mut self, r#type: ExtendType) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_integer_extend(destination, source, r#type);
	}

	fn handle_f32_saturate(&mut self, signed: bool, to: IntegerType) {
		self.handle_number_truncate(signed, true, to, NumberType::F32);
	}

	fn handle_f64_saturate(&mut self, signed: bool, to: IntegerType) {
		self.handle_number_truncate(signed, true, to, NumberType::F64);
	}

	fn handle_memory_init(&mut self, memory: u32, data: u32) {
		let size = self.stack_builder.pull_local();
		let source = Location {
			reference: data.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};
		let destination = Location {
			reference: memory.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};

		self.code_builder.add_memory_init(destination, source, size);
	}

	fn handle_data_drop(&mut self, data: u32) {
		let data = data.try_into().unwrap();

		self.code_builder.add_data_drop(data);
	}

	fn handle_memory_copy(&mut self, destination: u32, source: u32) {
		let size = self.stack_builder.pull_local();
		let source: Location = Location {
			reference: source.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};
		let destination = Location {
			reference: destination.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};

		self.code_builder.add_memory_copy(destination, source, size);
	}

	fn handle_memory_fill(&mut self, memory: u32) {
		let size = self.stack_builder.pull_local();
		let byte = self.stack_builder.pull_local();
		let destination = Location {
			reference: memory.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};

		self.code_builder.add_memory_fill(destination, byte, size);
	}

	fn handle_table_init(&mut self, table: u32, elements: u32) {
		let size = self.stack_builder.pull_local();
		let source: Location = Location {
			reference: elements.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};
		let destination = Location {
			reference: table.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};

		self.code_builder.add_table_init(destination, source, size);
	}

	fn handle_elements_drop(&mut self, elements: u32) {
		let elements = elements.try_into().unwrap();

		self.code_builder.add_elements_drop(elements);
	}

	fn handle_table_copy(&mut self, destination: u32, source: u32) {
		let size = self.stack_builder.pull_local();
		let source: Location = Location {
			reference: source.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};
		let destination = Location {
			reference: destination.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};

		self.code_builder.add_table_copy(destination, source, size);
	}

	fn handle_ref_null(&mut self) {
		let destination = self.stack_builder.push_local();

		self.code_builder.add_ref_null(destination);
	}

	fn handle_ref_is_null(&mut self) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder.add_ref_is_null(destination, source);
	}

	fn handle_ref_function(&mut self, function: u32) {
		let destination = self.stack_builder.push_local();
		let function = function.try_into().unwrap();

		self.code_builder.add_ref_function(destination, function);
	}

	fn handle_table_fill(&mut self, table: u32) {
		let size = self.stack_builder.pull_local();
		let source = self.stack_builder.pull_local();
		let destination = Location {
			reference: table.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};

		self.code_builder.add_table_fill(destination, source, size);
	}

	fn handle_table_get(&mut self, table: u32) {
		let source = Location {
			reference: table.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};
		let destination = self.stack_builder.push_local();

		self.code_builder.add_table_get(destination, source);
	}

	fn handle_table_set(&mut self, table: u32) {
		let source = self.stack_builder.pull_local();
		let destination = Location {
			reference: table.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};

		self.code_builder.add_table_set(destination, source);
	}

	fn handle_table_grow(&mut self, table: u32) {
		let reference = table.try_into().unwrap();
		let size = self.stack_builder.pull_local();
		let initializer = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_table_grow(reference, destination, size, initializer);
	}

	fn handle_table_size(&mut self, table: u32) {
		let reference = table.try_into().unwrap();
		let destination = self.stack_builder.push_local();

		self.code_builder.add_table_size(reference, destination);
	}

	#[expect(clippy::too_many_lines)]
	fn handle_operator(&mut self, types: &Types, operator: Operator) {
		match operator {
			Operator::Unreachable => self.handle_unreachable(),
			Operator::Nop => {}
			Operator::Block { blockty } => self.handle_block(types, blockty),
			Operator::Loop { blockty } => self.handle_loop(types, blockty),
			Operator::If { blockty } => self.handle_if(types, blockty),
			Operator::Else => self.handle_else(),
			Operator::End => self.handle_end(),
			Operator::Br { relative_depth } => self.handle_br(relative_depth),
			Operator::BrIf { relative_depth } => self.handle_br_if(relative_depth),
			Operator::BrTable { ref targets } => self.handle_br_table(targets),
			Operator::Return => self.handle_return(),
			Operator::Call { function_index } => {
				self.handle_call(function_index, types.get_function_type(function_index));
			}
			Operator::CallIndirect {
				type_index,
				table_index,
			} => self.handle_call_indirect(table_index, types.get_type(type_index).unwrap_func()),
			Operator::Drop => self.handle_drop(),
			Operator::Select | Operator::TypedSelect { .. } => self.handle_select(),
			Operator::LocalGet { local_index } => self.handle_local_get(local_index),
			Operator::LocalSet { local_index } => self.handle_local_set(local_index),
			Operator::LocalTee { local_index } => self.handle_local_tee(local_index),
			Operator::GlobalGet { global_index } => self.handle_global_get(global_index),
			Operator::GlobalSet { global_index } => self.handle_global_set(global_index),
			Operator::I32Load { memarg } => self.handle_load(memarg, LoadType::I32),
			Operator::I64Load { memarg } => self.handle_load(memarg, LoadType::I64),
			Operator::F32Load { memarg } => self.handle_load(memarg, LoadType::F32),
			Operator::F64Load { memarg } => self.handle_load(memarg, LoadType::F64),
			Operator::I32Load8S { memarg } => self.handle_load(memarg, LoadType::I32_S8),
			Operator::I32Load8U { memarg } => self.handle_load(memarg, LoadType::I32_U8),
			Operator::I32Load16S { memarg } => self.handle_load(memarg, LoadType::I32_S16),
			Operator::I32Load16U { memarg } => self.handle_load(memarg, LoadType::I32_U16),
			Operator::I64Load8S { memarg } => self.handle_load(memarg, LoadType::I64_S8),
			Operator::I64Load8U { memarg } => self.handle_load(memarg, LoadType::I64_U8),
			Operator::I64Load16S { memarg } => self.handle_load(memarg, LoadType::I64_S16),
			Operator::I64Load16U { memarg } => self.handle_load(memarg, LoadType::I64_U16),
			Operator::I64Load32S { memarg } => self.handle_load(memarg, LoadType::I64_S32),
			Operator::I64Load32U { memarg } => self.handle_load(memarg, LoadType::I64_U32),
			Operator::I32Store { memarg } => self.handle_store(memarg, StoreType::I32),
			Operator::I64Store { memarg } => self.handle_store(memarg, StoreType::I64),
			Operator::F32Store { memarg } => self.handle_store(memarg, StoreType::F32),
			Operator::F64Store { memarg } => self.handle_store(memarg, StoreType::F64),
			Operator::I32Store8 { memarg } => self.handle_store(memarg, StoreType::I32_I8),
			Operator::I32Store16 { memarg } => self.handle_store(memarg, StoreType::I32_I16),
			Operator::I64Store8 { memarg } => self.handle_store(memarg, StoreType::I64_I8),
			Operator::I64Store16 { memarg } => self.handle_store(memarg, StoreType::I64_I16),
			Operator::I64Store32 { memarg } => self.handle_store(memarg, StoreType::I64_I32),
			Operator::MemorySize { mem } => self.handle_memory_size(mem),
			Operator::MemoryGrow { mem } => self.handle_memory_grow(mem),
			Operator::I32Const { value } => self.handle_i32_const(value),
			Operator::I64Const { value } => self.handle_i64_const(value),
			Operator::F32Const { value } => self.handle_f32_const(value),
			Operator::F64Const { value } => self.handle_f64_const(value),
			Operator::I32Eqz => self.handle_integer_equals_zero(IntegerType::I32),
			Operator::I32Eq => self.handle_i32_compare(IntegerCompareOperator::Equal),
			Operator::I32Ne => self.handle_i32_compare(IntegerCompareOperator::NotEqual),
			Operator::I32LtS => {
				self.handle_i32_compare(IntegerCompareOperator::LessThan { signed: true });
			}
			Operator::I32LtU => {
				self.handle_i32_compare(IntegerCompareOperator::LessThan { signed: false });
			}
			Operator::I32GtS => {
				self.handle_i32_compare(IntegerCompareOperator::GreaterThan { signed: true });
			}
			Operator::I32GtU => {
				self.handle_i32_compare(IntegerCompareOperator::GreaterThan { signed: false });
			}
			Operator::I32LeS => {
				self.handle_i32_compare(IntegerCompareOperator::LessThanEqual { signed: true });
			}
			Operator::I32LeU => {
				self.handle_i32_compare(IntegerCompareOperator::LessThanEqual { signed: false });
			}
			Operator::I32GeS => {
				self.handle_i32_compare(IntegerCompareOperator::GreaterThanEqual { signed: true });
			}
			Operator::I32GeU => {
				self.handle_i32_compare(IntegerCompareOperator::GreaterThanEqual { signed: false });
			}
			Operator::I64Eqz => self.handle_integer_equals_zero(IntegerType::I64),
			Operator::I64Eq => self.handle_i64_compare(IntegerCompareOperator::Equal),
			Operator::I64Ne => self.handle_i64_compare(IntegerCompareOperator::NotEqual),
			Operator::I64LtS => {
				self.handle_i64_compare(IntegerCompareOperator::LessThan { signed: true });
			}
			Operator::I64LtU => {
				self.handle_i64_compare(IntegerCompareOperator::LessThan { signed: false });
			}
			Operator::I64GtS => {
				self.handle_i64_compare(IntegerCompareOperator::GreaterThan { signed: true });
			}
			Operator::I64GtU => {
				self.handle_i64_compare(IntegerCompareOperator::GreaterThan { signed: false });
			}
			Operator::I64LeS => {
				self.handle_i64_compare(IntegerCompareOperator::LessThanEqual { signed: true });
			}
			Operator::I64LeU => {
				self.handle_i64_compare(IntegerCompareOperator::LessThanEqual { signed: false });
			}
			Operator::I64GeS => {
				self.handle_i64_compare(IntegerCompareOperator::GreaterThanEqual { signed: true });
			}
			Operator::I64GeU => {
				self.handle_i64_compare(IntegerCompareOperator::GreaterThanEqual { signed: false });
			}
			Operator::F32Eq => self.handle_f32_compare(NumberCompareOperator::Equal),
			Operator::F32Ne => self.handle_f32_compare(NumberCompareOperator::NotEqual),
			Operator::F32Lt => self.handle_f32_compare(NumberCompareOperator::LessThan),
			Operator::F32Gt => self.handle_f32_compare(NumberCompareOperator::GreaterThan),
			Operator::F32Le => self.handle_f32_compare(NumberCompareOperator::LessThanEqual),
			Operator::F32Ge => self.handle_f32_compare(NumberCompareOperator::GreaterThanEqual),
			Operator::F64Eq => self.handle_f64_compare(NumberCompareOperator::Equal),
			Operator::F64Ne => self.handle_f64_compare(NumberCompareOperator::NotEqual),
			Operator::F64Lt => self.handle_f64_compare(NumberCompareOperator::LessThan),
			Operator::F64Gt => self.handle_f64_compare(NumberCompareOperator::GreaterThan),
			Operator::F64Le => self.handle_f64_compare(NumberCompareOperator::LessThanEqual),
			Operator::F64Ge => self.handle_f64_compare(NumberCompareOperator::GreaterThanEqual),
			Operator::I32Clz => self.handle_i32_unary(IntegerUnaryOperator::LeadingZeroes),
			Operator::I32Ctz => self.handle_i32_unary(IntegerUnaryOperator::TrailingZeroes),
			Operator::I32Popcnt => self.handle_i32_unary(IntegerUnaryOperator::CountOnes),
			Operator::I32Add => self.handle_i32_binary(IntegerBinaryOperator::Add),
			Operator::I32Sub => self.handle_i32_binary(IntegerBinaryOperator::Subtract),
			Operator::I32Mul => self.handle_i32_binary(IntegerBinaryOperator::Multiply),
			Operator::I32DivS => {
				self.handle_i32_binary(IntegerBinaryOperator::Divide { signed: true });
			}
			Operator::I32DivU => {
				self.handle_i32_binary(IntegerBinaryOperator::Divide { signed: false });
			}
			Operator::I32RemS => {
				self.handle_i32_binary(IntegerBinaryOperator::Remainder { signed: true });
			}
			Operator::I32RemU => {
				self.handle_i32_binary(IntegerBinaryOperator::Remainder { signed: false });
			}
			Operator::I32And => self.handle_i32_binary(IntegerBinaryOperator::And),
			Operator::I32Or => self.handle_i32_binary(IntegerBinaryOperator::Or),
			Operator::I32Xor => self.handle_i32_binary(IntegerBinaryOperator::ExclusiveOr),
			Operator::I32Shl => self.handle_i32_binary(IntegerBinaryOperator::ShiftLeft),
			Operator::I32ShrS => {
				self.handle_i32_binary(IntegerBinaryOperator::ShiftRight { signed: true });
			}
			Operator::I32ShrU => {
				self.handle_i32_binary(IntegerBinaryOperator::ShiftRight { signed: false });
			}
			Operator::I32Rotl => self.handle_i32_binary(IntegerBinaryOperator::RotateLeft),
			Operator::I32Rotr => self.handle_i32_binary(IntegerBinaryOperator::RotateRight),
			Operator::I64Clz => self.handle_i64_unary(IntegerUnaryOperator::LeadingZeroes),
			Operator::I64Ctz => self.handle_i64_unary(IntegerUnaryOperator::TrailingZeroes),
			Operator::I64Popcnt => self.handle_i64_unary(IntegerUnaryOperator::CountOnes),
			Operator::I64Add => self.handle_i64_binary(IntegerBinaryOperator::Add),
			Operator::I64Sub => self.handle_i64_binary(IntegerBinaryOperator::Subtract),
			Operator::I64Mul => self.handle_i64_binary(IntegerBinaryOperator::Multiply),
			Operator::I64DivS => {
				self.handle_i64_binary(IntegerBinaryOperator::Divide { signed: true });
			}
			Operator::I64DivU => {
				self.handle_i64_binary(IntegerBinaryOperator::Divide { signed: false });
			}
			Operator::I64RemS => {
				self.handle_i64_binary(IntegerBinaryOperator::Remainder { signed: true });
			}
			Operator::I64RemU => {
				self.handle_i64_binary(IntegerBinaryOperator::Remainder { signed: false });
			}
			Operator::I64And => self.handle_i64_binary(IntegerBinaryOperator::And),
			Operator::I64Or => self.handle_i64_binary(IntegerBinaryOperator::Or),
			Operator::I64Xor => self.handle_i64_binary(IntegerBinaryOperator::ExclusiveOr),
			Operator::I64Shl => self.handle_i64_binary(IntegerBinaryOperator::ShiftLeft),
			Operator::I64ShrS => {
				self.handle_i64_binary(IntegerBinaryOperator::ShiftRight { signed: true });
			}
			Operator::I64ShrU => {
				self.handle_i64_binary(IntegerBinaryOperator::ShiftRight { signed: false });
			}
			Operator::I64Rotl => self.handle_i64_binary(IntegerBinaryOperator::RotateLeft),
			Operator::I64Rotr => self.handle_i64_binary(IntegerBinaryOperator::RotateRight),
			Operator::F32Abs => self.handle_f32_unary(NumberUnaryOperator::Absolute),
			Operator::F32Neg => self.handle_f32_unary(NumberUnaryOperator::Negate),
			Operator::F32Ceil => self.handle_f32_unary(NumberUnaryOperator::RoundUp),
			Operator::F32Floor => self.handle_f32_unary(NumberUnaryOperator::RoundDown),
			Operator::F32Trunc => self.handle_f32_unary(NumberUnaryOperator::Truncate),
			Operator::F32Nearest => self.handle_f32_unary(NumberUnaryOperator::Nearest),
			Operator::F32Sqrt => self.handle_f32_unary(NumberUnaryOperator::SquareRoot),
			Operator::F32Add => self.handle_f32_binary(NumberBinaryOperator::Add),
			Operator::F32Sub => self.handle_f32_binary(NumberBinaryOperator::Subtract),
			Operator::F32Mul => self.handle_f32_binary(NumberBinaryOperator::Multiply),
			Operator::F32Div => self.handle_f32_binary(NumberBinaryOperator::Divide),
			Operator::F32Min => self.handle_f32_binary(NumberBinaryOperator::Minimum),
			Operator::F32Max => self.handle_f32_binary(NumberBinaryOperator::Maximum),
			Operator::F32Copysign => self.handle_f32_binary(NumberBinaryOperator::CopySign),
			Operator::F64Abs => self.handle_f64_unary(NumberUnaryOperator::Absolute),
			Operator::F64Neg => self.handle_f64_unary(NumberUnaryOperator::Negate),
			Operator::F64Ceil => self.handle_f64_unary(NumberUnaryOperator::RoundUp),
			Operator::F64Floor => self.handle_f64_unary(NumberUnaryOperator::RoundDown),
			Operator::F64Trunc => self.handle_f64_unary(NumberUnaryOperator::Truncate),
			Operator::F64Nearest => self.handle_f64_unary(NumberUnaryOperator::Nearest),
			Operator::F64Sqrt => self.handle_f64_unary(NumberUnaryOperator::SquareRoot),
			Operator::F64Add => self.handle_f64_binary(NumberBinaryOperator::Add),
			Operator::F64Sub => self.handle_f64_binary(NumberBinaryOperator::Subtract),
			Operator::F64Mul => self.handle_f64_binary(NumberBinaryOperator::Multiply),
			Operator::F64Div => self.handle_f64_binary(NumberBinaryOperator::Divide),
			Operator::F64Min => self.handle_f64_binary(NumberBinaryOperator::Minimum),
			Operator::F64Max => self.handle_f64_binary(NumberBinaryOperator::Maximum),
			Operator::F64Copysign => self.handle_f64_binary(NumberBinaryOperator::CopySign),
			Operator::I32WrapI64 => self.handle_integer_narrow(),
			Operator::I32TruncF32S => self.handle_f32_truncate(true, IntegerType::I32),
			Operator::I32TruncF32U => self.handle_f32_truncate(false, IntegerType::I32),
			Operator::I32TruncF64S => self.handle_f64_truncate(true, IntegerType::I32),
			Operator::I32TruncF64U => self.handle_f64_truncate(false, IntegerType::I32),
			Operator::I64ExtendI32S => self.handle_integer_widen(true),
			Operator::I64ExtendI32U => self.handle_integer_widen(false),
			Operator::I64TruncF32S => self.handle_f32_truncate(true, IntegerType::I64),
			Operator::I64TruncF32U => self.handle_f32_truncate(false, IntegerType::I64),
			Operator::I64TruncF64S => self.handle_f64_truncate(true, IntegerType::I64),
			Operator::I64TruncF64U => self.handle_f64_truncate(false, IntegerType::I64),
			Operator::F32ConvertI32S => self.handle_i32_convert_to_number(true, NumberType::F32),
			Operator::F32ConvertI32U => self.handle_i32_convert_to_number(false, NumberType::F32),
			Operator::F32ConvertI64S => self.handle_i64_convert_to_number(true, NumberType::F32),
			Operator::F32ConvertI64U => self.handle_i64_convert_to_number(false, NumberType::F32),
			Operator::F32DemoteF64 => self.handle_number_narrow(),
			Operator::F64ConvertI32S => self.handle_i32_convert_to_number(true, NumberType::F64),
			Operator::F64ConvertI32U => self.handle_i32_convert_to_number(false, NumberType::F64),
			Operator::F64ConvertI64S => self.handle_i64_convert_to_number(true, NumberType::F64),
			Operator::F64ConvertI64U => self.handle_i64_convert_to_number(false, NumberType::F64),
			Operator::F64PromoteF32 => self.handle_number_widen(),
			Operator::I32ReinterpretF32 => self.handle_number_reinterpret(NumberType::F32),
			Operator::I64ReinterpretF64 => self.handle_number_reinterpret(NumberType::F64),
			Operator::F32ReinterpretI32 => self.handle_integer_reinterpret(IntegerType::I32),
			Operator::F64ReinterpretI64 => self.handle_integer_reinterpret(IntegerType::I64),
			Operator::I32Extend8S => self.handle_integer_extend(ExtendType::I32_S8),
			Operator::I32Extend16S => self.handle_integer_extend(ExtendType::I32_S16),
			Operator::I64Extend8S => self.handle_integer_extend(ExtendType::I64_S8),
			Operator::I64Extend16S => self.handle_integer_extend(ExtendType::I64_S16),
			Operator::I64Extend32S => self.handle_integer_extend(ExtendType::I64_S32),
			Operator::I32TruncSatF32S => self.handle_f32_saturate(true, IntegerType::I32),
			Operator::I32TruncSatF32U => self.handle_f32_saturate(false, IntegerType::I32),
			Operator::I32TruncSatF64S => self.handle_f64_saturate(true, IntegerType::I32),
			Operator::I32TruncSatF64U => self.handle_f64_saturate(false, IntegerType::I32),
			Operator::I64TruncSatF32S => self.handle_f32_saturate(true, IntegerType::I64),
			Operator::I64TruncSatF32U => self.handle_f32_saturate(false, IntegerType::I64),
			Operator::I64TruncSatF64S => self.handle_f64_saturate(true, IntegerType::I64),
			Operator::I64TruncSatF64U => self.handle_f64_saturate(false, IntegerType::I64),
			Operator::MemoryInit { data_index, mem } => self.handle_memory_init(mem, data_index),
			Operator::DataDrop { data_index } => self.handle_data_drop(data_index),
			Operator::MemoryCopy { dst_mem, src_mem } => self.handle_memory_copy(dst_mem, src_mem),
			Operator::MemoryFill { mem } => self.handle_memory_fill(mem),
			Operator::TableInit { elem_index, table } => self.handle_table_init(table, elem_index),
			Operator::ElemDrop { elem_index } => self.handle_elements_drop(elem_index),
			Operator::TableCopy {
				dst_table,
				src_table,
			} => self.handle_table_copy(dst_table, src_table),
			Operator::RefNull { .. } => self.handle_ref_null(),
			Operator::RefIsNull => self.handle_ref_is_null(),
			Operator::RefFunc { function_index } => self.handle_ref_function(function_index),
			Operator::TableFill { table } => self.handle_table_fill(table),
			Operator::TableGet { table } => self.handle_table_get(table),
			Operator::TableSet { table } => self.handle_table_set(table),
			Operator::TableGrow { table } => self.handle_table_grow(table),
			Operator::TableSize { table } => self.handle_table_size(table),

			operator => unimplemented!("{operator:?}"),
		}
	}

	pub fn run(
		&mut self,
		graph: &mut ControlFlowGraph,
		types: &Types,
		function_type: BlockType,
		locals: u16,
		operators: OperatorsReader,
	) {
		self.stack_builder
			.set_function_data(types, function_type, locals);

		self.code_builder.clear();
		self.code_builder.add_basic_block(1);

		for operator in operators.into_iter().map(Result::unwrap) {
			self.handle_operator(types, operator);
		}

		self.code_builder.add_basic_block(0);
		self.code_builder.swap_contents(graph);
	}
}
