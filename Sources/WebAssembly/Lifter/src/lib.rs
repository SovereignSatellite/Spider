//! Lifts WebAssembly binaries into the IR data flow graph.

extern crate alloc;

use alloc::sync::Arc;
use core::iter;

use list::resizable::Resizable;
use parking_lot::Mutex;
use wasmparser::FunctionBody;

use ir_graph::{
	Link, Node,
	operation::{
		Apply, Export, Extract, Fence, Import, Location, MemoryNew, MutableNew, MutableSet,
		TableNew, TableSet,
	},
	region::Function,
};
use web_assembly_builder::Types;

use self::{
	entities::Entities,
	environment::{
		ElementItemPlan, ElementKindPlan, ElementPlan, EnvironmentPlan, ExportKind, ExportPlan,
		ImportKind, ImportPlan,
	},
	function::FunctionLifter,
	graph_builder::GraphBuilder,
	module::Module,
};

mod constant_expression;
mod dependencies;
mod entities;
mod environment;
mod function;
mod graph_builder;
mod interval;
mod module;
mod slots;
mod synthesis;

fn add_mutable_from_null(nodes: &mut Vec<Node>) -> Link {
	let null = Node::add_null_into(nodes);

	MutableNew::add_into(nodes, null)
}

impl ElementItemPlan {
	fn emit_into(self, nodes: &mut Vec<Node>, entities: &Entities) -> Link {
		match self {
			Self::Function(function) => entities.emit_function_reference(nodes, function),
			Self::Expression(expression) => expression.emit_into(nodes, entities),
		}
	}
}

/// Lifts WebAssembly binary data into an IR data flow graph.
pub struct WebAssemblyLifter {
	function_lifter: FunctionLifter,
	entities: Entities,
	graph_builder: GraphBuilder,
	types: Types,
}

impl WebAssemblyLifter {
	/// Creates a new WebAssembly lifter.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			function_lifter: FunctionLifter::new(),
			entities: Entities::new(),
			graph_builder: GraphBuilder::new(),
			types: Types::new(),
		}
	}

	fn load_types(&mut self, module: &Module<'_>) {
		self.types.clear();

		if let Some(types) = module.types.clone() {
			self.types.add_sub_types(types);
		}

		for import in &module.environment.imports {
			if let ImportKind::Function { type_index } = import.kind {
				self.types.add_function(type_index);
			}
		}

		if let Some(functions) = module.functions.clone() {
			self.types.add_functions(functions);
		}
	}

	fn create_import(&mut self, nodes: &mut Vec<Node>, import: &ImportPlan) {
		let namespace = Arc::clone(&import.namespace);
		let identifier = Arc::clone(&import.identifier);
		let value = Import::add_into(nodes, namespace, identifier);

		match import.kind {
			ImportKind::Function { .. } => {
				let slot = MutableNew::add_into(nodes, value);

				self.entities.functions.push(slot);
			}
			ImportKind::Table => self.entities.tables.push(value),
			ImportKind::Memory => self.entities.memories.push(value),
			ImportKind::Global => self.entities.globals.push(value),
		}
	}

	fn populate_element(&self, nodes: &mut Vec<Node>, element: &ElementPlan, table: Link) -> Link {
		let mut link = table;

		for (&item, offset) in element.items.iter().zip(0_i32..) {
			let source = item.emit_into(nodes, &self.entities);
			let destination = Location {
				reference: link,
				offset: Node::add_i32_into(nodes, offset),
			};

			link = TableSet::add_into(nodes, destination, source);
		}

		link
	}

	fn create_element(nodes: &mut Vec<Node>, element: &ElementPlan) -> Link {
		let Ok(count) = u32::try_from(element.items.len()) else {
			unreachable!()
		};

		TableNew::add_into(nodes, Vec::new(), count, count)
	}

	// Population happens after function installation so that function
	// reference items read the slots through their post-installation states.
	// Declared segments are dropped without ever being readable, so only
	// active and passive segments receive their items.
	fn populate_elements(&mut self, nodes: &mut Vec<Node>, environment: &EnvironmentPlan) {
		for (index, element) in environment.elements.iter().enumerate() {
			if matches!(element.kind, ElementKindPlan::Declared) {
				continue;
			}

			let table = self.entities.elements[index];
			let link = self.populate_element(nodes, element, table);

			self.entities.elements[index] = link;
		}
	}

	fn create_entities(
		&mut self,
		nodes: &mut Vec<Node>,
		environment: &EnvironmentPlan,
		declared_function_count: usize,
	) {
		for import in &environment.imports {
			self.create_import(nodes, import);
		}

		self.entities.functions.extend(
			iter::repeat_with(|| add_mutable_from_null(nodes)).take(declared_function_count),
		);

		self.entities.globals.extend(
			iter::repeat_with(|| add_mutable_from_null(nodes)).take(environment.globals.len()),
		);

		self.entities.tables.extend(
			environment
				.tables
				.iter()
				.map(|table| TableNew::add_into(nodes, Vec::new(), table.minimum, table.maximum)),
		);

		self.entities.memories.extend(
			environment.memories.iter().map(|memory| {
				MemoryNew::add_into(nodes, Vec::new(), memory.minimum, memory.maximum)
			}),
		);

		self.entities
			.datas
			.extend(environment.datas.iter().map(|data| {
				let bytes = Arc::clone(&data.bytes);
				let Ok(size) = u32::try_from(data.bytes.len()) else {
					unreachable!()
				};

				MemoryNew::add_into(nodes, vec![(bytes, 0)], size, size)
			}));

		self.entities.elements.extend(
			environment
				.elements
				.iter()
				.map(|element| Self::create_element(nodes, element)),
		);
	}

	fn install_functions(&mut self, nodes: &mut Vec<Node>, code: &[FunctionBody<'_>]) {
		let import_count = self.entities.functions.len() - code.len();

		for (offset, body) in code.iter().enumerate() {
			let overall_index = import_count + offset;
			let Ok(function_index) = u32::try_from(overall_index) else {
				unreachable!()
			};
			let function = self.function_lifter.build_function(
				nodes,
				body,
				function_index,
				&self.types,
				&self.entities,
			);
			let slot = self.entities.functions[overall_index];

			self.entities.functions[overall_index] = MutableSet::add_into(nodes, slot, function);
		}
	}

	fn lower_initialization(
		&mut self,
		nodes: &mut Vec<Node>,
		environment: &EnvironmentPlan,
	) -> Link {
		self.graph_builder.clear();

		synthesis::synthesize_initialization(&mut self.graph_builder, environment);

		self.function_lifter
			.build_synthesized(nodes, self.graph_builder.finish(), &self.entities)
	}

	// Every entity state feeds the fence so the compactor keeps root-level
	// writes whose only readers run later, inside called function bodies.
	fn create_fence(&self, nodes: &mut Vec<Node>, state: Link) -> Link {
		let mut states = vec![state];

		self.entities.collect_states_into(&mut states);

		let fence = Fence::add_into(nodes, Resizable::Heap(states));

		Link(fence, 0)
	}

	fn resolve_export(&self, nodes: &mut Vec<Node>, export: &ExportPlan) -> Link {
		let Ok(index) = usize::try_from(export.index) else {
			unreachable!()
		};

		match export.kind {
			ExportKind::Function => self.entities.emit_function_reference(nodes, export.index),
			ExportKind::Table => self.entities.tables[index],
			ExportKind::Memory => self.entities.memories[index],
			ExportKind::Global => self.entities.globals[index],
		}
	}

	fn emit_exports(&self, nodes: &mut Vec<Node>, environment: &EnvironmentPlan) -> Vec<Link> {
		environment
			.exports
			.iter()
			.map(|export| {
				let value = self.resolve_export(nodes, export);

				Export::add_into(nodes, Arc::clone(&export.identifier), value)
			})
			.collect()
	}

	/// Lifts the given WebAssembly binary data into a root function.
	pub fn run(&mut self, data: &[u8]) -> Arc<Mutex<Function>> {
		let module = Module::load(data);
		let environment = &module.environment;

		self.entities.clear();
		self.load_types(&module);

		Function::create(1, |nodes, arguments| {
			self.create_entities(nodes, environment, module.code.len());
			self.install_functions(nodes, &module.code);
			self.populate_elements(nodes, environment);

			let initialization = self.lower_initialization(nodes, environment);
			let trap = self.create_fence(nodes, Link(arguments, 0));
			let function = Extract::add_into(nodes, initialization, 0);
			let apply = Apply::add_into(nodes, function, vec![initialization, trap], 1);

			let mut states = self.emit_exports(nodes, environment);

			states.push(Link(apply, 0));

			// A single fenced result keeps every export observed without
			// returning one value per export, which targets cap.
			let fence = Fence::add_into(nodes, Resizable::Heap(states));

			vec![Link(fence, 0)]
		})
	}
}

impl Default for WebAssemblyLifter {
	fn default() -> Self {
		Self::new()
	}
}
