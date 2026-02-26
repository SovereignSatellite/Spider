#![no_std]

extern crate alloc;

use alloc::{sync::Arc, vec::Vec};
use ir_graph::{
	DataFlowGraph, Link, Node,
	control::{Export, Import, OmegaIn, OmegaOut},
	simple::{
		Apply, Fence, GlobalGet, GlobalNew, GlobalSet, Location, MemoryCopy, MemoryDrop, MemoryNew,
		TableCopy, TableDrop, TableFill, TableNew, TableSet,
	},
};
use wasmparser::{ConstExpr, ElementItems, FunctionBody, SectionLimited, ValType};
use web_assembly_builder::Types;
use web_assembly_graph::instruction::MemorySize;

use self::{function_lifter::FunctionLifter, global_state::GlobalState, sections::Sections};

mod control_flow_lifter;
mod function_lifter;
mod global_state;
mod sections;

fn get_element_count(items: &ElementItems) -> u32 {
	match items {
		ElementItems::Functions(section) => section.count(),
		ElementItems::Expressions(_, section) => section.count(),
	}
}

fn add_table_from_type(graph: &mut DataFlowGraph, table_type: wasmparser::TableType) -> Link {
	let wasmparser::TableType {
		initial, maximum, ..
	} = table_type;

	let minimum = initial.try_into().unwrap();
	let maximum = maximum.map_or(u32::MAX, |maximum| maximum.try_into().unwrap());

	TableNew::add_into(graph, Vec::new(), minimum, maximum)
}

fn add_table_from_items(graph: &mut DataFlowGraph, items: ElementItems) -> Link {
	let count = match items {
		ElementItems::Functions(section) => section.count(),
		ElementItems::Expressions(_, section) => section.count(),
	};

	TableNew::add_into(graph, Vec::new(), count, count)
}

fn add_memory_from_type(graph: &mut DataFlowGraph, memory_type: wasmparser::MemoryType) -> Link {
	let wasmparser::MemoryType {
		initial, maximum, ..
	} = memory_type;

	let page = u32::try_from(MemorySize::PAGE_SIZE).unwrap();

	let minimum = u32::try_from(initial).unwrap().saturating_mul(page);
	let maximum = maximum.map_or(u32::MAX, |maximum| {
		u32::try_from(maximum).unwrap().saturating_mul(page)
	});

	MemoryNew::add_into(graph, Vec::new(), minimum, maximum)
}

fn add_memory_from_data(graph: &mut DataFlowGraph, data: &[u8]) -> Link {
	let data = Arc::<[u8]>::from(data);
	let len = data.len().try_into().unwrap();

	MemoryNew::add_into(graph, alloc::vec![(data, 0)], len, len)
}

fn add_global_from_null(graph: &mut DataFlowGraph) -> Link {
	let null = Node::add_null_into(graph);

	GlobalNew::add_into(graph, null)
}

pub struct WebAssemblyLifter {
	function_lifter: FunctionLifter,
	global_state: GlobalState,
	types: Types,
}

impl WebAssemblyLifter {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			function_lifter: FunctionLifter::new(),
			global_state: GlobalState::new(),
			types: Types::new(),
		}
	}

	fn handle_import_section(
		&mut self,
		graph: &mut DataFlowGraph,
		omega_in: u32,
		section: SectionLimited<wasmparser::Import>,
	) {
		let environment = Link(omega_in, OmegaIn::ENVIRONMENT_PORT);

		for wasmparser::Import { module, name, ty } in section.into_iter().map(Result::unwrap) {
			let mut link = Import::add_into(graph, environment, module.into(), name.into());

			if let wasmparser::TypeRef::Func(function) = ty {
				self.types.add_function(function);

				link = GlobalNew::add_into(graph, link);
			}

			self.global_state.get_mut_type_ref(ty).push(link);
		}
	}

	fn handle_function_section(&mut self, graph: &mut DataFlowGraph, section: SectionLimited<u32>) {
		let len = section.count().try_into().unwrap();

		self.types.add_functions(section);

		self.global_state
			.functions
			.extend(core::iter::repeat_with(|| add_global_from_null(graph)).take(len));
	}

	fn build_expression(
		&mut self,
		graph: &mut DataFlowGraph,
		code: &ConstExpr,
		result: ValType,
	) -> Link {
		let code = code.get_operators_reader();

		self.function_lifter
			.build_expression(graph, code, result, &self.types, &self.global_state)
	}

	fn handle_table_declarations(
		&mut self,
		graph: &mut DataFlowGraph,
		section: SectionLimited<wasmparser::Table>,
	) {
		self.global_state.tables.extend(
			section
				.into_iter()
				.map(Result::unwrap)
				.map(|wasmparser::Table { ty, .. }| add_table_from_type(graph, ty)),
		);
	}

	fn load_table_fill(
		&mut self,
		graph: &mut DataFlowGraph,
		reference: Link,
		code: &ConstExpr,
		ty: wasmparser::TableType,
	) -> Link {
		let destination = Location {
			reference,
			offset: Node::add_i32_into(graph, 0),
		};

		let source = self.build_expression(graph, code, ValType::Ref(ty.element_type));
		let size = Node::add_i32_into(graph, ty.initial.try_into().unwrap());

		TableFill::add_into(graph, destination, source, size)
	}

	fn initialize_table(
		&mut self,
		graph: &mut DataFlowGraph,
		index: usize,
		table: &wasmparser::Table,
	) {
		let wasmparser::TableInit::Expr(code) = &table.init else {
			return;
		};

		let destination = self.global_state.tables[index];

		self.global_state.tables[index] = self.load_table_fill(graph, destination, code, table.ty);
	}

	fn handle_table_initializations(
		&mut self,
		graph: &mut DataFlowGraph,
		section: SectionLimited<wasmparser::Table>,
	) {
		let start = self.global_state.tables.len() - usize::try_from(section.count()).unwrap();

		for (offset, table) in section.into_iter().map(Result::unwrap).enumerate() {
			self.initialize_table(graph, start + offset, &table);
		}
	}

	fn handle_element_declarations(
		&mut self,
		graph: &mut DataFlowGraph,
		section: SectionLimited<wasmparser::Element>,
	) {
		self.global_state.elements.extend(
			section
				.into_iter()
				.map(Result::unwrap)
				.map(|wasmparser::Element { items, .. }| add_table_from_items(graph, items)),
		);
	}

	fn set_table_functions(
		&self,
		graph: &mut DataFlowGraph,
		section: SectionLimited<u32>,
		mut element: Link,
	) -> Link {
		let functions = &self.global_state.functions;

		for (function, offset) in section.into_iter().map(Result::unwrap).zip(0..) {
			let function = functions[usize::try_from(function).unwrap()];
			let source = GlobalGet::add_into(graph, function).0;
			let destination = Location {
				reference: element,
				offset: Node::add_i32_into(graph, offset),
			};

			element = TableSet::add_into(graph, destination, source);
		}

		element
	}

	fn set_table_expressions(
		&mut self,
		graph: &mut DataFlowGraph,
		section: SectionLimited<ConstExpr>,
		r#type: wasmparser::RefType,
		mut element: Link,
	) -> Link {
		for (code, offset) in section.into_iter().map(Result::unwrap).zip(0..) {
			let source = self.build_expression(graph, &code, ValType::Ref(r#type));
			let destination = Location {
				reference: element,
				offset: Node::add_i32_into(graph, offset),
			};

			element = TableSet::add_into(graph, destination, source);
		}

		element
	}

	fn initialize_element(
		&mut self,
		graph: &mut DataFlowGraph,
		items: ElementItems,
		element: Link,
	) -> Link {
		match items {
			ElementItems::Functions(section) => self.set_table_functions(graph, section, element),
			ElementItems::Expressions(r#type, section) => {
				self.set_table_expressions(graph, section, r#type, element)
			}
		}
	}

	fn load_table_copy(
		&mut self,
		graph: &mut DataFlowGraph,
		reference: Link,
		offset: &ConstExpr,
		elements: Link,
		size: i32,
	) -> Link {
		let destination = Location {
			reference,
			offset: self.build_expression(graph, offset, ValType::I32),
		};

		let source = Location {
			reference: elements,
			offset: Node::add_i32_into(graph, 0),
		};

		let size = Node::add_i32_into(graph, size);

		TableCopy::add_into(graph, destination, source, size).0
	}

	fn action_element(
		&mut self,
		graph: &mut DataFlowGraph,
		elements: Link,
		size: i32,
		element_kind: &wasmparser::ElementKind,
	) -> Link {
		match element_kind {
			wasmparser::ElementKind::Active {
				table_index,
				offset_expr,
			} => {
				let index: usize = table_index.unwrap_or(0).try_into().unwrap();
				let reference = self.global_state.tables[index];

				self.global_state.tables[index] =
					self.load_table_copy(graph, reference, offset_expr, elements, size);

				TableDrop::add_into(graph, elements)
			}
			wasmparser::ElementKind::Passive => elements,
			wasmparser::ElementKind::Declared => TableDrop::add_into(graph, elements),
		}
	}

	fn handle_element_initializations(
		&mut self,
		graph: &mut DataFlowGraph,
		section: SectionLimited<wasmparser::Element>,
	) {
		for (index, element) in section.into_iter().map(Result::unwrap).enumerate() {
			let size = get_element_count(&element.items);
			let size = i32::from_ne_bytes(size.to_ne_bytes());

			let link = self.global_state.elements[index];
			let link = self.initialize_element(graph, element.items, link);
			let link = self.action_element(graph, link, size, &element.kind);

			self.global_state.elements[index] = link;
		}
	}

	fn handle_memory_section(
		&mut self,
		graph: &mut DataFlowGraph,
		section: SectionLimited<wasmparser::MemoryType>,
	) {
		self.global_state.memories.extend(
			section
				.into_iter()
				.map(Result::unwrap)
				.map(|memory_type| add_memory_from_type(graph, memory_type)),
		);
	}

	fn load_memory_copy(
		&mut self,
		graph: &mut DataFlowGraph,
		reference: Link,
		offset: &ConstExpr,
		data: Link,
		size: i32,
	) -> Link {
		let destination = Location {
			reference,
			offset: self.build_expression(graph, offset, ValType::I32),
		};

		let source = Location {
			reference: data,
			offset: Node::add_i32_into(graph, 0),
		};

		let size = Node::add_i32_into(graph, size);

		MemoryCopy::add_into(graph, destination, source, size).0
	}

	fn handle_data_declarations(
		&mut self,
		graph: &mut DataFlowGraph,
		section: SectionLimited<wasmparser::Data>,
	) {
		self.global_state.datas.extend(
			section
				.into_iter()
				.map(Result::unwrap)
				.map(|wasmparser::Data { data, .. }| add_memory_from_data(graph, data)),
		);
	}

	fn action_data(
		&mut self,
		graph: &mut DataFlowGraph,
		data: Link,
		size: i32,
		data_kind: &wasmparser::DataKind,
	) -> Link {
		match data_kind {
			wasmparser::DataKind::Passive => data,
			wasmparser::DataKind::Active {
				memory_index,
				offset_expr,
			} => {
				let index = usize::try_from(*memory_index).unwrap();
				let reference = self.global_state.memories[index];

				self.global_state.memories[index] =
					self.load_memory_copy(graph, reference, offset_expr, data, size);

				MemoryDrop::add_into(graph, data)
			}
		}
	}

	fn handle_data_initializations(
		&mut self,
		graph: &mut DataFlowGraph,
		section: SectionLimited<wasmparser::Data>,
	) {
		for (index, data) in section.into_iter().map(Result::unwrap).enumerate() {
			let size = u32::try_from(data.data.len()).unwrap();
			let size = i32::from_ne_bytes(size.to_ne_bytes());

			let link = self.global_state.datas[index];
			let link = self.action_data(graph, link, size, &data.kind);

			self.global_state.datas[index] = link;
		}
	}

	fn handle_global_declarations(
		&mut self,
		graph: &mut DataFlowGraph,
		section: &SectionLimited<wasmparser::Global>,
	) {
		let len = section.count().try_into().unwrap();

		self.global_state
			.globals
			.extend(core::iter::repeat_with(|| add_global_from_null(graph)).take(len));
	}

	fn initialize_global(
		&mut self,
		graph: &mut DataFlowGraph,
		index: usize,
		global: &wasmparser::Global,
	) {
		let source = self.build_expression(graph, &global.init_expr, global.ty.content_type);

		self.global_state.globals[index] =
			GlobalSet::add_into(graph, self.global_state.globals[index], source);
	}

	fn handle_global_initializations(
		&mut self,
		graph: &mut DataFlowGraph,
		section: SectionLimited<wasmparser::Global>,
	) {
		let start = self.global_state.globals.len() - usize::try_from(section.count()).unwrap();

		for (offset, global) in section.into_iter().map(Result::unwrap).enumerate() {
			self.initialize_global(graph, start + offset, &global);
		}
	}

	#[expect(
		clippy::needless_pass_by_ref_mut,
		clippy::needless_pass_by_value,
		unused_variables
	)]
	fn handle_tag_section(
		&mut self,
		graph: &mut DataFlowGraph,
		section: SectionLimited<wasmparser::TagType>,
	) {
		if section.count() == 0 {
			return;
		}

		unimplemented!("`Tag`s are not supported yet")
	}

	fn build_function(
		&mut self,
		graph: &mut DataFlowGraph,
		body: &FunctionBody,
		index: usize,
	) -> u32 {
		self.function_lifter.build_function(
			graph,
			body,
			index.try_into().unwrap(),
			&self.types,
			&self.global_state,
		)
	}

	fn handle_code_section(
		&mut self,
		graph: &mut DataFlowGraph,
		section: &[FunctionBody],
		mut imports: usize,
	) {
		for body in section {
			let lambda_out = self.build_function(graph, body, imports);
			let functions = &mut self.global_state.functions;

			functions[imports] =
				GlobalSet::add_into(graph, functions[imports], Link(lambda_out, 0));

			imports += 1;
		}
	}

	fn load_export_information(
		&self,
		graph: &mut DataFlowGraph,
		export: wasmparser::Export,
	) -> Export {
		let index = usize::try_from(export.index).unwrap();
		let mut reference = self.global_state.get_external_kind(export.kind)[index];

		if export.kind == wasmparser::ExternalKind::Func {
			reference = GlobalGet::add_into(graph, reference).0;
		}

		Export {
			identifier: export.name.into(),
			reference,
		}
	}

	fn handle_export_section(
		&self,
		graph: &mut DataFlowGraph,
		section: SectionLimited<wasmparser::Export>,
	) -> Vec<Export> {
		section
			.into_iter()
			.map(Result::unwrap)
			.map(|export| self.load_export_information(graph, export))
			.collect()
	}

	fn handle_start_section(
		&self,
		graph: &mut DataFlowGraph,
		omega_in: u32,
		start: Option<u32>,
	) -> Link {
		let state = Link(omega_in, OmegaIn::STATE_PORT);

		start.map_or(state, |start| {
			let function = self.global_state.functions[usize::try_from(start).unwrap()];
			let function = GlobalGet::add_into(graph, function).0;
			let apply = Apply::add_into(graph, function, alloc::vec![state], 0, 1);

			Link(apply, 0)
		})
	}

	fn handle_module(
		&self,
		graph: &mut DataFlowGraph,
		omega_in: u32,
		start: Link,
		exports: Vec<Export>,
	) -> u32 {
		let mut states = Vec::new();

		self.global_state.retrieve_all_mutable(&mut states);
		states.push(start);

		let start = Fence::add_into(graph, list::resizable::Resizable::Heap(states));

		OmegaOut::add_into(graph, omega_in, start, exports)
	}

	pub fn run(&mut self, graph: &mut DataFlowGraph, data: &[u8]) -> u32 {
		let sections = Sections::load(data);

		self.global_state.clear();
		self.types.clear();
		self.types.add_sub_types(sections.types);

		let omega_in = OmegaIn::add_into(graph);

		self.handle_import_section(graph, omega_in, sections.imports);

		let function_imports = self.global_state.functions.len();

		self.handle_table_declarations(graph, sections.tables.clone());
		self.handle_element_declarations(graph, sections.elements.clone());
		self.handle_data_declarations(graph, sections.datas.clone());
		self.handle_global_declarations(graph, &sections.globals);

		self.handle_function_section(graph, sections.functions);
		self.handle_memory_section(graph, sections.memories);
		self.handle_tag_section(graph, sections.tags);
		self.handle_code_section(graph, &sections.code, function_imports);

		self.handle_table_initializations(graph, sections.tables);
		self.handle_element_initializations(graph, sections.elements);
		self.handle_data_initializations(graph, sections.datas);
		self.handle_global_initializations(graph, sections.globals);

		let start = self.handle_start_section(graph, omega_in, sections.start);
		let exports = self.handle_export_section(graph, sections.exports);

		self.handle_module(graph, omega_in, start, exports)
	}
}

impl Default for WebAssemblyLifter {
	fn default() -> Self {
		Self::new()
	}
}
