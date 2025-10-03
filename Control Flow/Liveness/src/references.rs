use alloc::vec::Vec;
use control_flow_graph::instruction::{
	DataDrop, ElementsDrop, GlobalGet, GlobalSet, Instruction, MemoryCopy, MemoryFill, MemoryGrow,
	MemoryInit, MemoryLoad, MemorySize, MemoryStore, RefFunction, TableCopy, TableFill, TableGet,
	TableGrow, TableInit, TableSet, TableSize,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum ReferenceType {
	Function,

	Global,
	Table,
	Elements,
	Memory,
	Data,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Reference {
	pub r#type: ReferenceType,
	pub id: u16,
}

fn read_function(references: &mut Vec<Reference>, function: u16) {
	references.push(Reference {
		r#type: ReferenceType::Function,
		id: function,
	});
}

fn read_global(references: &mut Vec<Reference>, global: u16) {
	references.push(Reference {
		r#type: ReferenceType::Global,
		id: global,
	});
}

fn write_global(references: &mut Vec<Reference>, global: u16) {
	read_global(references, global);
}

fn read_table(references: &mut Vec<Reference>, table: u16) {
	references.push(Reference {
		r#type: ReferenceType::Table,
		id: table,
	});
}

fn write_table(references: &mut Vec<Reference>, table: u16) {
	read_table(references, table);
}

fn read_elements(references: &mut Vec<Reference>, elements: u16) {
	references.push(Reference {
		r#type: ReferenceType::Elements,
		id: elements,
	});
}

fn write_elements(references: &mut Vec<Reference>, elements: u16) {
	read_elements(references, elements);
}

fn read_memory(references: &mut Vec<Reference>, memory: u16) {
	references.push(Reference {
		r#type: ReferenceType::Memory,
		id: memory,
	});
}

fn write_memory(references: &mut Vec<Reference>, memory: u16) {
	read_memory(references, memory);
}

fn read_data(references: &mut Vec<Reference>, data: u16) {
	references.push(Reference {
		r#type: ReferenceType::Data,
		id: data,
	});
}

fn write_data(references: &mut Vec<Reference>, data: u16) {
	read_data(references, data);
}

fn handle_ref_function(references: &mut Vec<Reference>, instruction: RefFunction) {
	let RefFunction { function, .. } = instruction;

	read_function(references, function);
}

fn handle_global_get(references: &mut Vec<Reference>, instruction: GlobalGet) {
	let GlobalGet { source, .. } = instruction;

	read_global(references, source);
}

fn handle_global_set(references: &mut Vec<Reference>, instruction: GlobalSet) {
	let GlobalSet { destination, .. } = instruction;

	write_global(references, destination);
}

fn handle_table_get(references: &mut Vec<Reference>, instruction: TableGet) {
	let TableGet { source, .. } = instruction;

	read_table(references, source.reference);
}

fn handle_table_set(references: &mut Vec<Reference>, instruction: TableSet) {
	let TableSet { destination, .. } = instruction;

	write_table(references, destination.reference);
}

fn handle_table_size(references: &mut Vec<Reference>, instruction: TableSize) {
	let TableSize { table, .. } = instruction;

	read_table(references, table);
}

fn handle_table_grow(references: &mut Vec<Reference>, instruction: TableGrow) {
	let TableGrow { table, .. } = instruction;

	write_table(references, table);
}

fn handle_table_fill(references: &mut Vec<Reference>, instruction: TableFill) {
	let TableFill { destination, .. } = instruction;

	write_table(references, destination.reference);
}

fn handle_table_copy(references: &mut Vec<Reference>, instruction: TableCopy) {
	let TableCopy {
		destination,
		source,
		..
	} = instruction;

	write_table(references, destination.reference);
	read_table(references, source.reference);
}

fn handle_table_init(references: &mut Vec<Reference>, instruction: TableInit) {
	let TableInit {
		destination,
		source,
		..
	} = instruction;

	write_table(references, destination.reference);
	read_elements(references, source.reference);
}

fn handle_elements_drop(references: &mut Vec<Reference>, instruction: ElementsDrop) {
	let ElementsDrop { source } = instruction;

	write_elements(references, source);
}

fn handle_memory_load(references: &mut Vec<Reference>, instruction: MemoryLoad) {
	let MemoryLoad { source, .. } = instruction;

	read_memory(references, source.reference);
}

fn handle_memory_store(references: &mut Vec<Reference>, instruction: MemoryStore) {
	let MemoryStore { destination, .. } = instruction;

	write_memory(references, destination.reference);
}

fn handle_memory_size(references: &mut Vec<Reference>, instruction: MemorySize) {
	let MemorySize { memory, .. } = instruction;

	read_memory(references, memory);
}

fn handle_memory_grow(references: &mut Vec<Reference>, instruction: MemoryGrow) {
	let MemoryGrow { memory, .. } = instruction;

	write_memory(references, memory);
}

fn handle_memory_fill(references: &mut Vec<Reference>, instruction: MemoryFill) {
	let MemoryFill { destination, .. } = instruction;

	write_memory(references, destination.reference);
}

fn handle_memory_copy(references: &mut Vec<Reference>, instruction: MemoryCopy) {
	let MemoryCopy {
		destination,
		source,
		..
	} = instruction;

	write_memory(references, destination.reference);
	read_memory(references, source.reference);
}

fn handle_memory_init(references: &mut Vec<Reference>, instruction: MemoryInit) {
	let MemoryInit {
		destination,
		source,
		..
	} = instruction;

	write_memory(references, destination.reference);
	read_data(references, source.reference);
}

fn handle_data_drop(references: &mut Vec<Reference>, instruction: DataDrop) {
	let DataDrop { source } = instruction;

	write_data(references, source);
}

fn handle_instruction(references: &mut Vec<Reference>, instruction: Instruction) {
	match instruction {
		Instruction::LocalSet(_)
		| Instruction::LocalBranch(_)
		| Instruction::I32Constant(_)
		| Instruction::I64Constant(_)
		| Instruction::F32Constant(_)
		| Instruction::F64Constant(_)
		| Instruction::RefIsNull(_)
		| Instruction::RefNull(_)
		| Instruction::Call(_)
		| Instruction::Unreachable
		| Instruction::IntegerUnaryOperation(_)
		| Instruction::IntegerBinaryOperation(_)
		| Instruction::IntegerCompareOperation(_)
		| Instruction::IntegerNarrow(_)
		| Instruction::IntegerWiden(_)
		| Instruction::IntegerExtend(_)
		| Instruction::IntegerConvertToNumber(_)
		| Instruction::IntegerTransmuteToNumber(_)
		| Instruction::NumberUnaryOperation(_)
		| Instruction::NumberBinaryOperation(_)
		| Instruction::NumberCompareOperation(_)
		| Instruction::NumberNarrow(_)
		| Instruction::NumberWiden(_)
		| Instruction::NumberTruncateToInteger(_)
		| Instruction::NumberTransmuteToInteger(_) => {}

		Instruction::RefFunction(instruction) => handle_ref_function(references, instruction),
		Instruction::GlobalGet(instruction) => handle_global_get(references, instruction),
		Instruction::GlobalSet(instruction) => handle_global_set(references, instruction),
		Instruction::TableGet(instruction) => handle_table_get(references, instruction),
		Instruction::TableSet(instruction) => handle_table_set(references, instruction),
		Instruction::TableSize(instruction) => handle_table_size(references, instruction),
		Instruction::TableGrow(instruction) => handle_table_grow(references, instruction),
		Instruction::TableFill(instruction) => handle_table_fill(references, instruction),
		Instruction::TableCopy(instruction) => handle_table_copy(references, instruction),
		Instruction::TableInit(instruction) => handle_table_init(references, instruction),
		Instruction::ElementsDrop(instruction) => handle_elements_drop(references, instruction),
		Instruction::MemoryLoad(instruction) => handle_memory_load(references, instruction),
		Instruction::MemoryStore(instruction) => handle_memory_store(references, instruction),
		Instruction::MemorySize(instruction) => handle_memory_size(references, instruction),
		Instruction::MemoryGrow(instruction) => handle_memory_grow(references, instruction),
		Instruction::MemoryFill(instruction) => handle_memory_fill(references, instruction),
		Instruction::MemoryCopy(instruction) => handle_memory_copy(references, instruction),
		Instruction::MemoryInit(instruction) => handle_memory_init(references, instruction),
		Instruction::DataDrop(instruction) => handle_data_drop(references, instruction),
	}
}

pub fn track(references: &mut Vec<Reference>, instructions: &[Instruction]) {
	references.clear();

	for &instruction in instructions {
		handle_instruction(references, instruction);
	}

	references.sort_unstable();
	references.dedup();
}
