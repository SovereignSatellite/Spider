//! Lifts WebAssembly binaries into the IR data flow graph.

extern crate alloc;

use alloc::sync::Arc;

use list::resizable::Resizable;
use parking_lot::Mutex;
use wasmparser::{ConstExpr, ElementItems, FunctionBody, SectionLimited, ValType};

use ir_graph::{
	Link, Node,
	control::{Export, Import, Module, ModuleArguments},
	simple::{
		Apply, Fence, GlobalGet, GlobalNew, GlobalSet, Location, MemoryCopy, MemoryDrop, MemoryNew,
		TableCopy, TableDrop, TableFill, TableNew, TableSet,
	},
};
use web_assembly_builder::Types;
use web_assembly_graph::instruction::MemorySize;

use self::{function_lifter::FunctionLifter, global_state::GlobalState, sections::Sections};

mod control_flow_lifter;
mod function_lifter;
mod global_state;
mod sections;

fn get_element_count(items: &ElementItems<'_>) -> u32 {
	match items {
		ElementItems::Functions(section) => section.count(),
		ElementItems::Expressions(_, section) => section.count(),
	}
}

fn add_table_from_type(nodes: &mut Vec<Node>, table_type: wasmparser::TableType) -> Link {
	let wasmparser::TableType {
		initial, maximum, ..
	} = table_type;

	let minimum = initial.try_into().unwrap();
	let maximum = maximum.map_or(u32::MAX, |maximum| maximum.try_into().unwrap());

	TableNew::add_into(nodes, Vec::new(), minimum, maximum)
}

fn add_table_from_items(nodes: &mut Vec<Node>, items: &ElementItems<'_>) -> Link {
	let count = get_element_count(items);

	TableNew::add_into(nodes, Vec::new(), count, count)
}

fn add_memory_from_type(nodes: &mut Vec<Node>, memory_type: wasmparser::MemoryType) -> Link {
	let wasmparser::MemoryType {
		initial, maximum, ..
	} = memory_type;

	let page = u32::try_from(MemorySize::PAGE_SIZE).unwrap();

	let minimum = u32::try_from(initial).unwrap().saturating_mul(page);
	let maximum = maximum.map_or(u32::MAX, |maximum| {
		u32::try_from(maximum).unwrap().saturating_mul(page)
	});

	MemoryNew::add_into(nodes, Vec::new(), minimum, maximum)
}

fn add_memory_from_data(nodes: &mut Vec<Node>, data: &[u8]) -> Link {
	let data = Arc::<[u8]>::from(data);
	let len = data.len().try_into().unwrap();

	MemoryNew::add_into(nodes, vec![(data, 0)], len, len)
}

fn add_global_from_null(nodes: &mut Vec<Node>) -> Link {
	let null = Node::add_null_into(nodes);

	GlobalNew::add_into(nodes, null)
}

/// Lifts WebAssembly binary data into an IR data flow graph.
pub struct WebAssemblyLifter {
	function_lifter: FunctionLifter,
	global_state: GlobalState,
	types: Types,
}

impl WebAssemblyLifter {
	/// Creates a new WebAssembly lifter.
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
		nodes: &mut Vec<Node>,
		arguments: u32,
		section: SectionLimited<'_, wasmparser::Import<'_>>,
	) {
		let environment = Link(arguments, ModuleArguments::ENVIRONMENT_PORT);

		for wasmparser::Import { module, name, ty } in section.into_iter().map(Result::unwrap) {
			let mut link = Import::add_into(nodes, environment, module.into(), name.into());

			if let wasmparser::TypeRef::Func(function) = ty {
				self.types.add_function(function);

				link = GlobalNew::add_into(nodes, link);
			}

			self.global_state.get_mut_type_ref(ty).push(link);
		}
	}

	fn handle_function_section(&mut self, nodes: &mut Vec<Node>, section: SectionLimited<'_, u32>) {
		let len = section.count().try_into().unwrap();

		self.types.add_functions(section);

		self.global_state
			.functions
			.extend(core::iter::repeat_with(|| add_global_from_null(nodes)).take(len));
	}

	fn build_expression(
		&mut self,
		nodes: &mut Vec<Node>,
		code: &ConstExpr<'_>,
		result: ValType,
	) -> Link {
		let code = code.get_operators_reader();

		self.function_lifter
			.build_expression(nodes, code, result, &self.types, &self.global_state)
	}

	fn handle_table_declarations(
		&mut self,
		nodes: &mut Vec<Node>,
		section: SectionLimited<'_, wasmparser::Table<'_>>,
	) {
		self.global_state.tables.extend(
			section
				.into_iter()
				.map(Result::unwrap)
				.map(|wasmparser::Table { ty, .. }| add_table_from_type(nodes, ty)),
		);
	}

	fn load_table_fill(
		&mut self,
		nodes: &mut Vec<Node>,
		reference: Link,
		code: &ConstExpr<'_>,
		ty: wasmparser::TableType,
	) -> Link {
		let destination = Location {
			reference,
			offset: Node::add_i32_into(nodes, 0),
		};

		let source = self.build_expression(nodes, code, ValType::Ref(ty.element_type));
		let size = Node::add_i32_into(nodes, ty.initial.try_into().unwrap());

		TableFill::add_into(nodes, destination, source, size)
	}

	fn initialize_table(
		&mut self,
		nodes: &mut Vec<Node>,
		index: usize,
		table: &wasmparser::Table<'_>,
	) {
		let wasmparser::TableInit::Expr(code) = &table.init else {
			return;
		};

		let destination = self.global_state.tables[index];

		self.global_state.tables[index] = self.load_table_fill(nodes, destination, code, table.ty);
	}

	fn handle_table_initializations(
		&mut self,
		nodes: &mut Vec<Node>,
		section: SectionLimited<'_, wasmparser::Table<'_>>,
	) {
		let start = self.global_state.tables.len() - usize::try_from(section.count()).unwrap();

		for (offset, table) in section.into_iter().map(Result::unwrap).enumerate() {
			self.initialize_table(nodes, start + offset, &table);
		}
	}

	fn handle_element_declarations(
		&mut self,
		nodes: &mut Vec<Node>,
		section: SectionLimited<'_, wasmparser::Element<'_>>,
	) {
		self.global_state.elements.extend(
			section
				.into_iter()
				.map(Result::unwrap)
				.map(|wasmparser::Element { items, .. }| add_table_from_items(nodes, &items)),
		);
	}

	fn set_table_functions(
		&self,
		nodes: &mut Vec<Node>,
		section: SectionLimited<'_, u32>,
		mut element: Link,
	) -> Link {
		let functions = &self.global_state.functions;

		for (function, offset) in section.into_iter().map(Result::unwrap).zip(0_i32..) {
			let function = functions[usize::try_from(function).unwrap()];
			let source = GlobalGet::add_into(nodes, function).0;
			let destination = Location {
				reference: element,
				offset: Node::add_i32_into(nodes, offset),
			};

			element = TableSet::add_into(nodes, destination, source);
		}

		element
	}

	fn set_table_expressions(
		&mut self,
		nodes: &mut Vec<Node>,
		section: SectionLimited<'_, ConstExpr<'_>>,
		kind: wasmparser::RefType,
		mut element: Link,
	) -> Link {
		for (code, offset) in section.into_iter().map(Result::unwrap).zip(0_i32..) {
			let source = self.build_expression(nodes, &code, ValType::Ref(kind));
			let destination = Location {
				reference: element,
				offset: Node::add_i32_into(nodes, offset),
			};

			element = TableSet::add_into(nodes, destination, source);
		}

		element
	}

	fn initialize_element(
		&mut self,
		nodes: &mut Vec<Node>,
		items: ElementItems<'_>,
		element: Link,
	) -> Link {
		match items {
			ElementItems::Functions(section) => self.set_table_functions(nodes, section, element),
			ElementItems::Expressions(kind, section) => {
				self.set_table_expressions(nodes, section, kind, element)
			}
		}
	}

	fn load_table_copy(
		&mut self,
		nodes: &mut Vec<Node>,
		reference: Link,
		offset: &ConstExpr<'_>,
		elements: Link,
		size: i32,
	) -> Link {
		let destination = Location {
			reference,
			offset: self.build_expression(nodes, offset, ValType::I32),
		};

		let source = Location {
			reference: elements,
			offset: Node::add_i32_into(nodes, 0),
		};

		let size = Node::add_i32_into(nodes, size);

		TableCopy::add_into(nodes, destination, source, size).0
	}

	fn action_element(
		&mut self,
		nodes: &mut Vec<Node>,
		elements: Link,
		size: i32,
		element_kind: &wasmparser::ElementKind<'_>,
	) -> Link {
		match element_kind {
			wasmparser::ElementKind::Active {
				table_index,
				offset_expr,
			} => {
				let index: usize = table_index.unwrap_or(0).try_into().unwrap();
				let reference = self.global_state.tables[index];

				self.global_state.tables[index] =
					self.load_table_copy(nodes, reference, offset_expr, elements, size);

				TableDrop::add_into(nodes, elements)
			}
			wasmparser::ElementKind::Passive => elements,
			wasmparser::ElementKind::Declared => TableDrop::add_into(nodes, elements),
		}
	}

	fn handle_element_initializations(
		&mut self,
		nodes: &mut Vec<Node>,
		section: SectionLimited<'_, wasmparser::Element<'_>>,
	) {
		for (index, element) in section.into_iter().map(Result::unwrap).enumerate() {
			let size = get_element_count(&element.items);
			let size = i32::from_ne_bytes(size.to_ne_bytes());

			let link = self.global_state.elements[index];
			let link = self.initialize_element(nodes, element.items, link);
			let link = self.action_element(nodes, link, size, &element.kind);

			self.global_state.elements[index] = link;
		}
	}

	fn handle_memory_section(
		&mut self,
		nodes: &mut Vec<Node>,
		section: SectionLimited<'_, wasmparser::MemoryType>,
	) {
		self.global_state.memories.extend(
			section
				.into_iter()
				.map(Result::unwrap)
				.map(|memory_type| add_memory_from_type(nodes, memory_type)),
		);
	}

	fn load_memory_copy(
		&mut self,
		nodes: &mut Vec<Node>,
		reference: Link,
		offset: &ConstExpr<'_>,
		data: Link,
		size: i32,
	) -> Link {
		let destination = Location {
			reference,
			offset: self.build_expression(nodes, offset, ValType::I32),
		};

		let source = Location {
			reference: data,
			offset: Node::add_i32_into(nodes, 0),
		};

		let size = Node::add_i32_into(nodes, size);

		MemoryCopy::add_into(nodes, destination, source, size).0
	}

	fn handle_data_declarations(
		&mut self,
		nodes: &mut Vec<Node>,
		section: SectionLimited<'_, wasmparser::Data<'_>>,
	) {
		self.global_state.datas.extend(
			section
				.into_iter()
				.map(Result::unwrap)
				.map(|wasmparser::Data { data, .. }| add_memory_from_data(nodes, data)),
		);
	}

	fn action_data(
		&mut self,
		nodes: &mut Vec<Node>,
		data: Link,
		size: i32,
		data_kind: &wasmparser::DataKind<'_>,
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
					self.load_memory_copy(nodes, reference, offset_expr, data, size);

				MemoryDrop::add_into(nodes, data)
			}
		}
	}

	fn handle_data_initializations(
		&mut self,
		nodes: &mut Vec<Node>,
		section: SectionLimited<'_, wasmparser::Data<'_>>,
	) {
		for (index, data) in section.into_iter().map(Result::unwrap).enumerate() {
			let size = u32::try_from(data.data.len()).unwrap();
			let size = i32::from_ne_bytes(size.to_ne_bytes());

			let link = self.global_state.datas[index];
			let link = self.action_data(nodes, link, size, &data.kind);

			self.global_state.datas[index] = link;
		}
	}

	fn handle_global_declarations(
		&mut self,
		nodes: &mut Vec<Node>,
		section: &SectionLimited<'_, wasmparser::Global<'_>>,
	) {
		let len = section.count().try_into().unwrap();

		self.global_state
			.globals
			.extend(core::iter::repeat_with(|| add_global_from_null(nodes)).take(len));
	}

	fn initialize_global(
		&mut self,
		nodes: &mut Vec<Node>,
		index: usize,
		global: &wasmparser::Global<'_>,
	) {
		let source = self.build_expression(nodes, &global.init_expr, global.ty.content_type);

		self.global_state.globals[index] =
			GlobalSet::add_into(nodes, self.global_state.globals[index], source);
	}

	fn handle_global_initializations(
		&mut self,
		nodes: &mut Vec<Node>,
		section: SectionLimited<'_, wasmparser::Global<'_>>,
	) {
		let start = self.global_state.globals.len() - usize::try_from(section.count()).unwrap();

		for (offset, global) in section.into_iter().map(Result::unwrap).enumerate() {
			self.initialize_global(nodes, start + offset, &global);
		}
	}

	#[expect(
		clippy::needless_pass_by_ref_mut,
		clippy::needless_pass_by_value,
		clippy::ptr_arg,
		unused_variables,
		reason = "tag section handler signature matches other section handlers"
	)]
	fn handle_tag_section(
		&mut self,
		nodes: &mut Vec<Node>,
		section: SectionLimited<'_, wasmparser::TagType>,
	) {
		if section.count() == 0 {
			return;
		}

		unimplemented!("`Tag`s are not supported yet")
	}

	fn build_function(
		&mut self,
		nodes: &mut Vec<Node>,
		body: &FunctionBody<'_>,
		index: usize,
	) -> Link {
		self.function_lifter.build_function(
			nodes,
			body,
			index.try_into().unwrap(),
			&self.types,
			&self.global_state,
		)
	}

	fn handle_code_section(
		&mut self,
		nodes: &mut Vec<Node>,
		section: &[FunctionBody<'_>],
		mut imports: usize,
	) {
		for body in section {
			let function = self.build_function(nodes, body, imports);
			let functions = &mut self.global_state.functions;

			functions[imports] = GlobalSet::add_into(nodes, functions[imports], function);

			imports += 1;
		}
	}

	fn load_export_information(
		&self,
		nodes: &mut Vec<Node>,
		export: wasmparser::Export<'_>,
	) -> Export {
		let index = usize::try_from(export.index).unwrap();
		let mut reference = self.global_state.get_external_kind(export.kind)[index];

		if export.kind == wasmparser::ExternalKind::Func {
			reference = GlobalGet::add_into(nodes, reference).0;
		}

		Export {
			identifier: export.name.into(),
			reference,
		}
	}

	fn handle_export_section(
		&self,
		nodes: &mut Vec<Node>,
		section: SectionLimited<'_, wasmparser::Export<'_>>,
	) -> Vec<Export> {
		section
			.into_iter()
			.map(Result::unwrap)
			.map(|export| self.load_export_information(nodes, export))
			.collect()
	}

	fn create_fence(&self, nodes: &mut Vec<Node>, state: Link) -> Link {
		let mut states = vec![state];

		self.global_state.retrieve_all_mutable(&mut states);

		let fence = Fence::add_into(nodes, Resizable::Heap(states));

		Link(fence, 0)
	}

	fn handle_start_section(
		&self,
		nodes: &mut Vec<Node>,
		arguments: u32,
		start: Option<u32>,
	) -> Link {
		let state = Link(arguments, ModuleArguments::STATE_PORT);
		let state = self.create_fence(nodes, state);

		start.map_or(state, |start| {
			let function = self.global_state.functions[usize::try_from(start).unwrap()];
			let function = GlobalGet::add_into(nodes, function).0;
			let apply = Apply::add_into(nodes, function, vec![state], 1);

			Link(apply, 0)
		})
	}

	/// Lifts the given WebAssembly binary data into a module.
	pub fn run(&mut self, data: &[u8]) -> Arc<Mutex<Module>> {
		let sections = Sections::load(data);

		self.global_state.clear();
		self.types.clear();
		self.types.add_sub_types(sections.types);

		Module::create(|nodes, arguments| {
			self.handle_import_section(nodes, arguments, sections.imports);

			let function_imports = self.global_state.functions.len();

			self.handle_table_declarations(nodes, sections.tables.clone());
			self.handle_element_declarations(nodes, sections.elements.clone());
			self.handle_data_declarations(nodes, sections.datas.clone());
			self.handle_global_declarations(nodes, &sections.globals);

			self.handle_function_section(nodes, sections.functions);
			self.handle_memory_section(nodes, sections.memories);
			self.handle_tag_section(nodes, sections.tags);
			self.handle_code_section(nodes, &sections.code, function_imports);

			self.handle_table_initializations(nodes, sections.tables);
			self.handle_element_initializations(nodes, sections.elements);
			self.handle_data_initializations(nodes, sections.datas);
			self.handle_global_initializations(nodes, sections.globals);

			let start = self.handle_start_section(nodes, arguments, sections.start);
			let exports = self.handle_export_section(nodes, sections.exports);

			(start, exports)
		})
	}
}

impl Default for WebAssemblyLifter {
	fn default() -> Self {
		Self::new()
	}
}
