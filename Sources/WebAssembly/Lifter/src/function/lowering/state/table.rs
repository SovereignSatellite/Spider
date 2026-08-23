use ir_graph::{Node, operation};
use web_assembly_control_flow::instruction::{
	ElementsDrop, ReferenceType, TableCopy, TableFill, TableGet, TableGrow, TableInit, TableSet,
	TableSize,
};

use super::LoweringState;

impl LoweringState {
	pub fn handle_table_get(&mut self, nodes: &mut Vec<Node>, instruction: TableGet) {
		let TableGet {
			destination,
			source,
		} = instruction;

		let state = self.load_location(ReferenceType::Table, source);
		let (result, state) = operation::TableGet::add_into(nodes, state);

		self.locals[usize::from(destination)] = result;

		self.dependencies
			.set(ReferenceType::Table, source.reference, state);
	}

	pub fn handle_table_set(&mut self, nodes: &mut Vec<Node>, instruction: TableSet) {
		let TableSet {
			destination,
			source,
		} = instruction;

		let state = operation::TableSet::add_into(
			nodes,
			self.load_location(ReferenceType::Table, destination),
			self.locals[usize::from(source)],
		);

		self.dependencies
			.set(ReferenceType::Table, destination.reference, state);
	}

	pub fn handle_table_size(&mut self, nodes: &mut Vec<Node>, instruction: TableSize) {
		let TableSize { destination, table } = instruction;

		let state = self.dependencies.get(ReferenceType::Table, table);
		let (result, state) = operation::TableSize::add_into(nodes, state);

		self.locals[usize::from(destination)] = result;

		self.dependencies.set(ReferenceType::Table, table, state);
	}

	pub fn handle_table_grow(&mut self, nodes: &mut Vec<Node>, instruction: TableGrow) {
		let TableGrow {
			destination,
			table,
			size,
			initializer,
		} = instruction;

		let (result, state) = operation::TableGrow::add_into(
			nodes,
			self.dependencies.get(ReferenceType::Table, table),
			self.locals[usize::from(initializer)],
			self.locals[usize::from(size)],
		);

		self.locals[usize::from(destination)] = result;

		self.dependencies.set(ReferenceType::Table, table, state);
	}

	pub fn handle_table_fill(&mut self, nodes: &mut Vec<Node>, instruction: TableFill) {
		let TableFill {
			destination,
			source,
			size,
		} = instruction;

		let state = operation::TableFill::add_into(
			nodes,
			self.load_location(ReferenceType::Table, destination),
			self.locals[usize::from(source)],
			self.locals[usize::from(size)],
		);

		self.dependencies
			.set(ReferenceType::Table, destination.reference, state);
	}

	pub fn handle_table_copy(&mut self, nodes: &mut Vec<Node>, instruction: TableCopy) {
		let TableCopy {
			destination,
			source,
			size,
		} = instruction;

		let (destination_state, source_state) = operation::TableCopy::add_into(
			nodes,
			self.load_location(ReferenceType::Table, destination),
			self.load_location(ReferenceType::Table, source),
			self.locals[usize::from(size)],
		);

		self.dependencies.set(
			ReferenceType::Table,
			destination.reference,
			destination_state,
		);

		self.dependencies
			.set(ReferenceType::Table, source.reference, source_state);
	}

	pub fn handle_table_init(&mut self, nodes: &mut Vec<Node>, instruction: TableInit) {
		let TableInit {
			destination,
			source,
			size,
		} = instruction;

		let (destination_state, source_state) = operation::TableCopy::add_into(
			nodes,
			self.load_location(ReferenceType::Table, destination),
			self.load_location(ReferenceType::Elements, source),
			self.locals[usize::from(size)],
		);

		self.dependencies.set(
			ReferenceType::Table,
			destination.reference,
			destination_state,
		);

		self.dependencies
			.set(ReferenceType::Elements, source.reference, source_state);
	}

	pub fn handle_elements_drop(&mut self, nodes: &mut Vec<Node>, instruction: ElementsDrop) {
		let ElementsDrop { source } = instruction;

		let state = self.dependencies.get(ReferenceType::Elements, source);
		let state = operation::TableDrop::add_into(nodes, state);

		self.dependencies
			.set(ReferenceType::Elements, source, state);
	}
}
