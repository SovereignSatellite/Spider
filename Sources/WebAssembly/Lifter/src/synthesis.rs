use super::{
	environment::{DataModePlan, DataPlan, ElementKindPlan, ElementPlan, EnvironmentPlan},
	graph_builder::GraphBuilder,
};

fn emit_global_initializations(builder: &mut GraphBuilder, environment: &EnvironmentPlan) {
	let first = environment.imported_global_count();

	for (offset, initializer) in environment.globals.iter().enumerate() {
		let value = initializer.emit(builder);
		let global = u32::try_from(first + offset).unwrap();

		builder.emit_global_set(global, value);
	}
}

fn emit_table_initializers(builder: &mut GraphBuilder, environment: &EnvironmentPlan) {
	let first = environment.imported_table_count();

	for (offset, table) in environment.tables.iter().enumerate() {
		let Some(initializer) = table.initializer else {
			continue;
		};

		let value = initializer.emit(builder);
		let size = builder.emit_size(table.minimum);
		let destination = u32::try_from(first + offset).unwrap();

		builder.emit_table_fill(destination, value, size);
	}
}

fn emit_element_initialization(builder: &mut GraphBuilder, elements: u32, element: &ElementPlan) {
	match &element.kind {
		ElementKindPlan::Active { table, offset } => {
			let table_offset = offset.emit(builder);
			let count = u32::try_from(element.items.len()).unwrap();
			let size = builder.emit_size(count);

			builder.emit_table_init(*table, table_offset, elements, size);
			builder.emit_elements_drop(elements);
		}
		ElementKindPlan::Passive => {}
		ElementKindPlan::Declared => builder.emit_elements_drop(elements),
	}
}

fn emit_element_initializations(builder: &mut GraphBuilder, environment: &EnvironmentPlan) {
	for (index, element) in environment.elements.iter().enumerate() {
		emit_element_initialization(builder, u32::try_from(index).unwrap(), element);
	}
}

fn emit_data_initialization(builder: &mut GraphBuilder, data_index: u32, data: &DataPlan) {
	match &data.kind {
		DataModePlan::Passive => {}
		DataModePlan::Active { memory, offset } => {
			let memory_offset = offset.emit(builder);
			let count = u32::try_from(data.bytes.len()).unwrap();
			let size = builder.emit_size(count);

			builder.emit_memory_init(*memory, memory_offset, data_index, size);
			builder.emit_data_drop(data_index);
		}
	}
}

fn emit_data_initializations(builder: &mut GraphBuilder, environment: &EnvironmentPlan) {
	for (index, data) in environment.datas.iter().enumerate() {
		emit_data_initialization(builder, u32::try_from(index).unwrap(), data);
	}
}

fn emit_start_call(builder: &mut GraphBuilder, start: Option<u32>) {
	let Some(start) = start else {
		return;
	};

	builder.emit_start_call(start);
}

pub fn synthesize_initialization(builder: &mut GraphBuilder, environment: &EnvironmentPlan) {
	emit_global_initializations(builder, environment);
	emit_table_initializers(builder, environment);
	emit_element_initializations(builder, environment);
	emit_data_initializations(builder, environment);
	emit_start_call(builder, environment.start);
}
