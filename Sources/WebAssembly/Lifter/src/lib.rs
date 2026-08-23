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
		Apply, Export, Extract, Fence, Import, Location, MemoryNew, MutableGet, MutableNew,
		MutableSet, TableNew, TableSet,
	},
	region::Function,
};

use self::{
	function::FunctionLifter,
	module::{
		ConstantExpression, ElementItemPlan, ElementKindPlan, ElementPlan, ExportKind, ExportPlan,
		ImportKind, ImportPlan, ModuleBindings, ModuleDefinition, ModulePlan, TypeRegistry,
		build_initialization,
	},
};

mod function;
mod memory;
mod module;

fn add_mutable_from_null(nodes: &mut Vec<Node>) -> Link {
	let null = Node::add_null_into(nodes);

	MutableNew::add_into(nodes, null)
}

fn create_data_segment(nodes: &mut Vec<Node>, initializer: Arc<[u8]>) -> Link {
	let size = Node::add_i32_into(nodes, initializer.len().try_into().unwrap());
	let content = MemoryNew::add_into(nodes, vec![(initializer, 0)], size);

	MutableNew::add_into(nodes, content)
}

fn emit_constant_expression_to_root(
	nodes: &mut Vec<Node>,
	bindings: &ModuleBindings,
	expression: ConstantExpression,
) -> Link {
	match expression {
		ConstantExpression::I32(value) => Node::add_i32_into(nodes, value),
		ConstantExpression::I64(value) => Node::add_i64_into(nodes, value),
		ConstantExpression::F32(value) => Node::add_f32_into(nodes, value),
		ConstantExpression::F64(value) => Node::add_f64_into(nodes, value),
		ConstantExpression::RefNull => Node::add_null_into(nodes),
		ConstantExpression::RefFunction(function) => {
			bindings.emit_function_reference(nodes, function)
		}
		ConstantExpression::GlobalGet(global) => {
			let Ok(index) = usize::try_from(global) else {
				unreachable!()
			};

			MutableGet::add_into(nodes, bindings.globals[index]).0
		}
	}
}

impl ElementItemPlan {
	fn emit_to_root(self, nodes: &mut Vec<Node>, bindings: &ModuleBindings) -> Link {
		match self {
			Self::Function(function) => bindings.emit_function_reference(nodes, function),
			Self::Expression(expression) => {
				emit_constant_expression_to_root(nodes, bindings, expression)
			}
		}
	}
}

/// Lifts WebAssembly binary data into an IR data flow graph.
pub struct WebAssemblyLifter {
	function_lifter: FunctionLifter,
	bindings: ModuleBindings,
	types: TypeRegistry,
}

impl WebAssemblyLifter {
	/// Creates a new WebAssembly lifter.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			function_lifter: FunctionLifter::new(),
			bindings: ModuleBindings::new(),
			types: TypeRegistry::new(),
		}
	}

	fn create_import(&mut self, nodes: &mut Vec<Node>, import: &ImportPlan) {
		let namespace = Arc::clone(&import.namespace);
		let identifier = Arc::clone(&import.identifier);
		let value = Import::add_into(nodes, namespace, identifier);

		match import.kind {
			ImportKind::Function => {
				let slot = MutableNew::add_into(nodes, value);

				self.bindings.functions.push(slot);
			}
			ImportKind::Table => self.bindings.tables.push(value),
			ImportKind::Memory => self.bindings.memories.push(value),
			ImportKind::Global => self.bindings.globals.push(value),
		}
	}

	fn populate_element_segment(
		&self,
		nodes: &mut Vec<Node>,
		element: &ElementPlan,
		table: Link,
	) -> Link {
		let mut link = table;

		for (&item, offset) in element.items.iter().zip(0_i32..) {
			let source = item.emit_to_root(nodes, &self.bindings);
			let destination = Location {
				reference: link,
				offset: Node::add_i32_into(nodes, offset),
			};

			link = TableSet::add_into(nodes, destination, source);
		}

		link
	}

	fn create_element_segment(nodes: &mut Vec<Node>, element: &ElementPlan) -> Link {
		let Ok(count) = u32::try_from(element.items.len()) else {
			unreachable!()
		};

		TableNew::add_into(nodes, Vec::new(), count, count)
	}

	// Population happens after function installation so that function
	// reference items read the slots through their post-installation states.
	// Declared segments are dropped without ever being readable, so only
	// active and passive segments receive their items.
	fn populate_element_segments(&mut self, nodes: &mut Vec<Node>, module: &ModulePlan) {
		for (index, element) in module.elements.iter().enumerate() {
			if matches!(element.kind, ElementKindPlan::Declared) {
				continue;
			}

			let table = self.bindings.elements[index];
			let link = self.populate_element_segment(nodes, element, table);

			self.bindings.elements[index] = link;
		}
	}

	fn create_bindings(
		&mut self,
		nodes: &mut Vec<Node>,
		module: &ModulePlan,
		declared_function_count: usize,
	) {
		for import in &module.imports {
			self.create_import(nodes, import);
		}

		self.bindings.functions.extend(
			iter::repeat_with(|| add_mutable_from_null(nodes)).take(declared_function_count),
		);

		self.bindings
			.globals
			.extend(iter::repeat_with(|| add_mutable_from_null(nodes)).take(module.globals.len()));

		self.bindings.tables.extend(
			module
				.tables
				.iter()
				.map(|table| TableNew::add_into(nodes, Vec::new(), table.minimum, table.maximum)),
		);

		self.bindings.memories.extend(
			module
				.memories
				.iter()
				.map(|plan| memory::create(nodes, Vec::new(), plan.minimum, plan.maximum)),
		);

		self.bindings.datas.extend(
			module
				.datas
				.iter()
				.map(|data| create_data_segment(nodes, Arc::clone(&data.bytes))),
		);

		self.bindings.elements.extend(
			module
				.elements
				.iter()
				.map(|element| Self::create_element_segment(nodes, element)),
		);
	}

	fn install_functions(&mut self, nodes: &mut Vec<Node>, code: &[FunctionBody<'_>]) {
		let import_count = self.bindings.functions.len() - code.len();

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
				&self.bindings,
			);
			let slot = self.bindings.functions[overall_index];

			self.bindings.functions[overall_index] = MutableSet::add_into(nodes, slot, function);
		}
	}

	fn lower_initialization(&mut self, nodes: &mut Vec<Node>, plan: &ModulePlan) -> Link {
		let graph = self.function_lifter.take_graph();
		let graph = build_initialization(graph, plan);

		self.function_lifter
			.build_synthesized(nodes, graph, &self.bindings)
	}

	fn fence_root_states(&self, nodes: &mut Vec<Node>, state: Link) -> Link {
		// Every entity state feeds the fence so the compactor keeps root-level
		// writes whose only readers run later, inside called function bodies.
		let mut states = vec![state];

		self.bindings.collect_states_into(&mut states);

		let fence = Fence::add_into(nodes, Resizable::Heap(states));

		Link(fence, 0)
	}

	fn resolve_export(&self, nodes: &mut Vec<Node>, export: &ExportPlan) -> Link {
		let Ok(index) = usize::try_from(export.index) else {
			unreachable!()
		};

		match export.kind {
			ExportKind::Function => self.bindings.emit_function_reference(nodes, export.index),
			ExportKind::Table => self.bindings.tables[index],
			ExportKind::Memory => self.bindings.memories[index],
			ExportKind::Global => self.bindings.globals[index],
		}
	}

	fn emit_exports(&self, nodes: &mut Vec<Node>, plan: &ModulePlan) -> Vec<Link> {
		plan.exports
			.iter()
			.map(|export| {
				let value = self.resolve_export(nodes, export);

				Export::add_into(nodes, Arc::clone(&export.identifier), value)
			})
			.collect()
	}

	/// Lifts the given WebAssembly binary data into a root function.
	#[must_use = "use the lifted root function"]
	pub fn run(&mut self, binary: &[u8]) -> Arc<Mutex<Function>> {
		let module = ModuleDefinition::parse(binary, &mut self.types);
		let plan = &module.plan;

		self.bindings.clear();

		Function::create(1, |nodes, arguments| {
			self.create_bindings(nodes, plan, module.code.len());
			self.install_functions(nodes, &module.code);
			self.populate_element_segments(nodes, plan);

			let initialization = self.lower_initialization(nodes, plan);
			let trap = self.fence_root_states(nodes, Link(arguments, 0));
			let function = Extract::add_into(nodes, initialization, 0);
			let apply = Apply::add_into(nodes, function, vec![initialization, trap], 1);

			let mut states = self.emit_exports(nodes, plan);

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
