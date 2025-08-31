#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use control_flow_builder::Types;
use data_flow_graph::{
	mvp::Location,
	nested::{Export, OmegaIn},
	DataFlowGraph, Link,
};
use wasmparser::{ConstExpr, ElementItems, FunctionBody, RecGroup, SectionLimited, ValType};

use self::{function_builder::FunctionBuilder, global_state::GlobalState, sections::Sections};

mod control_flow_converter;
mod function_builder;
mod global_state;
mod sections;

pub struct DataFlowBuilder {
	function_builder: FunctionBuilder,
	global_state: GlobalState,
	types: Types,
}

impl DataFlowBuilder {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			function_builder: FunctionBuilder::new(),
			global_state: GlobalState::new(),
			types: Types::new(),
		}
	}

	fn handle_type_section(&mut self, section: SectionLimited<RecGroup>) {
		self.types.add_sub_types(section);
	}

	fn handle_import_section(
		&mut self,
		graph: &mut DataFlowGraph,
		omega_in: u32,
		section: SectionLimited<wasmparser::Import>,
	) {
		let environment = Link(omega_in, OmegaIn::ENVIRONMENT_PORT);

		for wasmparser::Import { module, name, ty } in section.into_iter().map(Result::unwrap) {
			let mut link = graph.add_import(environment, module.into(), name.into());

			if let wasmparser::TypeRef::Func(function) = ty {
				self.types.add_function(function);

				link = graph.add_global_new(link);
			}

			self.global_state.get_mut_type_ref(ty).push(link);
		}
	}

	fn handle_function_section(&mut self, graph: &mut DataFlowGraph, section: SectionLimited<u32>) {
		let len = section.count().try_into().unwrap();

		self.types.add_functions(section);

		self.global_state.functions.extend(
			core::iter::repeat_with(|| {
				let null = graph.add_null();

				graph.add_global_new(null)
			})
			.take(len),
		);
	}

	fn build_expression(
		&mut self,
		graph: &mut DataFlowGraph,
		code: &ConstExpr,
		result: ValType,
	) -> Link {
		let code = code.get_operators_reader();

		self.function_builder
			.build_expression(graph, code, result, &self.types, &self.global_state)
	}

	fn load_table_node(graph: &mut DataFlowGraph, table_type: wasmparser::TableType) -> Link {
		let initializer = graph.add_null();
		let minimum = table_type.initial.try_into().unwrap();
		let maximum = table_type
			.maximum
			.map_or(u32::MAX, |maximum| maximum.try_into().unwrap());

		graph.add_table_new(initializer, minimum, maximum)
	}

	fn handle_table_declaration(
		&mut self,
		graph: &mut DataFlowGraph,
		section: SectionLimited<wasmparser::Table>,
	) {
		self.global_state.tables.extend(
			section
				.into_iter()
				.map(Result::unwrap)
				.map(|wasmparser::Table { ty, .. }| Self::load_table_node(graph, ty)),
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
			offset: graph.add_i32(0),
		};

		let source = self.build_expression(graph, code, ValType::Ref(ty.element_type));
		let size = graph.add_i32(ty.initial.try_into().unwrap());

		graph.add_table_fill(destination, source, size)
	}

	fn do_table_fill(
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

	fn handle_table_initialization(
		&mut self,
		graph: &mut DataFlowGraph,
		section: SectionLimited<wasmparser::Table>,
	) {
		let start = self.global_state.tables.len() - usize::try_from(section.count()).unwrap();

		for (offset, table) in section.into_iter().map(Result::unwrap).enumerate() {
			self.do_table_fill(graph, start + offset, &table);
		}
	}

	fn load_elements_functions(
		&self,
		graph: &mut DataFlowGraph,
		section: SectionLimited<u32>,
	) -> Vec<Link> {
		let functions = &self.global_state.functions;

		section
			.into_iter()
			.map(Result::unwrap)
			.map(|index| functions[usize::try_from(index).unwrap()])
			.map(|link| graph.add_global_get(link))
			.collect()
	}

	fn load_elements_expressions(
		&mut self,
		graph: &mut DataFlowGraph,
		section: SectionLimited<ConstExpr>,
		r#type: wasmparser::RefType,
	) -> Vec<Link> {
		section
			.into_iter()
			.map(Result::unwrap)
			.map(|initializer| self.build_expression(graph, &initializer, ValType::Ref(r#type)))
			.collect()
	}

	fn load_elements_node(
		&mut self,
		graph: &mut DataFlowGraph,
		items: ElementItems,
	) -> (Link, i32) {
		let content = match items {
			ElementItems::Functions(section) => self.load_elements_functions(graph, section),
			ElementItems::Expressions(r#type, section) => {
				self.load_elements_expressions(graph, section, r#type)
			}
		};

		let size = u32::try_from(content.len()).unwrap();
		let size = i32::from_ne_bytes(size.to_ne_bytes());

		(graph.add_elements_new(content), size)
	}

	fn load_table_init(
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
			offset: graph.add_i32(0),
		};

		let size = graph.add_i32(size);

		graph.add_table_init(destination, source, size)
	}

	fn handle_element_kind(
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
					self.load_table_init(graph, reference, offset_expr, elements, size);

				graph.add_elements_drop(elements)
			}
			wasmparser::ElementKind::Passive => elements,
			wasmparser::ElementKind::Declared => graph.add_elements_drop(elements),
		}
	}

	fn handle_element_declaration(
		&mut self,
		graph: &mut DataFlowGraph,
		section: &SectionLimited<wasmparser::Element>,
	) {
		let len = section.count().try_into().unwrap();

		self.global_state.elements.extend(
			core::iter::repeat_with(|| {
				let null = graph.add_null();

				graph.add_global_new(null)
			})
			.take(len),
		);
	}

	fn handle_element_initialization(
		&mut self,
		graph: &mut DataFlowGraph,
		section: SectionLimited<wasmparser::Element>,
	) {
		for (index, element) in section.into_iter().map(Result::unwrap).enumerate() {
			let (link, size) = self.load_elements_node(graph, element.items);
			let link = self.handle_element_kind(graph, link, size, &element.kind);

			self.global_state.elements[index] =
				graph.add_global_set(self.global_state.elements[index], link);
		}
	}

	fn load_memory_node(graph: &mut DataFlowGraph, memory_type: wasmparser::MemoryType) -> Link {
		let minimum = memory_type.initial.try_into().unwrap();
		let maximum = memory_type
			.maximum
			.map_or(u32::MAX, |maximum| maximum.try_into().unwrap());

		graph.add_memory_new(minimum, maximum)
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
				.map(|memory_type| Self::load_memory_node(graph, memory_type)),
		);
	}

	fn load_memory_init(
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
			offset: graph.add_i32(0),
		};

		let size = graph.add_i32(size);

		graph.add_memory_init(destination, source, size)
	}

	fn handle_data_kind(
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
					self.load_memory_init(graph, reference, offset_expr, data, size);

				graph.add_data_drop(data)
			}
		}
	}

	fn handle_data_declaration(
		&mut self,
		graph: &mut DataFlowGraph,
		section: SectionLimited<wasmparser::Data>,
	) {
		self.global_state.datas.extend(
			section
				.into_iter()
				.map(Result::unwrap)
				.map(|wasmparser::Data { data, .. }| graph.add_data_new(data.into())),
		);
	}

	fn handle_data_initialization(
		&mut self,
		graph: &mut DataFlowGraph,
		section: SectionLimited<wasmparser::Data>,
	) {
		for (index, data) in section.into_iter().map(Result::unwrap).enumerate() {
			let link = self.global_state.datas[index];
			let size = u32::try_from(data.data.len()).unwrap();
			let size = i32::from_ne_bytes(size.to_ne_bytes());

			self.global_state.datas[index] = self.handle_data_kind(graph, link, size, &data.kind);
		}
	}

	fn handle_global_declaration(
		&mut self,
		graph: &mut DataFlowGraph,
		section: &SectionLimited<wasmparser::Global>,
	) {
		let len = section.count().try_into().unwrap();

		self.global_state.globals.extend(
			core::iter::repeat_with(|| {
				let null = graph.add_null();

				graph.add_global_new(null)
			})
			.take(len),
		);
	}

	fn do_global_set(
		&mut self,
		graph: &mut DataFlowGraph,
		index: usize,
		global: &wasmparser::Global,
	) {
		let source = self.build_expression(graph, &global.init_expr, global.ty.content_type);

		self.global_state.globals[index] =
			graph.add_global_set(self.global_state.globals[index], source);
	}

	fn handle_global_initialization(
		&mut self,
		graph: &mut DataFlowGraph,
		section: SectionLimited<wasmparser::Global>,
	) {
		let start = self.global_state.globals.len() - usize::try_from(section.count()).unwrap();

		for (offset, global) in section.into_iter().map(Result::unwrap).enumerate() {
			self.do_global_set(graph, start + offset, &global);
		}
	}

	fn handle_tag_section(
		&mut self,
		_graph: &mut DataFlowGraph,
		_section: SectionLimited<wasmparser::TagType>,
	) {
	}

	fn build_function(
		&mut self,
		graph: &mut DataFlowGraph,
		body: &FunctionBody,
		index: usize,
	) -> u32 {
		self.function_builder.build_function(
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

			functions[imports] = graph.add_global_set(functions[imports], Link(lambda_out, 0));

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
			reference = graph.add_global_get(reference);
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
			let function = graph.add_global_get(function);
			let call = graph.add_call(function, alloc::vec![state], 0, 1);

			Link(call, 0)
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

		let start = graph.add_merge(states);

		graph.add_omega_out(omega_in, start, exports)
	}

	pub fn run(&mut self, graph: &mut DataFlowGraph, data: &[u8]) -> u32 {
		let sections = Sections::load(data);

		graph.inner_mut().clear();
		self.global_state.clear();
		self.types.clear();

		self.handle_type_section(sections.types);

		let omega_in = graph.add_omega_in();

		self.handle_import_section(graph, omega_in, sections.imports);

		let function_imports = self.global_state.functions.len();

		self.handle_table_declaration(graph, sections.tables.clone());
		self.handle_element_declaration(graph, &sections.elements);
		self.handle_data_declaration(graph, sections.datas.clone());
		self.handle_global_declaration(graph, &sections.globals);

		self.handle_function_section(graph, sections.functions);
		self.handle_memory_section(graph, sections.memories);
		self.handle_tag_section(graph, sections.tags);
		self.handle_code_section(graph, &sections.code, function_imports);

		self.handle_table_initialization(graph, sections.tables);
		self.handle_element_initialization(graph, sections.elements);
		self.handle_data_initialization(graph, sections.datas);
		self.handle_global_initialization(graph, sections.globals);

		let start = self.handle_start_section(graph, omega_in, sections.start);
		let exports = self.handle_export_section(graph, sections.exports);

		self.handle_module(graph, omega_in, start, exports)
	}
}

impl Default for DataFlowBuilder {
	fn default() -> Self {
		Self::new()
	}
}
