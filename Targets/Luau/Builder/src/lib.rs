#![no_std]

use data_flow_graph::{
	DataFlowGraph, Link, Node,
	base::{
		Call, DataDrop, DataNew, ElementsDrop, ElementsNew, GlobalGet, GlobalNew, GlobalSet, Host,
		Identity, IntegerBinaryOperation, IntegerCompareOperation, IntegerConvertToNumber,
		IntegerExtend, IntegerNarrow, IntegerTransmuteToNumber, IntegerUnaryOperation,
		IntegerWiden, MemoryCopy, MemoryFill, MemoryGrow, MemoryInit, MemoryLoad, MemoryNew,
		MemorySize, MemoryStore, Merge, NumberBinaryOperation, NumberCompareOperation,
		NumberNarrow, NumberTransmuteToInteger, NumberTruncateToInteger, NumberUnaryOperation,
		NumberWiden, RefIsNull, TableCopy, TableFill, TableGet, TableGrow, TableInit, TableNew,
		TableSet, TableSize,
	},
	control::{
		GammaIn, GammaOut, Import, LambdaIn, LambdaOut, OmegaIn, OmegaOut, RegionIn, ThetaIn,
		ThetaOut,
	},
};
use luau_tree::{LuauTree, expression::Expression};

use self::{code_handler::CodeHandler, data_handler::DataHandler, local_allocator::LocalAllocator};

extern crate alloc;

mod code_handler;
mod data_handler;
mod local_allocator;

pub struct LuauBuilder {
	local_allocator: LocalAllocator,

	code_handler: CodeHandler,
	data_handler: DataHandler,

	luau_tree: Option<LuauTree>,
}

impl LuauBuilder {
	#[must_use]
	pub fn new() -> Self {
		Self {
			local_allocator: LocalAllocator::new(),

			code_handler: CodeHandler::new(),
			data_handler: DataHandler::new(),

			luau_tree: None,
		}
	}

	fn do_assignment(&mut self, destination: u32, source: Expression) {
		if let Some(destination) = self.data_handler.get_local(Link(destination, 0)) {
			self.code_handler.do_assign(destination, source);
		} else {
			self.data_handler.store_expression(destination, source);
		}
	}

	fn handle_lambda_in(&mut self) {
		self.code_handler.push_scope();
	}

	fn handle_lambda_out(&mut self, graph: &DataFlowGraph, lambda_out: &LambdaOut) {
		let LambdaOut { results, input } = lambda_out;
		let lambda_in @ LambdaIn {
			r#type,
			dependencies,
			output,
		} = graph.get(*input).as_lambda_in().unwrap();

		let dependencies =
			self.data_handler
				.load_dependencies(*input, lambda_in.dependency_ports(), dependencies);

		let arguments = self
			.data_handler
			.load_name_assignments(*input, lambda_in.argument_ports());

		let mut locals = self.data_handler.load_declarations(*input);

		locals.retain(|&name| {
			!arguments.contains(&name) && !dependencies.iter().any(|item| item.0 == name)
		});

		let stack = self.data_handler.get_stack_size(*input);
		let code = self.code_handler.pop_scope();
		let returns = self.data_handler.load_returns(results, r#type);

		let function =
			DataHandler::load_scoped(dependencies, arguments, locals, stack, code, returns);

		self.do_assignment(*output, function);
	}

	fn handle_region_in(&mut self, graph: &DataFlowGraph, id: u32, region_in: &RegionIn) {
		let RegionIn { input, .. } = region_in;
		let GammaIn { arguments, .. } = graph.get(*input).as_gamma_in().unwrap();

		self.code_handler.push_scope();
		self.code_handler
			.do_assign_all(id, arguments, &self.data_handler);
	}

	fn handle_region_out(&mut self, id: u32) {
		self.code_handler.pop_branch(id);
	}

	fn handle_gamma_out(&mut self, graph: &DataFlowGraph, gamma_out: &GammaOut) {
		let GammaOut { input, regions } = gamma_out;
		let GammaIn { condition, .. } = graph.get(*input).as_gamma_in().unwrap();

		self.code_handler
			.do_match(regions, *condition, &mut self.data_handler);
	}

	fn handle_theta_in(&mut self, id: u32, theta_in: &ThetaIn) {
		let ThetaIn { arguments, .. } = theta_in;

		self.code_handler.push_scope();
		self.code_handler
			.do_assign_all(id, arguments, &self.data_handler);
	}

	fn handle_theta_out(&mut self, theta_out: &ThetaOut) {
		let ThetaOut { condition, .. } = theta_out;

		self.code_handler
			.do_repeat(*condition, &mut self.data_handler);
	}

	fn handle_omega_in(&mut self) {
		self.code_handler.push_scope();
	}

	fn handle_omega_out(&mut self, omega_out: &OmegaOut) {
		let OmegaOut { input, exports, .. } = omega_out;

		let environment = Link(*input, OmegaIn::ENVIRONMENT_PORT);
		let environment = self
			.data_handler
			.get_local(environment)
			.unwrap()
			.into_name();

		let mut locals = self.data_handler.load_declarations(*input);

		let position = locals.iter().position(|&name| name == environment).unwrap();

		locals.remove(position);

		let stack = self.data_handler.get_stack_size(*input);
		let code = self.code_handler.pop_scope();
		let exports = self.data_handler.load_exports(exports);

		self.luau_tree = Some(LuauTree {
			environment,
			locals,
			stack,
			code,
			exports,
		});
	}

	fn handle_import(&mut self, id: u32, import: &Import) {
		let import = self.data_handler.load_import(import);

		self.do_assignment(id, import);
	}

	#[expect(clippy::unused_self, clippy::needless_pass_by_ref_mut)]
	fn handle_host(&mut self, id: u32, host: &dyn Host) {
		panic!("unknown host operation {id} `{}`", host.identifier());
	}

	fn handle_trap(&mut self, id: u32) {
		self.do_assignment(id, Expression::Trap);
	}

	fn handle_null(&mut self, id: u32) {
		self.do_assignment(id, Expression::Null);
	}

	fn handle_identity(&mut self, id: u32, identity: Identity) {
		let identity = self.data_handler.load_identity(identity);

		self.do_assignment(id, identity);
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

	fn handle_call_statement(&mut self, id: u32, call: &Call) {
		self.code_handler.do_call(call, id, &mut self.data_handler);
	}

	fn handle_call_expression(&mut self, id: u32, call: &Call) {
		let call = self.data_handler.load_call(call);

		self.do_assignment(id, call);
	}

	fn handle_call(&mut self, id: u32, call: &Call) {
		if call.results == 0 || self.data_handler.get_local(Link(id, 0)).is_some() {
			self.handle_call_statement(id, call);
		} else {
			self.handle_call_expression(id, call);
		}

		let Call {
			ref arguments,
			results,
			states,
			..
		} = *call;

		for (&source, port) in arguments.iter().rev().zip((0..states).rev()) {
			let destination = Link(id, results + port);

			self.code_handler
				.do_rename(destination, source, &self.data_handler);
		}
	}

	fn handle_merge(&mut self, merge: &Merge) {
		let Merge { states } = merge;

		for &source in states {
			let _source = self.data_handler.load(source);
		}
	}

	fn handle_ref_is_null(&mut self, id: u32, ref_is_null: RefIsNull) {
		let ref_is_null = self.data_handler.load_ref_is_null(ref_is_null);

		self.do_assignment(id, ref_is_null);
	}

	fn handle_integer_unary_operation(&mut self, id: u32, operation: IntegerUnaryOperation) {
		let operation = self.data_handler.load_integer_unary_operation(operation);

		self.do_assignment(id, operation);
	}

	fn handle_integer_binary_operation(&mut self, id: u32, operation: IntegerBinaryOperation) {
		let operation = self.data_handler.load_integer_binary_operation(operation);

		self.do_assignment(id, operation);
	}

	fn handle_integer_compare_operation(&mut self, id: u32, operation: IntegerCompareOperation) {
		let operation = self.data_handler.load_integer_compare_operation(operation);

		self.do_assignment(id, operation);
	}

	fn handle_integer_narrow(&mut self, id: u32, operation: IntegerNarrow) {
		let operation = self.data_handler.load_integer_narrow(operation);

		self.do_assignment(id, operation);
	}

	fn handle_integer_widen(&mut self, id: u32, operation: IntegerWiden) {
		let operation = self.data_handler.load_integer_widen(operation);

		self.do_assignment(id, operation);
	}

	fn handle_integer_extend(&mut self, id: u32, operation: IntegerExtend) {
		let operation = self.data_handler.load_integer_extend(operation);

		self.do_assignment(id, operation);
	}

	fn handle_integer_convert_to_number(&mut self, id: u32, operation: IntegerConvertToNumber) {
		let operation = self.data_handler.load_integer_convert_to_number(operation);

		self.do_assignment(id, operation);
	}

	fn handle_integer_transmute_to_number(&mut self, id: u32, operation: IntegerTransmuteToNumber) {
		let operation = self
			.data_handler
			.load_integer_transmute_to_number(operation);

		self.do_assignment(id, operation);
	}

	fn handle_number_unary_operation(&mut self, id: u32, operation: NumberUnaryOperation) {
		let operation = self.data_handler.load_number_unary_operation(operation);

		self.do_assignment(id, operation);
	}

	fn handle_number_binary_operation(&mut self, id: u32, operation: NumberBinaryOperation) {
		let operation = self.data_handler.load_number_binary_operation(operation);

		self.do_assignment(id, operation);
	}

	fn handle_number_compare_operation(&mut self, id: u32, operation: NumberCompareOperation) {
		let operation = self.data_handler.load_number_compare_operation(operation);

		self.do_assignment(id, operation);
	}

	fn handle_number_narrow(&mut self, id: u32, operation: NumberNarrow) {
		let operation = self.data_handler.load_number_narrow(operation);

		self.do_assignment(id, operation);
	}

	fn handle_number_widen(&mut self, id: u32, operation: NumberWiden) {
		let operation = self.data_handler.load_number_widen(operation);

		self.do_assignment(id, operation);
	}

	fn handle_number_truncate_to_integer(&mut self, id: u32, operation: NumberTruncateToInteger) {
		let operation = self.data_handler.load_number_truncate_to_integer(operation);

		self.do_assignment(id, operation);
	}

	fn handle_number_transmute_to_integer(&mut self, id: u32, operation: NumberTransmuteToInteger) {
		let operation = self
			.data_handler
			.load_number_transmute_to_integer(operation);

		self.do_assignment(id, operation);
	}

	fn handle_global_new(&mut self, id: u32, global_new: GlobalNew) {
		let global_new = self.data_handler.load_global_new(global_new);

		self.do_assignment(id, global_new);
	}

	fn handle_global_get(&mut self, id: u32, global_get: GlobalGet) {
		let result = self.data_handler.load_global_get(global_get);

		self.do_assignment(id, result);
	}

	fn handle_global_set(&mut self, id: u32, global_set: GlobalSet) {
		self.code_handler
			.do_global_set(global_set, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, GlobalSet::STATE_PORT),
			global_set.destination,
			&self.data_handler,
		);
	}

	fn handle_table_new(&mut self, id: u32, table_new: TableNew) {
		let table_new = self.data_handler.load_table_new(table_new);

		self.do_assignment(id, table_new);
	}

	fn handle_table_get(&mut self, id: u32, table_get: TableGet) {
		let result = self.data_handler.load_table_get(table_get);

		self.do_assignment(id, result);
	}

	fn handle_table_set(&mut self, id: u32, table_set: TableSet) {
		self.code_handler
			.do_table_set(table_set, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, TableSet::STATE_PORT),
			table_set.destination.reference,
			&self.data_handler,
		);
	}

	fn handle_table_size(&mut self, id: u32, table_size: TableSize) {
		let result = self.data_handler.load_table_size(table_size);

		self.do_assignment(id, result);
	}

	fn handle_table_grow(&mut self, id: u32, table_grow: TableGrow) {
		let result = self.data_handler.load_table_grow(table_grow);

		self.do_assignment(id, result);

		self.code_handler.do_rename(
			Link(id, TableGrow::STATE_PORT),
			table_grow.destination,
			&self.data_handler,
		);
	}

	fn handle_table_fill(&mut self, id: u32, table_fill: TableFill) {
		self.code_handler
			.do_table_fill(table_fill, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, TableFill::STATE_PORT),
			table_fill.destination.reference,
			&self.data_handler,
		);
	}

	fn handle_table_copy(&mut self, id: u32, table_copy: TableCopy) {
		self.code_handler
			.do_table_copy(table_copy, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, TableCopy::DESTINATION_STATE_PORT),
			table_copy.destination.reference,
			&self.data_handler,
		);

		self.code_handler.do_rename(
			Link(id, TableCopy::SOURCE_STATE_PORT),
			table_copy.source.reference,
			&self.data_handler,
		);
	}

	fn handle_table_init(&mut self, id: u32, table_init: TableInit) {
		self.code_handler
			.do_table_init(table_init, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, TableInit::DESTINATION_STATE_PORT),
			table_init.destination.reference,
			&self.data_handler,
		);

		self.code_handler.do_rename(
			Link(id, TableInit::SOURCE_STATE_PORT),
			table_init.source.reference,
			&self.data_handler,
		);
	}

	fn handle_elements_new(&mut self, id: u32, elements_new: &ElementsNew) {
		let elements_new = self.data_handler.load_elements_new(elements_new);

		self.do_assignment(id, elements_new);
	}

	fn handle_elements_drop(&mut self, id: u32, elements_drop: ElementsDrop) {
		self.code_handler
			.do_elements_drop(elements_drop, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, ElementsDrop::STATE_PORT),
			elements_drop.source,
			&self.data_handler,
		);
	}

	fn handle_memory_new(&mut self, id: u32, memory_new: MemoryNew) {
		let memory_new = Expression::MemoryNew(memory_new);

		self.do_assignment(id, memory_new);
	}

	fn handle_memory_load(&mut self, id: u32, memory_load: MemoryLoad) {
		let result = self.data_handler.load_memory_load(memory_load);

		self.do_assignment(id, result);
	}

	fn handle_memory_store(&mut self, id: u32, memory_store: MemoryStore) {
		self.code_handler
			.do_memory_store(memory_store, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, MemoryStore::STATE_PORT),
			memory_store.destination.reference,
			&self.data_handler,
		);
	}

	fn handle_memory_size(&mut self, id: u32, memory_size: MemorySize) {
		let result = self.data_handler.load_memory_size(memory_size);

		self.do_assignment(id, result);
	}

	fn handle_memory_grow(&mut self, id: u32, memory_grow: MemoryGrow) {
		let result = self.data_handler.load_memory_grow(memory_grow);

		self.do_assignment(id, result);

		self.code_handler.do_rename(
			Link(id, MemoryGrow::STATE_PORT),
			memory_grow.destination,
			&self.data_handler,
		);
	}

	fn handle_memory_fill(&mut self, id: u32, memory_fill: MemoryFill) {
		self.code_handler
			.do_memory_fill(memory_fill, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, MemoryFill::STATE_PORT),
			memory_fill.destination.reference,
			&self.data_handler,
		);
	}

	fn handle_memory_copy(&mut self, id: u32, memory_copy: MemoryCopy) {
		self.code_handler
			.do_memory_copy(memory_copy, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, MemoryCopy::DESTINATION_STATE_PORT),
			memory_copy.destination.reference,
			&self.data_handler,
		);

		self.code_handler.do_rename(
			Link(id, MemoryCopy::SOURCE_STATE_PORT),
			memory_copy.source.reference,
			&self.data_handler,
		);
	}

	fn handle_memory_init(&mut self, id: u32, memory_init: MemoryInit) {
		self.code_handler
			.do_memory_init(memory_init, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, MemoryInit::DESTINATION_STATE_PORT),
			memory_init.destination.reference,
			&self.data_handler,
		);

		self.code_handler.do_rename(
			Link(id, MemoryInit::SOURCE_STATE_PORT),
			memory_init.source.reference,
			&self.data_handler,
		);
	}

	fn handle_data_new(&mut self, id: u32, data_new: &DataNew) {
		let data_new = Expression::DataNew(data_new.clone());

		self.do_assignment(id, data_new);
	}

	fn handle_data_drop(&mut self, id: u32, data_drop: DataDrop) {
		self.code_handler
			.do_data_drop(data_drop, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, DataDrop::STATE_PORT),
			data_drop.source,
			&self.data_handler,
		);
	}

	fn handle_node(&mut self, graph: &DataFlowGraph, id: u32, node: &Node) {
		match *node {
			Node::GammaIn(_) => {}

			Node::LambdaIn(_) => self.handle_lambda_in(),
			Node::LambdaOut(ref lambda_out) => self.handle_lambda_out(graph, lambda_out),
			Node::RegionIn(ref region_in) => self.handle_region_in(graph, id, region_in),
			Node::RegionOut(_) => self.handle_region_out(id),
			Node::GammaOut(ref gamma_out) => self.handle_gamma_out(graph, gamma_out),
			Node::ThetaIn(ref theta_in) => self.handle_theta_in(id, theta_in),
			Node::ThetaOut(ref theta_out) => self.handle_theta_out(theta_out),
			Node::OmegaIn(_) => self.handle_omega_in(),
			Node::OmegaOut(ref omega_out) => self.handle_omega_out(omega_out),

			Node::Import(ref import) => self.handle_import(id, import),
			Node::Host(ref host) => self.handle_host(id, host.as_ref()),
			Node::Trap => self.handle_trap(id),
			Node::Null => self.handle_null(id),
			Node::Identity(identity) => self.handle_identity(id, identity),
			Node::I32(i32) => self.handle_i32_const(id, i32),
			Node::I64(i64) => self.handle_i64_const(id, i64),
			Node::F32(f32) => self.handle_f32_const(id, f32),
			Node::F64(f64) => self.handle_f64_const(id, f64),

			Node::Call(ref call) => self.handle_call(id, call),
			Node::Merge(ref merge) => self.handle_merge(merge),
			Node::RefIsNull(ref_is_null) => self.handle_ref_is_null(id, ref_is_null),
			Node::IntegerUnaryOperation(integer_unary_operation) => {
				self.handle_integer_unary_operation(id, integer_unary_operation);
			}
			Node::IntegerBinaryOperation(integer_binary_operation) => {
				self.handle_integer_binary_operation(id, integer_binary_operation);
			}
			Node::IntegerCompareOperation(integer_compare_operation) => {
				self.handle_integer_compare_operation(id, integer_compare_operation);
			}
			Node::IntegerNarrow(integer_narrow) => self.handle_integer_narrow(id, integer_narrow),
			Node::IntegerWiden(integer_widen) => self.handle_integer_widen(id, integer_widen),
			Node::IntegerExtend(integer_extend) => self.handle_integer_extend(id, integer_extend),
			Node::IntegerConvertToNumber(integer_convert_to_number) => {
				self.handle_integer_convert_to_number(id, integer_convert_to_number);
			}
			Node::IntegerTransmuteToNumber(integer_transmute_to_number) => {
				self.handle_integer_transmute_to_number(id, integer_transmute_to_number);
			}
			Node::NumberUnaryOperation(number_unary_operation) => {
				self.handle_number_unary_operation(id, number_unary_operation);
			}
			Node::NumberBinaryOperation(number_binary_operation) => {
				self.handle_number_binary_operation(id, number_binary_operation);
			}
			Node::NumberCompareOperation(number_compare_operation) => {
				self.handle_number_compare_operation(id, number_compare_operation);
			}
			Node::NumberNarrow(number_narrow) => self.handle_number_narrow(id, number_narrow),
			Node::NumberWiden(number_widen) => self.handle_number_widen(id, number_widen),
			Node::NumberTruncateToInteger(number_truncate_to_integer) => {
				self.handle_number_truncate_to_integer(id, number_truncate_to_integer);
			}
			Node::NumberTransmuteToInteger(number_transmute_to_integer) => {
				self.handle_number_transmute_to_integer(id, number_transmute_to_integer);
			}
			Node::GlobalNew(global_new) => self.handle_global_new(id, global_new),
			Node::GlobalGet(global_get) => self.handle_global_get(id, global_get),
			Node::GlobalSet(global_set) => self.handle_global_set(id, global_set),
			Node::TableNew(table_new) => self.handle_table_new(id, table_new),
			Node::TableGet(table_get) => self.handle_table_get(id, table_get),
			Node::TableSet(table_set) => self.handle_table_set(id, table_set),
			Node::TableSize(table_size) => self.handle_table_size(id, table_size),
			Node::TableGrow(table_grow) => self.handle_table_grow(id, table_grow),
			Node::TableFill(table_fill) => self.handle_table_fill(id, table_fill),
			Node::TableCopy(table_copy) => self.handle_table_copy(id, table_copy),
			Node::TableInit(table_init) => self.handle_table_init(id, table_init),
			Node::ElementsNew(ref elements_new) => self.handle_elements_new(id, elements_new),
			Node::ElementsDrop(elements_drop) => self.handle_elements_drop(id, elements_drop),
			Node::MemoryNew(memory_new) => self.handle_memory_new(id, memory_new),
			Node::MemoryLoad(memory_load) => self.handle_memory_load(id, memory_load),
			Node::MemoryStore(memory_store) => self.handle_memory_store(id, memory_store),
			Node::MemorySize(memory_size) => self.handle_memory_size(id, memory_size),
			Node::MemoryGrow(memory_grow) => self.handle_memory_grow(id, memory_grow),
			Node::MemoryFill(memory_fill) => self.handle_memory_fill(id, memory_fill),
			Node::MemoryCopy(memory_copy) => self.handle_memory_copy(id, memory_copy),
			Node::MemoryInit(memory_init) => self.handle_memory_init(id, memory_init),
			Node::DataNew(ref data_new) => self.handle_data_new(id, data_new),
			Node::DataDrop(data_drop) => self.handle_data_drop(id, data_drop),
		}
	}

	pub fn run(&mut self, graph: &DataFlowGraph) -> LuauTree {
		let (declarations, assignments) = self.data_handler.locals_mut();

		self.local_allocator.run(declarations, assignments, graph);

		for (node, id) in graph.nodes().zip(0..) {
			self.handle_node(graph, id, node);
		}

		self.luau_tree.take().unwrap()
	}
}

impl Default for LuauBuilder {
	fn default() -> Self {
		Self::new()
	}
}
