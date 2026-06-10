use core::mem;

use web_assembly_graph::{
	BasicBlock, ControlFlowGraph,
	instruction::{
		Call, DataDrop, ElementsDrop, F32Constant, F64Constant, GlobalGet, GlobalSet, I32Constant,
		I64Constant, Instruction, Location, MemoryInit, Name, RefFunction, RefNull, TableFill,
		TableInit,
	},
};

pub struct GraphBuilder {
	graph: ControlFlowGraph,
	next_local: u16,
}

impl GraphBuilder {
	pub const fn new() -> Self {
		Self {
			graph: ControlFlowGraph::new(),
			next_local: Name::COUNT,
		}
	}

	pub fn clear(&mut self) {
		self.graph.instructions.clear();
		self.graph.basic_blocks.clear();

		self.next_local = Name::COUNT;
	}

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

	pub fn emit_i32_constant(&mut self, data: i32) -> u16 {
		let destination = self.allocate_local();

		self.push_instruction(Instruction::I32Constant(I32Constant { destination, data }));

		destination
	}

	pub fn emit_i64_constant(&mut self, data: i64) -> u16 {
		let destination = self.allocate_local();

		self.push_instruction(Instruction::I64Constant(I64Constant { destination, data }));

		destination
	}

	pub fn emit_f32_constant(&mut self, data: f32) -> u16 {
		let destination = self.allocate_local();

		self.push_instruction(Instruction::F32Constant(F32Constant { destination, data }));

		destination
	}

	pub fn emit_f64_constant(&mut self, data: f64) -> u16 {
		let destination = self.allocate_local();

		self.push_instruction(Instruction::F64Constant(F64Constant { destination, data }));

		destination
	}

	pub fn emit_size(&mut self, size: u32) -> u16 {
		let data = i32::from_ne_bytes(size.to_ne_bytes());

		self.emit_i32_constant(data)
	}

	pub fn emit_ref_null(&mut self) -> u16 {
		let destination = self.allocate_local();

		self.push_instruction(Instruction::RefNull(RefNull { destination }));

		destination
	}

	pub fn emit_ref_function(&mut self, function: u32) -> u16 {
		let destination = self.allocate_local();
		let function = function.try_into().unwrap();

		self.push_instruction(Instruction::RefFunction(RefFunction {
			destination,
			function,
		}));

		destination
	}

	pub fn emit_global_get(&mut self, source: u32) -> u16 {
		let destination = self.allocate_local();
		let source = source.try_into().unwrap();

		self.push_instruction(Instruction::GlobalGet(GlobalGet {
			destination,
			source,
		}));

		destination
	}

	pub fn emit_global_set(&mut self, destination: u32, source: u16) {
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

	pub fn emit_table_fill(&mut self, table: u32, source: u16, size: u16) {
		let zero = self.emit_i32_constant(0);
		let destination = Self::location(table, zero);

		self.push_instruction(Instruction::TableFill(TableFill {
			destination,
			source,
			size,
		}));
	}

	pub fn emit_table_init(&mut self, table: u32, table_offset: u16, elements: u32, size: u16) {
		let zero = self.emit_i32_constant(0);
		let destination = Self::location(table, table_offset);
		let source = Self::location(elements, zero);

		self.push_instruction(Instruction::TableInit(TableInit {
			destination,
			source,
			size,
		}));
	}

	pub fn emit_elements_drop(&mut self, source: u32) {
		let source = source.try_into().unwrap();

		self.push_instruction(Instruction::ElementsDrop(ElementsDrop { source }));
	}

	pub fn emit_memory_init(&mut self, memory: u32, memory_offset: u16, data: u32, size: u16) {
		let zero = self.emit_i32_constant(0);
		let destination = Self::location(memory, memory_offset);
		let source = Self::location(data, zero);

		self.push_instruction(Instruction::MemoryInit(MemoryInit {
			destination,
			source,
			size,
		}));
	}

	pub fn emit_data_drop(&mut self, source: u32) {
		let source = source.try_into().unwrap();

		self.push_instruction(Instruction::DataDrop(DataDrop { source }));
	}

	pub fn emit_start_call(&mut self, function: u32) {
		let function = self.emit_ref_function(function);
		let empty = self.allocate_local();

		self.push_instruction(Instruction::Call(Call {
			destinations: (empty, empty),
			sources: (empty, empty),
			function,
		}));
	}

	pub fn finish(&mut self) -> ControlFlowGraph {
		let end = self.graph.instructions.len().try_into().unwrap();

		self.graph.basic_blocks.push(BasicBlock::from_range(0, end));

		mem::take(&mut self.graph)
	}
}
