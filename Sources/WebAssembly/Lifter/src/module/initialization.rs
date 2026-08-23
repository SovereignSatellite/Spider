use web_assembly_control_flow::{
	BasicBlock, ControlFlowGraph,
	instruction::{
		Call, DataDrop, ElementsDrop, F32Constant, F64Constant, GlobalGet, GlobalSet, I32Constant,
		I64Constant, Instruction, Location, MemoryInit, RefFunction, RefNull, ReservedLocal,
		TableFill, TableInit,
	},
};

use super::{
	constant_expression::ConstantExpression,
	plans::{DataModePlan, DataPlan, ElementKindPlan, ElementPlan, ModulePlan},
};

struct InitializationBuilder {
	graph: ControlFlowGraph,
	next_local: u16,
}

fn emit_constant_expression(
	builder: &mut InitializationBuilder,
	expression: ConstantExpression,
) -> u16 {
	match expression {
		ConstantExpression::I32(value) => builder.emit_i32_constant(value),
		ConstantExpression::I64(value) => builder.emit_i64_constant(value),
		ConstantExpression::F32(value) => builder.emit_f32_constant(value),
		ConstantExpression::F64(value) => builder.emit_f64_constant(value),
		ConstantExpression::RefNull => builder.emit_ref_null(),
		ConstantExpression::RefFunction(function) => builder.emit_ref_function(function),
		ConstantExpression::GlobalGet(global) => builder.emit_global_get(global),
	}
}

impl InitializationBuilder {
	const fn allocate_local(&mut self) -> u16 {
		let local = self.next_local;
		let Some(next_local) = local.checked_add(1) else {
			unreachable!()
		};

		self.next_local = next_local;

		local
	}

	fn push_instruction(&mut self, instruction: Instruction) {
		self.graph.instructions.push(instruction);
	}

	fn emit_i32_constant(&mut self, data: i32) -> u16 {
		let destination = self.allocate_local();

		self.push_instruction(Instruction::I32Constant(I32Constant { destination, data }));

		destination
	}

	fn emit_i64_constant(&mut self, data: i64) -> u16 {
		let destination = self.allocate_local();

		self.push_instruction(Instruction::I64Constant(I64Constant { destination, data }));

		destination
	}

	fn emit_f32_constant(&mut self, data: f32) -> u16 {
		let destination = self.allocate_local();

		self.push_instruction(Instruction::F32Constant(F32Constant { destination, data }));

		destination
	}

	fn emit_f64_constant(&mut self, data: f64) -> u16 {
		let destination = self.allocate_local();

		self.push_instruction(Instruction::F64Constant(F64Constant { destination, data }));

		destination
	}

	fn emit_size(&mut self, size: u32) -> u16 {
		let data = i32::from_ne_bytes(size.to_ne_bytes());

		self.emit_i32_constant(data)
	}

	fn emit_ref_null(&mut self) -> u16 {
		let destination = self.allocate_local();

		self.push_instruction(Instruction::RefNull(RefNull { destination }));

		destination
	}

	fn emit_ref_function(&mut self, function: u32) -> u16 {
		let destination = self.allocate_local();
		let function = function.try_into().unwrap();

		self.push_instruction(Instruction::RefFunction(RefFunction {
			destination,
			function,
		}));

		destination
	}

	fn emit_global_get(&mut self, source: u32) -> u16 {
		let destination = self.allocate_local();
		let source = source.try_into().unwrap();

		self.push_instruction(Instruction::GlobalGet(GlobalGet {
			destination,
			source,
		}));

		destination
	}

	fn emit_global_set(&mut self, destination: u32, source: u16) {
		let destination = destination.try_into().unwrap();

		self.push_instruction(Instruction::GlobalSet(GlobalSet {
			destination,
			source,
		}));
	}

	fn location(reference: u32, offset: u16) -> Location {
		Location {
			reference: reference.try_into().unwrap(),
			offset,
		}
	}

	fn emit_table_fill(&mut self, table: u32, source: u16, size: u16) {
		let zero = self.emit_i32_constant(0);
		let destination = Self::location(table, zero);

		self.push_instruction(Instruction::TableFill(TableFill {
			destination,
			source,
			size,
		}));
	}

	fn emit_table_init(&mut self, table: u32, table_offset: u16, elements: u32, size: u16) {
		let zero = self.emit_i32_constant(0);
		let destination = Self::location(table, table_offset);
		let source = Self::location(elements, zero);

		self.push_instruction(Instruction::TableInit(TableInit {
			destination,
			source,
			size,
		}));
	}

	fn emit_elements_drop(&mut self, source: u32) {
		let source = source.try_into().unwrap();

		self.push_instruction(Instruction::ElementsDrop(ElementsDrop { source }));
	}

	fn emit_memory_init(&mut self, memory: u32, memory_offset: u16, data: u32, size: u16) {
		let zero = self.emit_i32_constant(0);
		let destination = Self::location(memory, memory_offset);
		let source = Self::location(data, zero);

		self.push_instruction(Instruction::MemoryInit(MemoryInit {
			destination,
			source,
			size,
		}));
	}

	fn emit_data_drop(&mut self, source: u32) {
		let source = source.try_into().unwrap();

		self.push_instruction(Instruction::DataDrop(DataDrop { source }));
	}

	fn emit_start_call(&mut self, function: u32) {
		let function = self.emit_ref_function(function);
		let empty = self.allocate_local();

		self.push_instruction(Instruction::Call(Call {
			destinations: (empty, empty),
			sources: (empty, empty),
			function,
		}));
	}

	fn finish(mut self) -> ControlFlowGraph {
		let end = self.graph.instructions.len().try_into().unwrap();

		self.graph.basic_blocks.push(BasicBlock::from_range(0, end));

		self.graph
	}
}

pub fn build_initialization(mut graph: ControlFlowGraph, module: &ModulePlan) -> ControlFlowGraph {
	graph.instructions.clear();
	graph.basic_blocks.clear();

	let mut builder = InitializationBuilder {
		graph,
		next_local: ReservedLocal::COUNT,
	};

	emit_global_initializations(&mut builder, module);
	emit_table_initializers(&mut builder, module);
	emit_element_initializations(&mut builder, module);
	emit_data_initializations(&mut builder, module);
	emit_start_call(&mut builder, module.start);

	builder.finish()
}

fn emit_global_initializations(builder: &mut InitializationBuilder, module: &ModulePlan) {
	let first = module.imported_global_count();

	for (offset, initializer) in module.globals.iter().enumerate() {
		let value = emit_constant_expression(builder, *initializer);
		let global = u32::try_from(first + offset).unwrap();

		builder.emit_global_set(global, value);
	}
}

fn emit_table_initializers(builder: &mut InitializationBuilder, module: &ModulePlan) {
	let first = module.imported_table_count();

	for (offset, table) in module.tables.iter().enumerate() {
		let Some(initializer) = table.initializer else {
			continue;
		};

		let value = emit_constant_expression(builder, initializer);
		let size = builder.emit_size(table.minimum);
		let destination = u32::try_from(first + offset).unwrap();

		builder.emit_table_fill(destination, value, size);
	}
}

fn emit_element_initialization(
	builder: &mut InitializationBuilder,
	elements: u32,
	element: &ElementPlan,
) {
	match &element.kind {
		ElementKindPlan::Active { table, offset } => {
			let table_offset = emit_constant_expression(builder, *offset);
			let count = u32::try_from(element.items.len()).unwrap();
			let size = builder.emit_size(count);

			builder.emit_table_init(*table, table_offset, elements, size);
			builder.emit_elements_drop(elements);
		}
		ElementKindPlan::Passive => {}
		ElementKindPlan::Declared => builder.emit_elements_drop(elements),
	}
}

fn emit_element_initializations(builder: &mut InitializationBuilder, module: &ModulePlan) {
	for (index, element) in module.elements.iter().enumerate() {
		emit_element_initialization(builder, u32::try_from(index).unwrap(), element);
	}
}

fn emit_data_initialization(builder: &mut InitializationBuilder, data_index: u32, data: &DataPlan) {
	match &data.kind {
		DataModePlan::Passive => {}
		DataModePlan::Active { memory, offset } => {
			let memory_offset = emit_constant_expression(builder, *offset);
			let count = u32::try_from(data.bytes.len()).unwrap();
			let size = builder.emit_size(count);

			builder.emit_memory_init(*memory, memory_offset, data_index, size);
			builder.emit_data_drop(data_index);
		}
	}
}

fn emit_data_initializations(builder: &mut InitializationBuilder, module: &ModulePlan) {
	for (index, data) in module.datas.iter().enumerate() {
		emit_data_initialization(builder, u32::try_from(index).unwrap(), data);
	}
}

fn emit_start_call(builder: &mut InitializationBuilder, start: Option<u32>) {
	let Some(start) = start else {
		return;
	};

	builder.emit_start_call(start);
}
