//! Expression builder that converts WebAssembly operators into IR instructions.

use wasmparser::{BlockType, BrTable, FuncType, Ieee32, Ieee64, MemArg, Operator, OperatorsReader};

use web_assembly_graph::{
	ControlFlowGraph,
	instruction::{ExtendType, LoadType, Location, StoreType, integer, number},
};

use super::{
	code_builder::CodeBuilder,
	stack_builder::{Jump, LOCAL_BASE, Level, SHARED_LOCAL, StackBuilder},
	types::Types,
};

pub struct ExpressionBuilder {
	code_builder: CodeBuilder,
	stack_builder: StackBuilder,
}

impl ExpressionBuilder {
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

	fn handle_br_table(&mut self, table: &BrTable<'_>) {
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

	fn handle_call(&mut self, function: u32, kind: &FuncType) {
		let function = function.try_into().unwrap();
		let (destinations, sources) = self.stack_builder.load_function_type(kind);

		self.code_builder.add_ref_function(SHARED_LOCAL, function);
		self.code_builder
			.add_call(destinations, sources, SHARED_LOCAL);
	}

	fn handle_call_indirect(&mut self, table: u32, kind: &FuncType) {
		let function = Location {
			reference: table.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};
		let (destinations, sources) = self.stack_builder.load_function_type(kind);

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

		self.code_builder
			.add_select(destination, condition, on_false, on_true);
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

	fn handle_load(&mut self, memory_argument: MemArg, kind: LoadType) {
		let source = Location {
			reference: memory_argument.memory.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};

		self.code_builder
			.apply_memory_offset(source.offset, memory_argument.offset);

		let destination = self.stack_builder.push_local();

		self.code_builder.add_memory_load(destination, source, kind);
	}

	fn handle_store(&mut self, memory_argument: MemArg, kind: StoreType) {
		let source = self.stack_builder.pull_local();
		let destination = Location {
			reference: memory_argument.memory.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};

		self.code_builder
			.apply_memory_offset(destination.offset, memory_argument.offset);

		self.code_builder
			.add_memory_store(destination, source, kind);
	}

	fn handle_memory_size(&mut self, memory: u32) {
		let memory = memory.try_into().unwrap();
		let destination = self.stack_builder.push_local();

		self.code_builder.add_paged_memory_size(destination, memory);
	}

	fn handle_memory_grow(&mut self, memory: u32) {
		let memory = memory.try_into().unwrap();
		let size = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_paged_memory_grow(destination, memory, size);
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

	fn handle_i32_equals_zero(&mut self) {
		let lhs = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder.add_i32_compare_constant(
			destination,
			lhs,
			0,
			integer::CompareOperator::Equal,
		);
	}

	fn handle_i64_equals_zero(&mut self) {
		let lhs = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder.add_i64_constant(SHARED_LOCAL, 0);
		self.code_builder.add_integer_compare_operation(
			destination,
			lhs,
			SHARED_LOCAL,
			integer::Type::I64,
			integer::CompareOperator::Equal,
		);
	}

	fn handle_integer_unary_operation(
		&mut self,
		kind: integer::Type,
		operator: integer::UnaryOperator,
	) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_integer_unary_operation(destination, source, kind, operator);
	}

	fn handle_i32_unary(&mut self, operator: integer::UnaryOperator) {
		self.handle_integer_unary_operation(integer::Type::I32, operator);
	}

	fn handle_i64_unary(&mut self, operator: integer::UnaryOperator) {
		self.handle_integer_unary_operation(integer::Type::I64, operator);
	}

	fn handle_integer_compare_operation(
		&mut self,
		kind: integer::Type,
		operator: integer::CompareOperator,
	) {
		let rhs = self.stack_builder.pull_local();
		let lhs = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_integer_compare_operation(destination, lhs, rhs, kind, operator);
	}

	fn handle_i32_compare(&mut self, operator: integer::CompareOperator) {
		self.handle_integer_compare_operation(integer::Type::I32, operator);
	}

	fn handle_i64_compare(&mut self, operator: integer::CompareOperator) {
		self.handle_integer_compare_operation(integer::Type::I64, operator);
	}

	fn handle_number_compare_operation(
		&mut self,
		kind: number::Type,
		operator: number::CompareOperator,
	) {
		let rhs = self.stack_builder.pull_local();
		let lhs = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_number_compare_operation(destination, lhs, rhs, kind, operator);
	}

	fn handle_f32_compare(&mut self, operator: number::CompareOperator) {
		self.handle_number_compare_operation(number::Type::F32, operator);
	}

	fn handle_f64_compare(&mut self, operator: number::CompareOperator) {
		self.handle_number_compare_operation(number::Type::F64, operator);
	}

	fn handle_integer_binary_operation(
		&mut self,
		kind: integer::Type,
		operator: integer::BinaryOperator,
	) {
		let rhs = self.stack_builder.pull_local();
		let lhs = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_integer_binary_operation(destination, lhs, rhs, kind, operator);
	}

	fn handle_i32_binary(&mut self, operator: integer::BinaryOperator) {
		self.handle_integer_binary_operation(integer::Type::I32, operator);
	}

	fn handle_i64_binary(&mut self, operator: integer::BinaryOperator) {
		self.handle_integer_binary_operation(integer::Type::I64, operator);
	}

	fn handle_number_unary_operation(
		&mut self,
		kind: number::Type,
		operator: number::UnaryOperator,
	) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_number_unary_operation(destination, source, kind, operator);
	}

	fn handle_f32_unary(&mut self, operator: number::UnaryOperator) {
		self.handle_number_unary_operation(number::Type::F32, operator);
	}

	fn handle_f64_unary(&mut self, operator: number::UnaryOperator) {
		self.handle_number_unary_operation(number::Type::F64, operator);
	}

	fn handle_number_binary_operation(
		&mut self,
		kind: number::Type,
		operator: number::BinaryOperator,
	) {
		let rhs = self.stack_builder.pull_local();
		let lhs = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_number_binary_operation(destination, lhs, rhs, kind, operator);
	}

	fn handle_f32_binary(&mut self, operator: number::BinaryOperator) {
		self.handle_number_binary_operation(number::Type::F32, operator);
	}

	fn handle_f64_binary(&mut self, operator: number::BinaryOperator) {
		self.handle_number_binary_operation(number::Type::F64, operator);
	}

	fn handle_integer_narrow(&mut self) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder.add_integer_narrow(destination, source);
	}

	fn handle_number_truncate(
		&mut self,
		is_signed: bool,
		is_saturating: bool,
		to: integer::Type,
		from: number::Type,
	) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder.add_number_truncate_to_integer(
			destination,
			source,
			is_signed,
			is_saturating,
			to,
			from,
		);
	}

	fn handle_f32_truncate(&mut self, is_signed: bool, to: integer::Type) {
		self.handle_number_truncate(is_signed, false, to, number::Type::F32);
	}

	fn handle_f64_truncate(&mut self, is_signed: bool, to: integer::Type) {
		self.handle_number_truncate(is_signed, false, to, number::Type::F64);
	}

	fn handle_integer_widen(&mut self, is_signed: bool) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder.add_integer_widen(destination, source);

		if is_signed {
			self.code_builder
				.add_integer_extend(destination, destination, ExtendType::I64_S32);
		}
	}

	fn handle_integer_convert_to_number(
		&mut self,
		is_signed: bool,
		to: number::Type,
		from: integer::Type,
	) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_integer_convert_to_number(destination, source, is_signed, to, from);
	}

	fn handle_i32_convert_to_number(&mut self, is_signed: bool, to: number::Type) {
		self.handle_integer_convert_to_number(is_signed, to, integer::Type::I32);
	}

	fn handle_i64_convert_to_number(&mut self, is_signed: bool, to: number::Type) {
		self.handle_integer_convert_to_number(is_signed, to, integer::Type::I64);
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

	fn handle_number_reinterpret(&mut self, from: number::Type) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_number_transmute_to_integer(destination, source, from);
	}

	fn handle_integer_reinterpret(&mut self, from: integer::Type) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_integer_transmute_to_number(destination, source, from);
	}

	fn handle_integer_extend(&mut self, kind: ExtendType) {
		let source = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_integer_extend(destination, source, kind);
	}

	fn handle_f32_saturate(&mut self, is_signed: bool, to: integer::Type) {
		self.handle_number_truncate(is_signed, true, to, number::Type::F32);
	}

	fn handle_f64_saturate(&mut self, is_signed: bool, to: integer::Type) {
		self.handle_number_truncate(is_signed, true, to, number::Type::F64);
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

	fn handle_memory_copy(&mut self, destination_memory: u32, source_memory: u32) {
		let size = self.stack_builder.pull_local();
		let source_location: Location = Location {
			reference: source_memory.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};
		let destination_location = Location {
			reference: destination_memory.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};

		self.code_builder
			.add_memory_copy(destination_location, source_location, size);
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

	fn handle_table_copy(&mut self, destination_table: u32, source_table: u32) {
		let size = self.stack_builder.pull_local();
		let source_location: Location = Location {
			reference: source_table.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};
		let destination_location = Location {
			reference: destination_table.try_into().unwrap(),
			offset: self.stack_builder.pull_local(),
		};

		self.code_builder
			.add_table_copy(destination_location, source_location, size);
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
		let table = table.try_into().unwrap();
		let size = self.stack_builder.pull_local();
		let initializer = self.stack_builder.pull_local();
		let destination = self.stack_builder.push_local();

		self.code_builder
			.add_table_grow(destination, table, size, initializer);
	}

	fn handle_table_size(&mut self, table: u32) {
		let table = table.try_into().unwrap();
		let destination = self.stack_builder.push_local();

		self.code_builder.add_table_size(destination, table);
	}

	#[expect(
		clippy::too_many_lines,
		reason = "large match needed for all WebAssembly operators"
	)]
	fn handle_operator(&mut self, types: &Types, operator: &Operator<'_>) {
		match *operator {
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
			Operator::I32Eqz => self.handle_i32_equals_zero(),
			Operator::I32Eq => self.handle_i32_compare(integer::CompareOperator::Equal),
			Operator::I32Ne => self.handle_i32_compare(integer::CompareOperator::NotEqual),
			Operator::I32LtS => {
				self.handle_i32_compare(integer::CompareOperator::LessThan { is_signed: true });
			}
			Operator::I32LtU => {
				self.handle_i32_compare(integer::CompareOperator::LessThan { is_signed: false });
			}
			Operator::I32GtS => {
				self.handle_i32_compare(integer::CompareOperator::GreaterThan { is_signed: true });
			}
			Operator::I32GtU => {
				self.handle_i32_compare(integer::CompareOperator::GreaterThan { is_signed: false });
			}
			Operator::I32LeS => {
				self.handle_i32_compare(integer::CompareOperator::LessThanEqual {
					is_signed: true,
				});
			}
			Operator::I32LeU => {
				self.handle_i32_compare(integer::CompareOperator::LessThanEqual {
					is_signed: false,
				});
			}
			Operator::I32GeS => {
				self.handle_i32_compare(integer::CompareOperator::GreaterThanEqual {
					is_signed: true,
				});
			}
			Operator::I32GeU => {
				self.handle_i32_compare(integer::CompareOperator::GreaterThanEqual {
					is_signed: false,
				});
			}
			Operator::I64Eqz => self.handle_i64_equals_zero(),
			Operator::I64Eq => self.handle_i64_compare(integer::CompareOperator::Equal),
			Operator::I64Ne => self.handle_i64_compare(integer::CompareOperator::NotEqual),
			Operator::I64LtS => {
				self.handle_i64_compare(integer::CompareOperator::LessThan { is_signed: true });
			}
			Operator::I64LtU => {
				self.handle_i64_compare(integer::CompareOperator::LessThan { is_signed: false });
			}
			Operator::I64GtS => {
				self.handle_i64_compare(integer::CompareOperator::GreaterThan { is_signed: true });
			}
			Operator::I64GtU => {
				self.handle_i64_compare(integer::CompareOperator::GreaterThan { is_signed: false });
			}
			Operator::I64LeS => {
				self.handle_i64_compare(integer::CompareOperator::LessThanEqual {
					is_signed: true,
				});
			}
			Operator::I64LeU => {
				self.handle_i64_compare(integer::CompareOperator::LessThanEqual {
					is_signed: false,
				});
			}
			Operator::I64GeS => {
				self.handle_i64_compare(integer::CompareOperator::GreaterThanEqual {
					is_signed: true,
				});
			}
			Operator::I64GeU => {
				self.handle_i64_compare(integer::CompareOperator::GreaterThanEqual {
					is_signed: false,
				});
			}
			Operator::F32Eq => self.handle_f32_compare(number::CompareOperator::Equal),
			Operator::F32Ne => self.handle_f32_compare(number::CompareOperator::NotEqual),
			Operator::F32Lt => self.handle_f32_compare(number::CompareOperator::LessThan),
			Operator::F32Gt => self.handle_f32_compare(number::CompareOperator::GreaterThan),
			Operator::F32Le => self.handle_f32_compare(number::CompareOperator::LessThanEqual),
			Operator::F32Ge => self.handle_f32_compare(number::CompareOperator::GreaterThanEqual),
			Operator::F64Eq => self.handle_f64_compare(number::CompareOperator::Equal),
			Operator::F64Ne => self.handle_f64_compare(number::CompareOperator::NotEqual),
			Operator::F64Lt => self.handle_f64_compare(number::CompareOperator::LessThan),
			Operator::F64Gt => self.handle_f64_compare(number::CompareOperator::GreaterThan),
			Operator::F64Le => self.handle_f64_compare(number::CompareOperator::LessThanEqual),
			Operator::F64Ge => self.handle_f64_compare(number::CompareOperator::GreaterThanEqual),
			Operator::I32Clz => self.handle_i32_unary(integer::UnaryOperator::LeadingZeros),
			Operator::I32Ctz => self.handle_i32_unary(integer::UnaryOperator::TrailingZeros),
			Operator::I32Popcnt => self.handle_i32_unary(integer::UnaryOperator::CountOnes),
			Operator::I32Add => self.handle_i32_binary(integer::BinaryOperator::Add),
			Operator::I32Sub => self.handle_i32_binary(integer::BinaryOperator::Subtract),
			Operator::I32Mul => self.handle_i32_binary(integer::BinaryOperator::Multiply),
			Operator::I32DivS => {
				self.handle_i32_binary(integer::BinaryOperator::Divide { is_signed: true });
			}
			Operator::I32DivU => {
				self.handle_i32_binary(integer::BinaryOperator::Divide { is_signed: false });
			}
			Operator::I32RemS => {
				self.handle_i32_binary(integer::BinaryOperator::Remainder { is_signed: true });
			}
			Operator::I32RemU => {
				self.handle_i32_binary(integer::BinaryOperator::Remainder { is_signed: false });
			}
			Operator::I32And => self.handle_i32_binary(integer::BinaryOperator::And),
			Operator::I32Or => self.handle_i32_binary(integer::BinaryOperator::Or),
			Operator::I32Xor => self.handle_i32_binary(integer::BinaryOperator::ExclusiveOr),
			Operator::I32Shl => self.handle_i32_binary(integer::BinaryOperator::ShiftLeft),
			Operator::I32ShrS => {
				self.handle_i32_binary(integer::BinaryOperator::ShiftRight { is_signed: true });
			}
			Operator::I32ShrU => {
				self.handle_i32_binary(integer::BinaryOperator::ShiftRight { is_signed: false });
			}
			Operator::I32Rotl => self.handle_i32_binary(integer::BinaryOperator::RotateLeft),
			Operator::I32Rotr => self.handle_i32_binary(integer::BinaryOperator::RotateRight),
			Operator::I64Clz => self.handle_i64_unary(integer::UnaryOperator::LeadingZeros),
			Operator::I64Ctz => self.handle_i64_unary(integer::UnaryOperator::TrailingZeros),
			Operator::I64Popcnt => self.handle_i64_unary(integer::UnaryOperator::CountOnes),
			Operator::I64Add => self.handle_i64_binary(integer::BinaryOperator::Add),
			Operator::I64Sub => self.handle_i64_binary(integer::BinaryOperator::Subtract),
			Operator::I64Mul => self.handle_i64_binary(integer::BinaryOperator::Multiply),
			Operator::I64DivS => {
				self.handle_i64_binary(integer::BinaryOperator::Divide { is_signed: true });
			}
			Operator::I64DivU => {
				self.handle_i64_binary(integer::BinaryOperator::Divide { is_signed: false });
			}
			Operator::I64RemS => {
				self.handle_i64_binary(integer::BinaryOperator::Remainder { is_signed: true });
			}
			Operator::I64RemU => {
				self.handle_i64_binary(integer::BinaryOperator::Remainder { is_signed: false });
			}
			Operator::I64And => self.handle_i64_binary(integer::BinaryOperator::And),
			Operator::I64Or => self.handle_i64_binary(integer::BinaryOperator::Or),
			Operator::I64Xor => self.handle_i64_binary(integer::BinaryOperator::ExclusiveOr),
			Operator::I64Shl => self.handle_i64_binary(integer::BinaryOperator::ShiftLeft),
			Operator::I64ShrS => {
				self.handle_i64_binary(integer::BinaryOperator::ShiftRight { is_signed: true });
			}
			Operator::I64ShrU => {
				self.handle_i64_binary(integer::BinaryOperator::ShiftRight { is_signed: false });
			}
			Operator::I64Rotl => self.handle_i64_binary(integer::BinaryOperator::RotateLeft),
			Operator::I64Rotr => self.handle_i64_binary(integer::BinaryOperator::RotateRight),
			Operator::F32Abs => self.handle_f32_unary(number::UnaryOperator::Absolute),
			Operator::F32Neg => self.handle_f32_unary(number::UnaryOperator::Negate),
			Operator::F32Ceil => self.handle_f32_unary(number::UnaryOperator::RoundUp),
			Operator::F32Floor => self.handle_f32_unary(number::UnaryOperator::RoundDown),
			Operator::F32Trunc => self.handle_f32_unary(number::UnaryOperator::Truncate),
			Operator::F32Nearest => self.handle_f32_unary(number::UnaryOperator::Nearest),
			Operator::F32Sqrt => self.handle_f32_unary(number::UnaryOperator::SquareRoot),
			Operator::F32Add => self.handle_f32_binary(number::BinaryOperator::Add),
			Operator::F32Sub => self.handle_f32_binary(number::BinaryOperator::Subtract),
			Operator::F32Mul => self.handle_f32_binary(number::BinaryOperator::Multiply),
			Operator::F32Div => self.handle_f32_binary(number::BinaryOperator::Divide),
			Operator::F32Min => self.handle_f32_binary(number::BinaryOperator::Minimum),
			Operator::F32Max => self.handle_f32_binary(number::BinaryOperator::Maximum),
			Operator::F32Copysign => self.handle_f32_binary(number::BinaryOperator::CopySign),
			Operator::F64Abs => self.handle_f64_unary(number::UnaryOperator::Absolute),
			Operator::F64Neg => self.handle_f64_unary(number::UnaryOperator::Negate),
			Operator::F64Ceil => self.handle_f64_unary(number::UnaryOperator::RoundUp),
			Operator::F64Floor => self.handle_f64_unary(number::UnaryOperator::RoundDown),
			Operator::F64Trunc => self.handle_f64_unary(number::UnaryOperator::Truncate),
			Operator::F64Nearest => self.handle_f64_unary(number::UnaryOperator::Nearest),
			Operator::F64Sqrt => self.handle_f64_unary(number::UnaryOperator::SquareRoot),
			Operator::F64Add => self.handle_f64_binary(number::BinaryOperator::Add),
			Operator::F64Sub => self.handle_f64_binary(number::BinaryOperator::Subtract),
			Operator::F64Mul => self.handle_f64_binary(number::BinaryOperator::Multiply),
			Operator::F64Div => self.handle_f64_binary(number::BinaryOperator::Divide),
			Operator::F64Min => self.handle_f64_binary(number::BinaryOperator::Minimum),
			Operator::F64Max => self.handle_f64_binary(number::BinaryOperator::Maximum),
			Operator::F64Copysign => self.handle_f64_binary(number::BinaryOperator::CopySign),
			Operator::I32WrapI64 => self.handle_integer_narrow(),
			Operator::I32TruncF32S => self.handle_f32_truncate(true, integer::Type::I32),
			Operator::I32TruncF32U => self.handle_f32_truncate(false, integer::Type::I32),
			Operator::I32TruncF64S => self.handle_f64_truncate(true, integer::Type::I32),
			Operator::I32TruncF64U => self.handle_f64_truncate(false, integer::Type::I32),
			Operator::I64ExtendI32S => self.handle_integer_widen(true),
			Operator::I64ExtendI32U => self.handle_integer_widen(false),
			Operator::I64TruncF32S => self.handle_f32_truncate(true, integer::Type::I64),
			Operator::I64TruncF32U => self.handle_f32_truncate(false, integer::Type::I64),
			Operator::I64TruncF64S => self.handle_f64_truncate(true, integer::Type::I64),
			Operator::I64TruncF64U => self.handle_f64_truncate(false, integer::Type::I64),
			Operator::F32ConvertI32S => self.handle_i32_convert_to_number(true, number::Type::F32),
			Operator::F32ConvertI32U => self.handle_i32_convert_to_number(false, number::Type::F32),
			Operator::F32ConvertI64S => self.handle_i64_convert_to_number(true, number::Type::F32),
			Operator::F32ConvertI64U => self.handle_i64_convert_to_number(false, number::Type::F32),
			Operator::F32DemoteF64 => self.handle_number_narrow(),
			Operator::F64ConvertI32S => self.handle_i32_convert_to_number(true, number::Type::F64),
			Operator::F64ConvertI32U => self.handle_i32_convert_to_number(false, number::Type::F64),
			Operator::F64ConvertI64S => self.handle_i64_convert_to_number(true, number::Type::F64),
			Operator::F64ConvertI64U => self.handle_i64_convert_to_number(false, number::Type::F64),
			Operator::F64PromoteF32 => self.handle_number_widen(),
			Operator::I32ReinterpretF32 => self.handle_number_reinterpret(number::Type::F32),
			Operator::I64ReinterpretF64 => self.handle_number_reinterpret(number::Type::F64),
			Operator::F32ReinterpretI32 => self.handle_integer_reinterpret(integer::Type::I32),
			Operator::F64ReinterpretI64 => self.handle_integer_reinterpret(integer::Type::I64),
			Operator::I32Extend8S => self.handle_integer_extend(ExtendType::I32_S8),
			Operator::I32Extend16S => self.handle_integer_extend(ExtendType::I32_S16),
			Operator::I64Extend8S => self.handle_integer_extend(ExtendType::I64_S8),
			Operator::I64Extend16S => self.handle_integer_extend(ExtendType::I64_S16),
			Operator::I64Extend32S => self.handle_integer_extend(ExtendType::I64_S32),
			Operator::I32TruncSatF32S => self.handle_f32_saturate(true, integer::Type::I32),
			Operator::I32TruncSatF32U => self.handle_f32_saturate(false, integer::Type::I32),
			Operator::I32TruncSatF64S => self.handle_f64_saturate(true, integer::Type::I32),
			Operator::I32TruncSatF64U => self.handle_f64_saturate(false, integer::Type::I32),
			Operator::I64TruncSatF32S => self.handle_f32_saturate(true, integer::Type::I64),
			Operator::I64TruncSatF32U => self.handle_f32_saturate(false, integer::Type::I64),
			Operator::I64TruncSatF64S => self.handle_f64_saturate(true, integer::Type::I64),
			Operator::I64TruncSatF64U => self.handle_f64_saturate(false, integer::Type::I64),
			Operator::MemoryInit { data_index, mem } => self.handle_memory_init(mem, data_index),
			Operator::DataDrop { data_index } => self.handle_data_drop(data_index),
			Operator::MemoryCopy {
				dst_mem: destination_memory,
				src_mem: source_memory,
			} => self.handle_memory_copy(destination_memory, source_memory),
			Operator::MemoryFill { mem } => self.handle_memory_fill(mem),
			Operator::TableInit { elem_index, table } => self.handle_table_init(table, elem_index),
			Operator::ElemDrop { elem_index } => self.handle_elements_drop(elem_index),
			Operator::TableCopy {
				dst_table: destination_table,
				src_table: source_table,
			} => self.handle_table_copy(destination_table, source_table),
			Operator::RefNull { .. } => self.handle_ref_null(),
			Operator::RefIsNull => self.handle_ref_is_null(),
			Operator::RefFunc { function_index } => self.handle_ref_function(function_index),
			Operator::TableFill { table } => self.handle_table_fill(table),
			Operator::TableGet { table } => self.handle_table_get(table),
			Operator::TableSet { table } => self.handle_table_set(table),
			Operator::TableGrow { table } => self.handle_table_grow(table),
			Operator::TableSize { table } => self.handle_table_size(table),

			Operator::RefEq
			| Operator::StructNew { .. }
			| Operator::StructNewDefault { .. }
			| Operator::StructGet { .. }
			| Operator::StructGetS { .. }
			| Operator::StructGetU { .. }
			| Operator::StructSet { .. }
			| Operator::ArrayNew { .. }
			| Operator::ArrayNewDefault { .. }
			| Operator::ArrayNewFixed { .. }
			| Operator::ArrayNewData { .. }
			| Operator::ArrayNewElem { .. }
			| Operator::ArrayGet { .. }
			| Operator::ArrayGetS { .. }
			| Operator::ArrayGetU { .. }
			| Operator::ArraySet { .. }
			| Operator::ArrayLen
			| Operator::ArrayFill { .. }
			| Operator::ArrayCopy { .. }
			| Operator::ArrayInitData { .. }
			| Operator::ArrayInitElem { .. }
			| Operator::RefTestNonNull { .. }
			| Operator::RefTestNullable { .. }
			| Operator::RefCastNonNull { .. }
			| Operator::RefCastNullable { .. }
			| Operator::BrOnCast { .. }
			| Operator::BrOnCastFail { .. }
			| Operator::AnyConvertExtern
			| Operator::ExternConvertAny
			| Operator::RefI31
			| Operator::I31GetS
			| Operator::I31GetU
			| Operator::TypedSelectMulti { .. }
			| Operator::ReturnCall { .. }
			| Operator::ReturnCallIndirect { .. }
			| Operator::MemoryDiscard { .. }
			| Operator::MemoryAtomicNotify { .. }
			| Operator::MemoryAtomicWait32 { .. }
			| Operator::MemoryAtomicWait64 { .. }
			| Operator::AtomicFence
			| Operator::I32AtomicLoad { .. }
			| Operator::I64AtomicLoad { .. }
			| Operator::I32AtomicLoad8U { .. }
			| Operator::I32AtomicLoad16U { .. }
			| Operator::I64AtomicLoad8U { .. }
			| Operator::I64AtomicLoad16U { .. }
			| Operator::I64AtomicLoad32U { .. }
			| Operator::I32AtomicStore { .. }
			| Operator::I64AtomicStore { .. }
			| Operator::I32AtomicStore8 { .. }
			| Operator::I32AtomicStore16 { .. }
			| Operator::I64AtomicStore8 { .. }
			| Operator::I64AtomicStore16 { .. }
			| Operator::I64AtomicStore32 { .. }
			| Operator::I32AtomicRmwAdd { .. }
			| Operator::I64AtomicRmwAdd { .. }
			| Operator::I32AtomicRmw8AddU { .. }
			| Operator::I32AtomicRmw16AddU { .. }
			| Operator::I64AtomicRmw8AddU { .. }
			| Operator::I64AtomicRmw16AddU { .. }
			| Operator::I64AtomicRmw32AddU { .. }
			| Operator::I32AtomicRmwSub { .. }
			| Operator::I64AtomicRmwSub { .. }
			| Operator::I32AtomicRmw8SubU { .. }
			| Operator::I32AtomicRmw16SubU { .. }
			| Operator::I64AtomicRmw8SubU { .. }
			| Operator::I64AtomicRmw16SubU { .. }
			| Operator::I64AtomicRmw32SubU { .. }
			| Operator::I32AtomicRmwAnd { .. }
			| Operator::I64AtomicRmwAnd { .. }
			| Operator::I32AtomicRmw8AndU { .. }
			| Operator::I32AtomicRmw16AndU { .. }
			| Operator::I64AtomicRmw8AndU { .. }
			| Operator::I64AtomicRmw16AndU { .. }
			| Operator::I64AtomicRmw32AndU { .. }
			| Operator::I32AtomicRmwOr { .. }
			| Operator::I64AtomicRmwOr { .. }
			| Operator::I32AtomicRmw8OrU { .. }
			| Operator::I32AtomicRmw16OrU { .. }
			| Operator::I64AtomicRmw8OrU { .. }
			| Operator::I64AtomicRmw16OrU { .. }
			| Operator::I64AtomicRmw32OrU { .. }
			| Operator::I32AtomicRmwXor { .. }
			| Operator::I64AtomicRmwXor { .. }
			| Operator::I32AtomicRmw8XorU { .. }
			| Operator::I32AtomicRmw16XorU { .. }
			| Operator::I64AtomicRmw8XorU { .. }
			| Operator::I64AtomicRmw16XorU { .. }
			| Operator::I64AtomicRmw32XorU { .. }
			| Operator::I32AtomicRmwXchg { .. }
			| Operator::I64AtomicRmwXchg { .. }
			| Operator::I32AtomicRmw8XchgU { .. }
			| Operator::I32AtomicRmw16XchgU { .. }
			| Operator::I64AtomicRmw8XchgU { .. }
			| Operator::I64AtomicRmw16XchgU { .. }
			| Operator::I64AtomicRmw32XchgU { .. }
			| Operator::I32AtomicRmwCmpxchg { .. }
			| Operator::I64AtomicRmwCmpxchg { .. }
			| Operator::I32AtomicRmw8CmpxchgU { .. }
			| Operator::I32AtomicRmw16CmpxchgU { .. }
			| Operator::I64AtomicRmw8CmpxchgU { .. }
			| Operator::I64AtomicRmw16CmpxchgU { .. }
			| Operator::I64AtomicRmw32CmpxchgU { .. }
			| Operator::TryTable { .. }
			| Operator::Throw { .. }
			| Operator::ThrowRef
			| Operator::Try { .. }
			| Operator::Catch { .. }
			| Operator::Rethrow { .. }
			| Operator::Delegate { .. }
			| Operator::CatchAll
			| Operator::GlobalAtomicGet { .. }
			| Operator::GlobalAtomicSet { .. }
			| Operator::GlobalAtomicRmwAdd { .. }
			| Operator::GlobalAtomicRmwSub { .. }
			| Operator::GlobalAtomicRmwAnd { .. }
			| Operator::GlobalAtomicRmwOr { .. }
			| Operator::GlobalAtomicRmwXor { .. }
			| Operator::GlobalAtomicRmwXchg { .. }
			| Operator::GlobalAtomicRmwCmpxchg { .. }
			| Operator::TableAtomicGet { .. }
			| Operator::TableAtomicSet { .. }
			| Operator::TableAtomicRmwXchg { .. }
			| Operator::TableAtomicRmwCmpxchg { .. }
			| Operator::StructAtomicGet { .. }
			| Operator::StructAtomicGetS { .. }
			| Operator::StructAtomicGetU { .. }
			| Operator::StructAtomicSet { .. }
			| Operator::StructAtomicRmwAdd { .. }
			| Operator::StructAtomicRmwSub { .. }
			| Operator::StructAtomicRmwAnd { .. }
			| Operator::StructAtomicRmwOr { .. }
			| Operator::StructAtomicRmwXor { .. }
			| Operator::StructAtomicRmwXchg { .. }
			| Operator::StructAtomicRmwCmpxchg { .. }
			| Operator::ArrayAtomicGet { .. }
			| Operator::ArrayAtomicGetS { .. }
			| Operator::ArrayAtomicGetU { .. }
			| Operator::ArrayAtomicSet { .. }
			| Operator::ArrayAtomicRmwAdd { .. }
			| Operator::ArrayAtomicRmwSub { .. }
			| Operator::ArrayAtomicRmwAnd { .. }
			| Operator::ArrayAtomicRmwOr { .. }
			| Operator::ArrayAtomicRmwXor { .. }
			| Operator::ArrayAtomicRmwXchg { .. }
			| Operator::ArrayAtomicRmwCmpxchg { .. }
			| Operator::RefI31Shared
			| Operator::CallRef { .. }
			| Operator::ReturnCallRef { .. }
			| Operator::RefAsNonNull
			| Operator::BrOnNull { .. }
			| Operator::BrOnNonNull { .. }
			| Operator::ContNew { .. }
			| Operator::ContBind { .. }
			| Operator::Suspend { .. }
			| Operator::Resume { .. }
			| Operator::ResumeThrow { .. }
			| Operator::Switch { .. }
			| Operator::I64Add128
			| Operator::I64Sub128
			| Operator::I64MulWideS
			| Operator::I64MulWideU
			| _ => {
				unimplemented!("WebAssembly operator {operator:?} is not supported")
			}
		}
	}

	#[expect(
		clippy::too_many_arguments,
		reason = "entry point requires all builder state"
	)]
	pub fn run(
		&mut self,
		graph: &mut ControlFlowGraph,
		types: &Types,
		function_type: BlockType,
		locals: u16,
		operators: OperatorsReader<'_>,
	) {
		self.stack_builder
			.set_function_data(types, function_type, locals);

		self.code_builder.clear();
		self.code_builder.add_basic_block(1);

		for operator in operators.into_iter().map(Result::unwrap) {
			self.handle_operator(types, &operator);
		}

		self.code_builder.add_basic_block(0);
		self.code_builder.swap_contents(graph);
	}
}
