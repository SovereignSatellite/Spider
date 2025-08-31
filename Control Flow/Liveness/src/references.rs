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

fn handle_ref_function(references: &mut Vec<Reference>, ref_function: RefFunction) {
	let RefFunction {
		destination: _,
		function,
	} = ref_function;

	read_function(references, function);
}

fn handle_global_get(references: &mut Vec<Reference>, global_get: GlobalGet) {
	let GlobalGet {
		destination: _,
		source,
	} = global_get;

	read_global(references, source);
}

fn handle_global_set(references: &mut Vec<Reference>, global_set: GlobalSet) {
	let GlobalSet {
		destination,
		source: _,
	} = global_set;

	write_global(references, destination);
}

fn handle_table_get(references: &mut Vec<Reference>, table_get: TableGet) {
	let TableGet {
		destination: _,
		source,
	} = table_get;

	read_table(references, source.reference);
}

fn handle_table_set(references: &mut Vec<Reference>, table_set: TableSet) {
	let TableSet {
		destination,
		source: _,
	} = table_set;

	write_table(references, destination.reference);
}

fn handle_table_size(references: &mut Vec<Reference>, table_size: TableSize) {
	let TableSize {
		reference,
		destination: _,
	} = table_size;

	read_table(references, reference);
}

fn handle_table_grow(references: &mut Vec<Reference>, table_grow: TableGrow) {
	let TableGrow {
		reference,
		destination: _,
		size: _,
		initializer: _,
	} = table_grow;

	write_table(references, reference);
}

fn handle_table_fill(references: &mut Vec<Reference>, table_fill: TableFill) {
	let TableFill {
		destination,
		source: _,
		size: _,
	} = table_fill;

	write_table(references, destination.reference);
}

fn handle_table_copy(references: &mut Vec<Reference>, table_copy: TableCopy) {
	let TableCopy {
		destination,
		source,
		size: _,
	} = table_copy;

	write_table(references, destination.reference);
	read_table(references, source.reference);
}

fn handle_table_init(references: &mut Vec<Reference>, table_init: TableInit) {
	let TableInit {
		destination,
		source,
		size: _,
	} = table_init;

	write_table(references, destination.reference);
	read_elements(references, source.reference);
}

fn handle_elements_drop(references: &mut Vec<Reference>, elements_drop: ElementsDrop) {
	let ElementsDrop { source } = elements_drop;

	write_elements(references, source);
}

fn handle_memory_load(references: &mut Vec<Reference>, memory_load: MemoryLoad) {
	let MemoryLoad {
		destination: _,
		source,
		r#type: _,
	} = memory_load;

	read_memory(references, source.reference);
}

fn handle_memory_store(references: &mut Vec<Reference>, memory_store: MemoryStore) {
	let MemoryStore {
		destination,
		source: _,
		r#type: _,
	} = memory_store;

	write_memory(references, destination.reference);
}

fn handle_memory_size(references: &mut Vec<Reference>, memory_size: MemorySize) {
	let MemorySize {
		reference,
		destination: _,
	} = memory_size;

	read_memory(references, reference);
}

fn handle_memory_grow(references: &mut Vec<Reference>, memory_grow: MemoryGrow) {
	let MemoryGrow {
		reference,
		destination: _,
		size: _,
	} = memory_grow;

	write_memory(references, reference);
}

fn handle_memory_fill(references: &mut Vec<Reference>, memory_fill: MemoryFill) {
	let MemoryFill {
		destination,
		byte: _,
		size: _,
	} = memory_fill;

	write_memory(references, destination.reference);
}

fn handle_memory_copy(references: &mut Vec<Reference>, memory_copy: MemoryCopy) {
	let MemoryCopy {
		destination,
		source,
		size: _,
	} = memory_copy;

	write_memory(references, destination.reference);
	read_memory(references, source.reference);
}

fn handle_memory_init(references: &mut Vec<Reference>, memory_init: MemoryInit) {
	let MemoryInit {
		destination,
		source,
		size: _,
	} = memory_init;

	write_memory(references, destination.reference);
	read_data(references, source.reference);
}

fn handle_data_drop(references: &mut Vec<Reference>, data_drop: DataDrop) {
	let DataDrop { source } = data_drop;

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

		Instruction::RefFunction(ref_function) => handle_ref_function(references, ref_function),
		Instruction::GlobalGet(global_get) => handle_global_get(references, global_get),
		Instruction::GlobalSet(global_set) => handle_global_set(references, global_set),
		Instruction::TableGet(table_get) => handle_table_get(references, table_get),
		Instruction::TableSet(table_set) => handle_table_set(references, table_set),
		Instruction::TableSize(table_size) => handle_table_size(references, table_size),
		Instruction::TableGrow(table_grow) => handle_table_grow(references, table_grow),
		Instruction::TableFill(table_fill) => handle_table_fill(references, table_fill),
		Instruction::TableCopy(table_copy) => handle_table_copy(references, table_copy),
		Instruction::TableInit(table_init) => handle_table_init(references, table_init),
		Instruction::ElementsDrop(elements_drop) => handle_elements_drop(references, elements_drop),
		Instruction::MemoryLoad(memory_load) => handle_memory_load(references, memory_load),
		Instruction::MemoryStore(memory_store) => handle_memory_store(references, memory_store),
		Instruction::MemorySize(memory_size) => handle_memory_size(references, memory_size),
		Instruction::MemoryGrow(memory_grow) => handle_memory_grow(references, memory_grow),
		Instruction::MemoryFill(memory_fill) => handle_memory_fill(references, memory_fill),
		Instruction::MemoryCopy(memory_copy) => handle_memory_copy(references, memory_copy),
		Instruction::MemoryInit(memory_init) => handle_memory_init(references, memory_init),
		Instruction::DataDrop(data_drop) => handle_data_drop(references, data_drop),
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
