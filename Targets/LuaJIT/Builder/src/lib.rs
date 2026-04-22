//! Builds `LuaJIT` trees from IR data flow graphs.

extern crate alloc;

use alloc::sync::Arc;
use core::any::Any;

use parking_lot::Mutex;

use ir_graph::{
	Link, Node,
	foreign::Foreign,
	operation::{
		Aggregate, Apply, Extract, Fence, Identity, IntegerConvertToNumber, IntegerNarrow,
		IntegerSignExtend, IntegerTransmuteToNumber, IntegerWiden, MemoryCopy, MemoryDrop,
		MemoryFill, MemoryGrow, MemoryLoad, MemoryNew, MemorySize, MemoryStore, MutableGet,
		MutableNew, MutableSet, NumberNarrow, NumberTransmuteToInteger, NumberTruncateToInteger,
		NumberWiden, RefIsNull, TableCopy, TableDrop, TableFill, TableGet, TableGrow, TableNew,
		TableSet, TableSize, integer, number,
	},
	region::{Branch, Function, Match, Module, Repeat, repeat},
};
use luajit_tree::{
	LuaJITTree,
	expression::{self, Expression, Local, Name},
};
use turing_machine_source::foreign::{Ask as TuringAsk, Tell as TuringTell};
use web_assembly_lifter::foreign::{Export as WasmExport, Import as WasmImport};

use self::{code_handler::CodeHandler, data_handler::DataHandler, local_allocator::LocalAllocator};

mod assignment_simplifier;
mod code_handler;
mod data_handler;
mod local_allocator;

/// Builds a `LuaJIT` tree from an IR data flow graph.
pub struct LuaJITBuilder {
	local_allocator: LocalAllocator,

	code_handler: CodeHandler,
	data_handler: DataHandler,

	luajit_tree: Option<LuaJITTree>,
}

impl LuaJITBuilder {
	/// Creates a new `LuaJIT` builder.
	#[must_use]
	pub fn new() -> Self {
		Self {
			local_allocator: LocalAllocator::new(),

			code_handler: CodeHandler::new(),
			data_handler: DataHandler::new(),

			luajit_tree: None,
		}
	}

	fn do_assignment(&mut self, destination: u32, source: Expression) {
		if let Some(local) = self.data_handler.get_local(Link(destination, 0)) {
			self.code_handler.do_assign(local, source);
		} else {
			self.data_handler.store_expression(destination, source);
		}
	}

	fn bridge_argument_names(&mut self, names: &[Name]) {
		for (port, &name) in names.iter().enumerate() {
			let port = port.try_into().unwrap();
			let link = Link(Function::ARGUMENTS_ID, port);

			let Some(destination) = self.data_handler.get_local(link) else {
				continue;
			};

			self.code_handler
				.do_assign(destination, Expression::Local(Local::Fast { name }));
		}
	}

	fn load_function_returns(&mut self, results: &[Link], scope: usize) -> Vec<Expression> {
		results
			.iter()
			.map(|&link| {
				if let Some(local) = self.data_handler.get_scoped_local(link, scope) {
					Expression::Local(local)
				} else {
					self.data_handler
						.take_expression(link.0, scope)
						.unwrap_or(Expression::Null)
				}
			})
			.collect()
	}

	fn emit_function_body(
		&mut self,
		function: &Function,
		argument_names: Vec<Name>,
		function_scope: usize,
	) -> expression::Function {
		self.data_handler.set_scope(function_scope);
		self.code_handler.push_scope();

		self.bridge_argument_names(&argument_names);
		self.handle_nodes(&function.nodes, function_scope);

		let stack = self.data_handler.get_stack_size(function_scope);
		let code = self.code_handler.pop_scope();
		let returns = self.load_function_returns(&function.results().sources, function_scope);

		expression::Function {
			arguments: argument_names,
			locals: Vec::new(),
			stack,
			code,
			returns,
		}
	}

	fn handle_function(&mut self, id: u32, arc: &Arc<Mutex<Function>>) {
		let function = arc.lock();
		let function_scope = Arc::as_ptr(arc) as usize;
		let parent_scope = self.data_handler.scope();

		let argument_count = u32::from(function.argument_count);
		let argument_names: Vec<Name> = (0..argument_count)
			.map(|offset| Name { id: offset })
			.collect();

		let inner = self.emit_function_body(&function, argument_names, function_scope);

		drop(function);

		self.data_handler.set_scope(parent_scope);
		self.do_assignment(id, Expression::Function(inner.into()));
	}

	fn load_match_result_locals(&self, id: u32, scope: usize, result_count: u16) -> Vec<Local> {
		(0..result_count)
			.filter_map(|port| self.data_handler.get_scoped_local(Link(id, port), scope))
			.collect()
	}

	fn handle_branch(
		&mut self,
		branch_arc: &Arc<Mutex<Branch>>,
		matcher: &Match,
		parent_scope: usize,
		result_locals: &[Local],
	) -> usize {
		let branch = branch_arc.lock();
		let branch_scope = Arc::as_ptr(branch_arc) as usize;

		self.data_handler.set_scope(branch_scope);
		self.code_handler.push_scope();

		self.code_handler.do_bulk_assignment(
			0,
			&matcher.arguments,
			parent_scope,
			&self.data_handler,
		);

		self.handle_nodes(&branch.nodes, branch_scope);

		for (&result, &canonical) in branch.results().sources.iter().zip(result_locals) {
			let Some(actual) = self.data_handler.get_scoped_local(result, branch_scope) else {
				continue;
			};

			if actual != canonical {
				self.code_handler
					.do_assign(canonical, Expression::Local(actual));
			}
		}

		drop(branch);

		self.code_handler.pop_branch(branch_scope);

		branch_scope
	}

	fn handle_match(&mut self, id: u32, arc: &Arc<Mutex<Match>>) {
		let matcher = arc.lock();
		let parent_scope = self.data_handler.scope();

		let result_count = matcher.result_count();
		let result_locals = self.load_match_result_locals(id, parent_scope, result_count);

		let branch_keys: Vec<_> = matcher
			.branches
			.iter()
			.map(|branch_arc| {
				self.handle_branch(branch_arc, &matcher, parent_scope, &result_locals)
			})
			.collect();

		self.data_handler.set_scope(parent_scope);

		self.code_handler
			.do_match(&branch_keys, matcher.condition, &mut self.data_handler);
	}

	fn handle_repeat(&mut self, _id: u32, arc: &Arc<Mutex<Repeat>>) {
		let repeat = arc.lock();
		let repeat_scope = Arc::as_ptr(arc) as usize;
		let parent_scope = self.data_handler.scope();

		self.data_handler.set_scope(repeat_scope);
		self.code_handler.do_bulk_assignment(
			0,
			&repeat.arguments,
			parent_scope,
			&self.data_handler,
		);

		self.code_handler.push_scope();

		self.handle_nodes(&repeat.nodes, repeat_scope);

		drop(repeat);

		self.data_handler.set_scope(parent_scope);
	}

	fn handle_repeat_results(&mut self, node: &repeat::Results) {
		let scope = self.data_handler.scope();

		self.code_handler
			.do_bulk_assignment(0, &node.sources, scope, &self.data_handler);

		self.code_handler
			.do_repeat(node.condition, &mut self.data_handler);
	}

	fn handle_module(&mut self, module: &Module, scope: usize) {
		self.data_handler.set_scope(scope);
		self.code_handler.push_scope();

		self.handle_nodes(&module.nodes, scope);

		let stack = self.data_handler.get_stack_size(scope);
		let code = self.code_handler.pop_scope();

		self.luajit_tree = Some(LuaJITTree { stack, code });
	}

	fn handle_wasm_import(&mut self, id: u32, node: &WasmImport) {
		let expression = DataHandler::load_wasm_import(node);

		self.do_assignment(id, expression);
	}

	fn handle_wasm_export(&mut self, node: &WasmExport) {
		let value = self.data_handler.load(node.value);
		let identifier = Expression::String(Arc::clone(&node.identifier));

		self.code_handler
			.do_runtime_call("export", vec![identifier, value]);
	}

	fn handle_turing_ask(&mut self, id: u32, node: TuringAsk) {
		let expression = DataHandler::load_turing_ask();

		self.do_assignment(id, expression);

		self.code_handler.do_rename(
			Link(id, TuringAsk::STATE_PORT),
			node.state,
			&self.data_handler,
		);
	}

	fn handle_turing_tell(&mut self, id: u32, node: TuringTell) {
		let character = self.data_handler.load(node.character);

		self.code_handler
			.do_runtime_call("turing_tell", vec![character]);

		self.code_handler.do_rename(
			Link(id, TuringTell::STATE_PORT),
			node.state,
			&self.data_handler,
		);
	}

	fn handle_foreign(&mut self, id: u32, foreign: &dyn Foreign) {
		let any: &dyn Any = foreign;

		if let Some(node) = any.downcast_ref::<WasmImport>() {
			self.handle_wasm_import(id, node);
		} else if let Some(node) = any.downcast_ref::<WasmExport>() {
			self.handle_wasm_export(node);
		} else if let Some(&node) = any.downcast_ref::<TuringAsk>() {
			self.handle_turing_ask(id, node);
		} else if let Some(&node) = any.downcast_ref::<TuringTell>() {
			self.handle_turing_tell(id, node);
		} else {
			unimplemented!("`{}` at {id}", foreign.identifier());
		}
	}

	fn handle_trap(&mut self, id: u32) {
		self.do_assignment(id, Expression::Trap);
	}

	fn handle_null(&mut self, id: u32) {
		self.do_assignment(id, Expression::Null);
	}

	fn handle_i32_const(&mut self, id: u32, value: i32) {
		self.do_assignment(id, Expression::I32(value));
	}

	fn handle_i64_const(&mut self, id: u32, value: i64) {
		self.do_assignment(id, Expression::I64(value));
	}

	fn handle_f32_const(&mut self, id: u32, value: f32) {
		self.do_assignment(id, Expression::F32(value));
	}

	fn handle_f64_const(&mut self, id: u32, value: f64) {
		self.do_assignment(id, Expression::F64(value));
	}

	fn handle_identity(&mut self, id: u32, node: &Identity) {
		let Identity { sources } = node;

		self.code_handler.do_bulk_assignment(
			id,
			sources,
			self.data_handler.scope(),
			&self.data_handler,
		);
	}

	fn handle_fence(&mut self, id: u32, node: &Fence) {
		let Fence { sources } = node;

		self.code_handler.do_bulk_assignment(
			id,
			sources,
			self.data_handler.scope(),
			&self.data_handler,
		);
	}

	fn handle_call_statement(&mut self, id: u32, node: &Apply) {
		self.code_handler.do_call(node, id, &mut self.data_handler);
	}

	fn handle_call_expression(&mut self, id: u32, node: &Apply) {
		let expression = self.data_handler.load_call(node);

		self.do_assignment(id, expression);
	}

	fn handle_call(&mut self, id: u32, node: &Apply) {
		if node.results == 0 || self.data_handler.get_local(Link(id, 0)).is_some() {
			self.handle_call_statement(id, node);
		} else {
			self.handle_call_expression(id, node);
		}
	}

	fn handle_ref_is_null(&mut self, id: u32, node: RefIsNull) {
		let expression = self.data_handler.load_ref_is_null(node);

		self.do_assignment(id, expression);
	}

	fn handle_integer_unary_operation(&mut self, id: u32, node: integer::UnaryOperation) {
		let expression = self.data_handler.load_integer_unary_operation(node);

		self.do_assignment(id, expression);
	}

	fn handle_integer_binary_operation(&mut self, id: u32, node: integer::BinaryOperation) {
		let expression = self.data_handler.load_integer_binary_operation(node);

		self.do_assignment(id, expression);
	}

	fn handle_integer_compare_operation(&mut self, id: u32, node: integer::CompareOperation) {
		let expression = self.data_handler.load_integer_compare_operation(node);

		self.do_assignment(id, expression);
	}

	fn handle_integer_narrow(&mut self, id: u32, node: IntegerNarrow) {
		let expression = self.data_handler.load_integer_narrow(node);

		self.do_assignment(id, expression);
	}

	fn handle_integer_widen(&mut self, id: u32, node: IntegerWiden) {
		let expression = self.data_handler.load_integer_widen(node);

		self.do_assignment(id, expression);
	}

	fn handle_integer_sign_extend(&mut self, id: u32, node: IntegerSignExtend) {
		let expression = self.data_handler.load_integer_sign_extend(node);

		self.do_assignment(id, expression);
	}

	fn handle_integer_convert_to_number(&mut self, id: u32, node: IntegerConvertToNumber) {
		let expression = self.data_handler.load_integer_convert_to_number(node);

		self.do_assignment(id, expression);
	}

	fn handle_integer_transmute_to_number(&mut self, id: u32, node: IntegerTransmuteToNumber) {
		let expression = self.data_handler.load_integer_transmute_to_number(node);

		self.do_assignment(id, expression);
	}

	fn handle_number_unary_operation(&mut self, id: u32, node: number::UnaryOperation) {
		let expression = self.data_handler.load_number_unary_operation(node);

		self.do_assignment(id, expression);
	}

	fn handle_number_binary_operation(&mut self, id: u32, node: number::BinaryOperation) {
		let expression = self.data_handler.load_number_binary_operation(node);

		self.do_assignment(id, expression);
	}

	fn handle_number_compare_operation(&mut self, id: u32, node: number::CompareOperation) {
		let expression = self.data_handler.load_number_compare_operation(node);

		self.do_assignment(id, expression);
	}

	fn handle_number_narrow(&mut self, id: u32, node: NumberNarrow) {
		let expression = self.data_handler.load_number_narrow(node);

		self.do_assignment(id, expression);
	}

	fn handle_number_widen(&mut self, id: u32, node: NumberWiden) {
		let expression = self.data_handler.load_number_widen(node);

		self.do_assignment(id, expression);
	}

	fn handle_number_truncate_to_integer(&mut self, id: u32, node: NumberTruncateToInteger) {
		let expression = self.data_handler.load_number_truncate_to_integer(node);

		self.do_assignment(id, expression);
	}

	fn handle_number_transmute_to_integer(&mut self, id: u32, node: NumberTransmuteToInteger) {
		let expression = self.data_handler.load_number_transmute_to_integer(node);

		self.do_assignment(id, expression);
	}

	fn handle_mutable_new(&mut self, id: u32, node: MutableNew) {
		let expression = self.data_handler.load_mutable_new(node);

		self.do_assignment(id, expression);
	}

	fn handle_mutable_get(&mut self, id: u32, node: MutableGet) {
		let expression = self.data_handler.load_mutable_get(node);

		self.do_assignment(id, expression);

		self.code_handler.do_rename(
			Link(id, MutableGet::STATE_PORT),
			node.source,
			&self.data_handler,
		);
	}

	fn handle_mutable_set(&mut self, id: u32, node: MutableSet) {
		self.code_handler
			.do_mutable_set(node, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, MutableSet::STATE_PORT),
			node.destination,
			&self.data_handler,
		);
	}

	fn handle_aggregate(&mut self, id: u32, node: &Aggregate) {
		let expression = self.data_handler.load_aggregate(node);

		self.do_assignment(id, expression);
	}

	fn handle_extract(&mut self, id: u32, node: Extract) {
		let expression = self.data_handler.load_extract(&node);

		self.do_assignment(id, expression);
	}

	fn handle_table_new(&mut self, id: u32, node: &TableNew) {
		let expression = self.data_handler.load_table_new(node);

		self.do_assignment(id, expression);
	}

	fn handle_table_get(&mut self, id: u32, node: TableGet) {
		let expression = self.data_handler.load_table_get(node);

		self.do_assignment(id, expression);

		self.code_handler.do_rename(
			Link(id, TableGet::STATE_PORT),
			node.source.reference,
			&self.data_handler,
		);
	}

	fn handle_table_set(&mut self, id: u32, node: TableSet) {
		self.code_handler.do_table_set(node, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, TableSet::STATE_PORT),
			node.destination.reference,
			&self.data_handler,
		);
	}

	fn handle_table_size(&mut self, id: u32, node: TableSize) {
		let expression = self.data_handler.load_table_size(node);

		self.do_assignment(id, expression);

		self.code_handler.do_rename(
			Link(id, TableSize::STATE_PORT),
			node.source,
			&self.data_handler,
		);
	}

	fn handle_table_grow(&mut self, id: u32, node: TableGrow) {
		let expression = self.data_handler.load_table_grow(node);

		self.do_assignment(id, expression);

		self.code_handler.do_rename(
			Link(id, TableGrow::STATE_PORT),
			node.destination,
			&self.data_handler,
		);
	}

	fn handle_table_fill(&mut self, id: u32, node: TableFill) {
		self.code_handler
			.do_table_fill(node, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, TableFill::STATE_PORT),
			node.destination.reference,
			&self.data_handler,
		);
	}

	fn handle_table_copy(&mut self, id: u32, node: TableCopy) {
		self.code_handler
			.do_table_copy(node, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, TableCopy::DESTINATION_STATE_PORT),
			node.destination.reference,
			&self.data_handler,
		);

		self.code_handler.do_rename(
			Link(id, TableCopy::SOURCE_STATE_PORT),
			node.source.reference,
			&self.data_handler,
		);
	}

	fn handle_table_drop(&mut self, id: u32, node: TableDrop) {
		self.code_handler
			.do_table_drop(node, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, TableDrop::STATE_PORT),
			node.source,
			&self.data_handler,
		);
	}

	fn handle_memory_new(&mut self, id: u32, node: &MemoryNew) {
		let expression = Expression::MemoryNew(node.clone());

		self.do_assignment(id, expression);
	}

	fn handle_memory_load(&mut self, id: u32, node: MemoryLoad) {
		let expression = self.data_handler.load_memory_load(node);

		self.do_assignment(id, expression);

		self.code_handler.do_rename(
			Link(id, MemoryLoad::STATE_PORT),
			node.source.reference,
			&self.data_handler,
		);
	}

	fn handle_memory_store(&mut self, id: u32, node: MemoryStore) {
		self.code_handler
			.do_memory_store(node, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, MemoryStore::STATE_PORT),
			node.destination.reference,
			&self.data_handler,
		);
	}

	fn handle_memory_size(&mut self, id: u32, node: MemorySize) {
		let expression = self.data_handler.load_memory_size(node);

		self.do_assignment(id, expression);

		self.code_handler.do_rename(
			Link(id, MemorySize::STATE_PORT),
			node.source,
			&self.data_handler,
		);
	}

	fn handle_memory_grow(&mut self, id: u32, node: MemoryGrow) {
		let expression = self.data_handler.load_memory_grow(node);

		self.do_assignment(id, expression);

		self.code_handler.do_rename(
			Link(id, MemoryGrow::STATE_PORT),
			node.destination,
			&self.data_handler,
		);
	}

	fn handle_memory_fill(&mut self, id: u32, node: MemoryFill) {
		self.code_handler
			.do_memory_fill(node, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, MemoryFill::STATE_PORT),
			node.destination.reference,
			&self.data_handler,
		);
	}

	fn handle_memory_copy(&mut self, id: u32, node: MemoryCopy) {
		self.code_handler
			.do_memory_copy(node, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, MemoryCopy::DESTINATION_STATE_PORT),
			node.destination.reference,
			&self.data_handler,
		);

		self.code_handler.do_rename(
			Link(id, MemoryCopy::SOURCE_STATE_PORT),
			node.source.reference,
			&self.data_handler,
		);
	}

	fn handle_memory_drop(&mut self, id: u32, node: MemoryDrop) {
		self.code_handler
			.do_memory_drop(node, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, MemoryDrop::STATE_PORT),
			node.source,
			&self.data_handler,
		);
	}

	#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
	fn handle_node(&mut self, id: u32, node: &Node) {
		match *node {
			Node::Function(ref arc) => self.handle_function(id, arc),
			Node::Match(ref arc) => self.handle_match(id, arc),
			Node::Repeat(ref arc) => self.handle_repeat(id, arc),

			Node::ModuleArguments(_)
			| Node::ModuleResults(_)
			| Node::FunctionArguments(_)
			| Node::FunctionResults(_)
			| Node::BranchArguments(_)
			| Node::BranchResults(_)
			| Node::RepeatArguments(_) => {}

			Node::RepeatResults(ref node) => self.handle_repeat_results(node),

			Node::Foreign(ref node) => self.handle_foreign(id, node.as_ref()),
			Node::Trap => self.handle_trap(id),
			Node::Null => self.handle_null(id),
			Node::I32(value) => self.handle_i32_const(id, value),
			Node::I64(value) => self.handle_i64_const(id, value),
			Node::F32(value) => self.handle_f32_const(id, value),
			Node::F64(value) => self.handle_f64_const(id, value),

			Node::Identity(ref node) => self.handle_identity(id, node),
			Node::Fence(ref node) => self.handle_fence(id, node),
			Node::Apply(ref node) => self.handle_call(id, node),
			Node::RefIsNull(node) => self.handle_ref_is_null(id, node),
			Node::IntegerUnaryOperation(node) => self.handle_integer_unary_operation(id, node),
			Node::IntegerBinaryOperation(node) => self.handle_integer_binary_operation(id, node),
			Node::IntegerCompareOperation(node) => self.handle_integer_compare_operation(id, node),
			Node::IntegerNarrow(node) => self.handle_integer_narrow(id, node),
			Node::IntegerWiden(node) => self.handle_integer_widen(id, node),
			Node::IntegerSignExtend(node) => self.handle_integer_sign_extend(id, node),
			Node::IntegerConvertToNumber(node) => {
				self.handle_integer_convert_to_number(id, node);
			}
			Node::IntegerTransmuteToNumber(node) => {
				self.handle_integer_transmute_to_number(id, node);
			}
			Node::NumberUnaryOperation(node) => self.handle_number_unary_operation(id, node),
			Node::NumberBinaryOperation(node) => self.handle_number_binary_operation(id, node),
			Node::NumberCompareOperation(node) => self.handle_number_compare_operation(id, node),
			Node::NumberNarrow(node) => self.handle_number_narrow(id, node),
			Node::NumberWiden(node) => self.handle_number_widen(id, node),
			Node::NumberTruncateToInteger(node) => self.handle_number_truncate_to_integer(id, node),
			Node::NumberTransmuteToInteger(node) => {
				self.handle_number_transmute_to_integer(id, node);
			}
			Node::MutableNew(node) => self.handle_mutable_new(id, node),
			Node::MutableGet(node) => self.handle_mutable_get(id, node),
			Node::MutableSet(node) => self.handle_mutable_set(id, node),
			Node::Aggregate(ref node) => self.handle_aggregate(id, node),
			Node::Extract(node) => self.handle_extract(id, node),
			Node::TableNew(ref node) => self.handle_table_new(id, node),
			Node::TableGet(node) => self.handle_table_get(id, node),
			Node::TableSet(node) => self.handle_table_set(id, node),
			Node::TableSize(node) => self.handle_table_size(id, node),
			Node::TableGrow(node) => self.handle_table_grow(id, node),
			Node::TableFill(node) => self.handle_table_fill(id, node),
			Node::TableCopy(node) => self.handle_table_copy(id, node),
			Node::TableDrop(node) => self.handle_table_drop(id, node),
			Node::MemoryNew(ref node) => self.handle_memory_new(id, node),
			Node::MemoryLoad(node) => self.handle_memory_load(id, node),
			Node::MemoryStore(node) => self.handle_memory_store(id, node),
			Node::MemorySize(node) => self.handle_memory_size(id, node),
			Node::MemoryGrow(node) => self.handle_memory_grow(id, node),
			Node::MemoryFill(node) => self.handle_memory_fill(id, node),
			Node::MemoryCopy(node) => self.handle_memory_copy(id, node),
			Node::MemoryDrop(node) => self.handle_memory_drop(id, node),
		}
	}

	fn handle_nodes(&mut self, nodes: &[Node], scope: usize) {
		self.data_handler.set_scope(scope);

		for (id, node) in nodes.iter().enumerate() {
			let id = id.try_into().unwrap();

			self.handle_node(id, node);
		}
	}

	/// Builds a `LuaJIT` tree from the given module.
	///
	/// # Panics
	///
	/// Panics if the module does not contain valid structure;
	/// if this happens, it is a bug.
	pub fn run(&mut self, module: &Arc<Mutex<Module>>) -> LuaJITTree {
		let guard = module.lock();
		let scope = Arc::as_ptr(module) as usize;

		self.data_handler.clear();

		let (declarations, assignments) = self.data_handler.locals_mut();

		self.local_allocator.run(
			declarations,
			assignments,
			&guard.nodes,
			scope,
			&guard.results().sources,
		);

		self.handle_module(&guard, scope);

		drop(guard);

		self.luajit_tree.take().unwrap()
	}
}

impl Default for LuaJITBuilder {
	fn default() -> Self {
		Self::new()
	}
}
