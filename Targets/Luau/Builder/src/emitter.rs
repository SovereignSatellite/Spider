use alloc::sync::Arc;
use core::any::Any;

use parking_lot::Mutex;

use ir_graph::{
	Link, Node,
	foreign::Foreign,
	operation::{
		self, Aggregate, Apply, Extract, Fence, Identity, IntegerConvertToNumber, IntegerNarrow,
		IntegerSignExtend, IntegerTransmuteToNumber, IntegerWiden, MemoryCopy, MemoryDrop,
		MemoryFill, MemoryGrow, MemoryLoad, MemoryNew, MemorySize, MemoryStore, MutableGet,
		MutableNew, MutableSet, NumberNarrow, NumberTransmuteToInteger, NumberTruncateToInteger,
		NumberWiden, RefIsNull, TableCopy, TableDrop, TableFill, TableGet, TableGrow, TableNew,
		TableSet, TableSize, integer, number,
	},
	region::{Branch, Function, Match, Repeat, repeat},
};
use luau_tree::{
	expression::{self, Expression, Local, Location, Name},
	statement::Sequence,
};
use turing_machine_foreign::{Ask as TuringAsk, Tell as TuringTell};
use web_assembly_foreign::{Export as WasmExport, Import as WasmImport};

use super::{
	code_handler::CodeHandler,
	data_handler::{self, DataHandler},
	policy::{LuauPolicy, PHYSICAL_REGISTERS},
};

pub struct Emitter<'allocator, 'policy> {
	allocator: &'allocator mut ir_allocator::Allocator,
	policy: &'policy LuauPolicy,
	code_handler: CodeHandler,
	data_handler: DataHandler,
	scope: usize,
}

fn fast_locals_for(peak: u32, argument_count: u16) -> Vec<Name> {
	let fast_count = peak.min(PHYSICAL_REGISTERS);
	let start = u32::from(argument_count);

	(start..fast_count).map(|id| Name { id }).collect()
}

fn stack_size_for(peak: u32) -> u16 {
	let spill = peak.saturating_sub(PHYSICAL_REGISTERS);

	u16::try_from(spill).unwrap()
}

fn collect_argument_names(argument_count: u16) -> Vec<Name> {
	(0..u32::from(argument_count))
		.map(|id| Name { id })
		.collect()
}

impl<'allocator, 'policy> Emitter<'allocator, 'policy> {
	#[must_use]
	pub fn new(
		allocator: &'allocator mut ir_allocator::Allocator,
		policy: &'policy LuauPolicy,
	) -> Self {
		Self {
			allocator,
			policy,
			code_handler: CodeHandler::new(),
			data_handler: DataHandler::new(),
			scope: 0,
		}
	}

	fn emit_assignment(&mut self, destination: u32, source: Expression) {
		if let Some(local) = self
			.data_handler
			.get_local(self.scope, Link(destination, 0))
		{
			self.code_handler.emit_assign(local, source);
		} else {
			self.data_handler
				.store_expression(self.scope, destination, source);
		}
	}

	fn bridge_state(&mut self, state_link: Link, expression: Expression) {
		let local = self.data_handler.get_local(self.scope, state_link).unwrap();

		self.code_handler.emit_assign(local, expression);
	}

	fn bridge_link(&mut self, state_link: Link, source_link: Link) -> Expression {
		let source = self.data_handler.load(self.scope, source_link);

		self.bridge_state(state_link, source);

		self.data_handler.load(self.scope, state_link)
	}

	fn bridge_location(&mut self, state_link: Link, location: operation::Location) -> Location {
		let reference = self.bridge_link(state_link, location.reference);
		let offset = self.data_handler.load(self.scope, location.offset);

		Location { reference, offset }
	}

	fn bridge_argument_names(&mut self, names: &[Name]) {
		let pairs: Vec<(Local, Local)> = names
			.iter()
			.enumerate()
			.filter_map(|(port, &name)| {
				let port = port.try_into().unwrap();
				let link = Link(Function::ARGUMENTS_ID, port);
				let destination = self.data_handler.get_local(self.scope, link)?;

				Some((destination, Local::Fast { name }))
			})
			.collect();

		self.code_handler.emit_local_moves(pairs);
	}

	fn load_function_returns(&mut self, results: &[Link], scope: usize) -> Vec<Expression> {
		results
			.iter()
			.map(|&link| {
				if let Some(local) = self.data_handler.get_local(scope, link) {
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
		let peak = self.allocator.run(
			self.policy,
			function_scope,
			&function.nodes,
			self.data_handler.registers_mut(),
		);

		self.scope = function_scope;
		self.code_handler.push_scope();

		self.bridge_argument_names(&argument_names);
		self.handle_nodes(&function.nodes, function_scope);

		let locals = fast_locals_for(peak, function.argument_count);
		let stack = stack_size_for(peak);
		let code = self.code_handler.pop_scope();
		let returns = self.load_function_returns(&function.results().sources, function_scope);

		expression::Function {
			arguments: argument_names,
			locals,
			stack,
			code,
			returns,
		}
	}

	fn handle_function(&mut self, id: u32, arc: &Arc<Mutex<Function>>) {
		let function = arc.lock();
		let function_scope = Arc::as_ptr(arc) as usize;

		let argument_names = collect_argument_names(function.argument_count);

		let mut child = Emitter::new(&mut *self.allocator, self.policy);
		let inner = child.emit_function_body(&function, argument_names, function_scope);

		drop(function);

		self.emit_assignment(id, Expression::Function(inner.into()));
	}

	fn load_match_result_locals(&self, id: u32, scope: usize, result_count: u16) -> Vec<Local> {
		(0..result_count)
			.filter_map(|port| self.data_handler.get_local(scope, Link(id, port)))
			.collect()
	}

	fn handle_branch(
		&mut self,
		branch_arc: &Arc<Mutex<Branch>>,
		matcher: &Match,
		parent_scope: usize,
		result_locals: &[Local],
	) -> Sequence {
		let branch = branch_arc.lock();
		let branch_scope = Arc::as_ptr(branch_arc) as usize;

		self.scope = branch_scope;
		self.code_handler.push_scope();

		self.code_handler
			.emit_local_moves(self.data_handler.load_local_moves(
				branch_scope,
				0,
				&matcher.arguments,
				parent_scope,
			));

		self.handle_nodes(&branch.nodes, branch_scope);

		let pairs: Vec<(Local, Local)> = branch
			.results()
			.sources
			.iter()
			.zip(result_locals)
			.filter_map(|(&result, &canonical)| {
				let actual = self.data_handler.get_local(branch_scope, result)?;

				(actual != canonical).then_some((canonical, actual))
			})
			.collect();

		self.code_handler.emit_local_moves(pairs);

		drop(branch);

		self.code_handler.pop_scope()
	}

	fn handle_match(&mut self, id: u32, arc: &Arc<Mutex<Match>>) {
		let matcher = arc.lock();
		let parent_scope = self.scope;

		let result_count = matcher.result_count();
		let result_locals = self.load_match_result_locals(id, parent_scope, result_count);

		let branches: Vec<_> = matcher
			.branches
			.iter()
			.map(|branch_arc| {
				self.handle_branch(branch_arc, &matcher, parent_scope, &result_locals)
			})
			.collect();

		self.scope = parent_scope;

		self.code_handler.emit_match(
			branches,
			matcher.condition,
			parent_scope,
			&mut self.data_handler,
		);
	}

	fn handle_repeat(&mut self, id: u32, arc: &Arc<Mutex<Repeat>>) {
		let repeat = arc.lock();
		let repeat_scope = Arc::as_ptr(arc) as usize;
		let parent_scope = self.scope;
		let argument_count = repeat.argument_count();

		self.scope = repeat_scope;
		self.code_handler
			.emit_local_moves(self.data_handler.load_local_moves(
				repeat_scope,
				0,
				&repeat.arguments,
				parent_scope,
			));

		self.code_handler.push_scope();

		self.handle_nodes(&repeat.nodes, repeat_scope);

		drop(repeat);

		self.scope = parent_scope;
		self.bridge_repeat_outputs(id, argument_count, repeat_scope, parent_scope);
	}

	fn bridge_repeat_outputs(
		&mut self,
		id: u32,
		argument_count: u16,
		repeat_scope: usize,
		parent_scope: usize,
	) {
		let pairs: Vec<(Local, Local)> = (0..argument_count)
			.filter_map(|port| {
				let destination = self.data_handler.get_local(parent_scope, Link(id, port))?;
				let source = self.data_handler.get_local(repeat_scope, Link(0, port))?;

				(destination != source).then_some((destination, source))
			})
			.collect();

		self.code_handler.emit_local_moves(pairs);
	}

	fn handle_repeat_results(&mut self, node: &repeat::Results) {
		let scope = self.scope;

		self.code_handler
			.emit_local_moves(
				self.data_handler
					.load_local_moves(scope, 0, &node.sources, scope),
			);

		self.code_handler
			.emit_repeat(node.condition, scope, &mut self.data_handler);
	}

	pub fn emit_function(&mut self, function: &Function, scope: usize) -> expression::Function {
		let argument_names = collect_argument_names(function.argument_count);

		self.emit_function_body(function, argument_names, scope)
	}

	fn handle_wasm_import(&mut self, id: u32, node: &WasmImport) {
		let expression = data_handler::build_wasm_import(node);

		self.emit_assignment(id, expression);
	}

	fn handle_wasm_export(&mut self, node: &WasmExport) {
		let value = self.data_handler.load(self.scope, node.value);
		let identifier = Expression::String(Arc::clone(&node.identifier));

		self.code_handler
			.emit_runtime_call("export", vec![identifier, value]);
	}

	fn handle_turing_ask(&mut self, id: u32, node: TuringAsk) {
		let state = self.data_handler.load(self.scope, node.state);

		self.bridge_state(Link(id, TuringAsk::STATE_PORT), state);

		let expression = data_handler::build_turing_ask();

		self.emit_assignment(id, expression);
	}

	fn handle_turing_tell(&mut self, id: u32, node: TuringTell) {
		let state = self.data_handler.load(self.scope, node.state);

		self.bridge_state(Link(id, TuringTell::STATE_PORT), state);

		let character = self.data_handler.load(self.scope, node.character);

		self.code_handler
			.emit_runtime_call("turing_tell", vec![character]);
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
		self.emit_assignment(id, Expression::Trap);
	}

	fn handle_null(&mut self, id: u32) {
		self.emit_assignment(id, Expression::Null);
	}

	fn handle_i32_const(&mut self, id: u32, value: i32) {
		self.emit_assignment(id, Expression::I32(value));
	}

	fn handle_i64_const(&mut self, id: u32, value: i64) {
		self.emit_assignment(id, Expression::I64(value));
	}

	fn handle_f32_const(&mut self, id: u32, value: f32) {
		self.emit_assignment(id, Expression::F32(value));
	}

	fn handle_f64_const(&mut self, id: u32, value: f64) {
		self.emit_assignment(id, Expression::F64(value));
	}

	fn handle_identity(&mut self, id: u32, node: &Identity) {
		let Identity { sources } = node;

		self.code_handler.emit_local_moves(
			self.data_handler
				.load_local_moves(self.scope, id, sources, self.scope),
		);
	}

	fn handle_fence(&mut self, id: u32, node: &Fence) {
		let Fence { sources } = node;

		self.code_handler.emit_local_moves(
			self.data_handler
				.load_local_moves(self.scope, id, sources, self.scope),
		);
	}

	fn handle_call_statement(&mut self, id: u32, node: &Apply) {
		self.code_handler
			.emit_call(self.scope, node, id, &mut self.data_handler);
	}

	fn handle_call_expression(&mut self, id: u32, node: &Apply) {
		let expression = self.data_handler.build_call(self.scope, node);

		self.emit_assignment(id, expression);
	}

	fn handle_call(&mut self, id: u32, node: &Apply) {
		if node.result_count == 0
			|| self
				.data_handler
				.get_local(self.scope, Link(id, 0))
				.is_some()
		{
			self.handle_call_statement(id, node);
		} else {
			self.handle_call_expression(id, node);
		}
	}

	fn handle_ref_is_null(&mut self, id: u32, node: RefIsNull) {
		let expression = self.data_handler.build_ref_is_null(self.scope, node);

		self.emit_assignment(id, expression);
	}

	fn handle_integer_unary_operation(&mut self, id: u32, node: integer::UnaryOperation) {
		let expression = self
			.data_handler
			.build_integer_unary_operation(self.scope, node);

		self.emit_assignment(id, expression);
	}

	fn handle_integer_binary_operation(&mut self, id: u32, node: integer::BinaryOperation) {
		let expression = self
			.data_handler
			.build_integer_binary_operation(self.scope, node);

		self.emit_assignment(id, expression);
	}

	fn handle_integer_compare_operation(&mut self, id: u32, node: integer::CompareOperation) {
		let expression = self
			.data_handler
			.build_integer_compare_operation(self.scope, node);

		self.emit_assignment(id, expression);
	}

	fn handle_integer_narrow(&mut self, id: u32, node: IntegerNarrow) {
		let expression = self.data_handler.build_integer_narrow(self.scope, node);

		self.emit_assignment(id, expression);
	}

	fn handle_integer_widen(&mut self, id: u32, node: IntegerWiden) {
		let expression = self.data_handler.build_integer_widen(self.scope, node);

		self.emit_assignment(id, expression);
	}

	fn handle_integer_sign_extend(&mut self, id: u32, node: IntegerSignExtend) {
		let expression = self
			.data_handler
			.build_integer_sign_extend(self.scope, node);

		self.emit_assignment(id, expression);
	}

	fn handle_integer_convert_to_number(&mut self, id: u32, node: IntegerConvertToNumber) {
		let expression = self
			.data_handler
			.build_integer_convert_to_number(self.scope, node);

		self.emit_assignment(id, expression);
	}

	fn handle_integer_transmute_to_number(&mut self, id: u32, node: IntegerTransmuteToNumber) {
		let expression = self
			.data_handler
			.build_integer_transmute_to_number(self.scope, node);

		self.emit_assignment(id, expression);
	}

	fn handle_number_unary_operation(&mut self, id: u32, node: number::UnaryOperation) {
		let expression = self
			.data_handler
			.build_number_unary_operation(self.scope, node);

		self.emit_assignment(id, expression);
	}

	fn handle_number_binary_operation(&mut self, id: u32, node: number::BinaryOperation) {
		let expression = self
			.data_handler
			.build_number_binary_operation(self.scope, node);

		self.emit_assignment(id, expression);
	}

	fn handle_number_compare_operation(&mut self, id: u32, node: number::CompareOperation) {
		let expression = self
			.data_handler
			.build_number_compare_operation(self.scope, node);

		self.emit_assignment(id, expression);
	}

	fn handle_number_narrow(&mut self, id: u32, node: NumberNarrow) {
		let expression = self.data_handler.build_number_narrow(self.scope, node);

		self.emit_assignment(id, expression);
	}

	fn handle_number_widen(&mut self, id: u32, node: NumberWiden) {
		let expression = self.data_handler.build_number_widen(self.scope, node);

		self.emit_assignment(id, expression);
	}

	fn handle_number_truncate_to_integer(&mut self, id: u32, node: NumberTruncateToInteger) {
		let expression = self
			.data_handler
			.build_number_truncate_to_integer(self.scope, node);

		self.emit_assignment(id, expression);
	}

	fn handle_number_transmute_to_integer(&mut self, id: u32, node: NumberTransmuteToInteger) {
		let expression = self
			.data_handler
			.build_number_transmute_to_integer(self.scope, node);

		self.emit_assignment(id, expression);
	}

	fn handle_mutable_new(&mut self, id: u32, node: MutableNew) {
		let expression = self.data_handler.build_mutable_new(self.scope, node);

		self.emit_assignment(id, expression);
	}

	fn handle_mutable_get(&mut self, id: u32, node: MutableGet) {
		let source = self.bridge_link(Link(id, MutableGet::STATE_PORT), node.source);
		let expression = data_handler::build_mutable_get(source);

		self.emit_assignment(id, expression);
	}

	fn handle_mutable_set(&mut self, id: u32, node: MutableSet) {
		let destination = self.bridge_link(Link(id, MutableSet::STATE_PORT), node.destination);
		let source = self.data_handler.load(self.scope, node.source);

		self.code_handler.emit_mutable_set(destination, source);
	}

	fn handle_aggregate(&mut self, id: u32, node: &Aggregate) {
		let expression = self.data_handler.build_aggregate(self.scope, node);

		self.emit_assignment(id, expression);
	}

	fn handle_extract(&mut self, id: u32, node: Extract) {
		let expression = self.data_handler.build_extract(self.scope, &node);

		self.emit_assignment(id, expression);
	}

	fn handle_table_new(&mut self, id: u32, node: &TableNew) {
		let expression = self.data_handler.build_table_new(self.scope, node);

		self.emit_assignment(id, expression);
	}

	fn handle_table_get(&mut self, id: u32, node: TableGet) {
		let source = self.bridge_location(Link(id, TableGet::STATE_PORT), node.source);
		let expression = data_handler::build_table_get(source);

		self.emit_assignment(id, expression);
	}

	fn handle_table_set(&mut self, id: u32, node: TableSet) {
		let destination = self.bridge_location(Link(id, TableSet::STATE_PORT), node.destination);
		let source = self.data_handler.load(self.scope, node.source);

		self.code_handler.emit_table_set(destination, source);
	}

	fn handle_table_size(&mut self, id: u32, node: TableSize) {
		let source = self.bridge_link(Link(id, TableSize::STATE_PORT), node.source);
		let expression = data_handler::build_table_size(source);

		self.emit_assignment(id, expression);
	}

	fn handle_table_grow(&mut self, id: u32, node: TableGrow) {
		let destination = self.bridge_link(Link(id, TableGrow::STATE_PORT), node.destination);
		let initializer = self.data_handler.load(self.scope, node.initializer);
		let size = self.data_handler.load(self.scope, node.size);
		let expression = data_handler::build_table_grow(destination, initializer, size);

		self.emit_assignment(id, expression);
	}

	fn handle_table_fill(&mut self, id: u32, node: TableFill) {
		let destination = self.bridge_location(Link(id, TableFill::STATE_PORT), node.destination);
		let source = self.data_handler.load(self.scope, node.source);
		let size = self.data_handler.load(self.scope, node.size);

		self.code_handler.emit_table_fill(destination, source, size);
	}

	fn handle_table_copy(&mut self, id: u32, node: TableCopy) {
		let destination = self.bridge_location(
			Link(id, TableCopy::DESTINATION_STATE_PORT),
			node.destination,
		);
		let source = self.bridge_location(Link(id, TableCopy::SOURCE_STATE_PORT), node.source);
		let size = self.data_handler.load(self.scope, node.size);

		self.code_handler.emit_table_copy(destination, source, size);
	}

	fn handle_table_drop(&mut self, id: u32, node: TableDrop) {
		let source = self.bridge_link(Link(id, TableDrop::STATE_PORT), node.source);

		self.code_handler.emit_table_drop(source);
	}

	fn handle_memory_new(&mut self, id: u32, node: &MemoryNew) {
		let expression = Expression::MemoryNew(node.clone());

		self.emit_assignment(id, expression);
	}

	fn handle_memory_load(&mut self, id: u32, node: MemoryLoad) {
		let source = self.bridge_location(Link(id, MemoryLoad::STATE_PORT), node.source);
		let expression = data_handler::build_memory_load(source, node.kind);

		self.emit_assignment(id, expression);
	}

	fn handle_memory_store(&mut self, id: u32, node: MemoryStore) {
		let destination = self.bridge_location(Link(id, MemoryStore::STATE_PORT), node.destination);
		let source = self.data_handler.load(self.scope, node.source);

		self.code_handler
			.emit_memory_store(destination, source, node.kind);
	}

	fn handle_memory_size(&mut self, id: u32, node: MemorySize) {
		let source = self.bridge_link(Link(id, MemorySize::STATE_PORT), node.source);
		let expression = data_handler::build_memory_size(source);

		self.emit_assignment(id, expression);
	}

	fn handle_memory_grow(&mut self, id: u32, node: MemoryGrow) {
		let destination = self.bridge_link(Link(id, MemoryGrow::STATE_PORT), node.destination);
		let size = self.data_handler.load(self.scope, node.size);
		let expression = data_handler::build_memory_grow(destination, size);

		self.emit_assignment(id, expression);
	}

	fn handle_memory_fill(&mut self, id: u32, node: MemoryFill) {
		let destination = self.bridge_location(Link(id, MemoryFill::STATE_PORT), node.destination);
		let byte = self.data_handler.load(self.scope, node.byte);
		let size = self.data_handler.load(self.scope, node.size);

		self.code_handler.emit_memory_fill(destination, byte, size);
	}

	fn handle_memory_copy(&mut self, id: u32, node: MemoryCopy) {
		let destination = self.bridge_location(
			Link(id, MemoryCopy::DESTINATION_STATE_PORT),
			node.destination,
		);
		let source = self.bridge_location(Link(id, MemoryCopy::SOURCE_STATE_PORT), node.source);
		let size = self.data_handler.load(self.scope, node.size);

		self.code_handler
			.emit_memory_copy(destination, source, size);
	}

	fn handle_memory_drop(&mut self, id: u32, node: MemoryDrop) {
		let source = self.bridge_link(Link(id, MemoryDrop::STATE_PORT), node.source);

		self.code_handler.emit_memory_drop(source);
	}

	#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
	fn handle_node(&mut self, id: u32, node: &Node) {
		match *node {
			Node::Function(ref arc) => self.handle_function(id, arc),
			Node::Match(ref arc) => self.handle_match(id, arc),
			Node::Repeat(ref arc) => self.handle_repeat(id, arc),

			Node::FunctionArguments(_)
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
		self.scope = scope;

		for (id, node) in nodes.iter().enumerate() {
			let id = id.try_into().unwrap();

			self.handle_node(id, node);
		}
	}
}
