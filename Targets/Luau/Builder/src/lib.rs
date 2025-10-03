#![no_std]
#![expect(clippy::missing_panics_doc)]

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

	fn handle_host(&mut self, id: u32, host: &dyn Host) {
		panic!("unknown host operation {id} `{}`", host.identifier());
	}

	fn handle_trap(&mut self, id: u32) {
		self.do_assignment(id, Expression::Trap);
	}

	fn handle_null(&mut self, id: u32) {
		self.do_assignment(id, Expression::Null);
	}

	fn handle_identity(&mut self, id: u32, node: Identity) {
		let expression = self.data_handler.load_identity(node);

		self.do_assignment(id, expression);
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

	fn handle_call_statement(&mut self, id: u32, node: &Call) {
		self.code_handler.do_call(node, id, &mut self.data_handler);
	}

	fn handle_call_expression(&mut self, id: u32, node: &Call) {
		let expression = self.data_handler.load_call(node);

		self.do_assignment(id, expression);
	}

	fn handle_call(&mut self, id: u32, node: &Call) {
		if node.results == 0 || self.data_handler.get_local(Link(id, 0)).is_some() {
			self.handle_call_statement(id, node);
		} else {
			self.handle_call_expression(id, node);
		}

		let Call {
			ref arguments,
			results,
			states,
			..
		} = *node;

		for (&source, port) in arguments.iter().rev().zip((0..states).rev()) {
			let destination = Link(id, results + port);

			self.code_handler
				.do_rename(destination, source, &self.data_handler);
		}
	}

	fn handle_merge(&mut self, node: &Merge) {
		let Merge { states } = node;

		for &source in states {
			let _source = self.data_handler.load(source);
		}
	}

	fn handle_ref_is_null(&mut self, id: u32, node: RefIsNull) {
		let expression = self.data_handler.load_ref_is_null(node);

		self.do_assignment(id, expression);
	}

	fn handle_integer_unary_operation(&mut self, id: u32, node: IntegerUnaryOperation) {
		let expression = self.data_handler.load_integer_unary_operation(node);

		self.do_assignment(id, expression);
	}

	fn handle_integer_binary_operation(&mut self, id: u32, node: IntegerBinaryOperation) {
		let expression = self.data_handler.load_integer_binary_operation(node);

		self.do_assignment(id, expression);
	}

	fn handle_integer_compare_operation(&mut self, id: u32, node: IntegerCompareOperation) {
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

	fn handle_integer_extend(&mut self, id: u32, node: IntegerExtend) {
		let expression = self.data_handler.load_integer_extend(node);

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

	fn handle_number_unary_operation(&mut self, id: u32, node: NumberUnaryOperation) {
		let expression = self.data_handler.load_number_unary_operation(node);

		self.do_assignment(id, expression);
	}

	fn handle_number_binary_operation(&mut self, id: u32, node: NumberBinaryOperation) {
		let expression = self.data_handler.load_number_binary_operation(node);

		self.do_assignment(id, expression);
	}

	fn handle_number_compare_operation(&mut self, id: u32, node: NumberCompareOperation) {
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

	fn handle_global_new(&mut self, id: u32, node: GlobalNew) {
		let expression = self.data_handler.load_global_new(node);

		self.do_assignment(id, expression);
	}

	fn handle_global_get(&mut self, id: u32, node: GlobalGet) {
		let expression = self.data_handler.load_global_get(node);

		self.do_assignment(id, expression);

		self.code_handler.do_rename(
			Link(id, GlobalGet::STATE_PORT),
			node.source,
			&self.data_handler,
		);
	}

	fn handle_global_set(&mut self, id: u32, node: GlobalSet) {
		self.code_handler
			.do_global_set(node, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, GlobalSet::STATE_PORT),
			node.destination,
			&self.data_handler,
		);
	}

	fn handle_table_new(&mut self, id: u32, node: TableNew) {
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

	fn handle_table_init(&mut self, id: u32, node: TableInit) {
		self.code_handler
			.do_table_init(node, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, TableInit::DESTINATION_STATE_PORT),
			node.destination.reference,
			&self.data_handler,
		);

		self.code_handler.do_rename(
			Link(id, TableInit::SOURCE_STATE_PORT),
			node.source.reference,
			&self.data_handler,
		);
	}

	fn handle_elements_new(&mut self, id: u32, node: &ElementsNew) {
		let expression = self.data_handler.load_elements_new(node);

		self.do_assignment(id, expression);
	}

	fn handle_elements_drop(&mut self, id: u32, node: ElementsDrop) {
		self.code_handler
			.do_elements_drop(node, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, ElementsDrop::STATE_PORT),
			node.source,
			&self.data_handler,
		);
	}

	fn handle_memory_new(&mut self, id: u32, node: MemoryNew) {
		let expression = Expression::MemoryNew(node);

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

	fn handle_memory_init(&mut self, id: u32, node: MemoryInit) {
		self.code_handler
			.do_memory_init(node, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, MemoryInit::DESTINATION_STATE_PORT),
			node.destination.reference,
			&self.data_handler,
		);

		self.code_handler.do_rename(
			Link(id, MemoryInit::SOURCE_STATE_PORT),
			node.source.reference,
			&self.data_handler,
		);
	}

	fn handle_data_new(&mut self, id: u32, node: &DataNew) {
		let expression = Expression::DataNew(node.clone());

		self.do_assignment(id, expression);
	}

	fn handle_data_drop(&mut self, id: u32, node: DataDrop) {
		self.code_handler.do_data_drop(node, &mut self.data_handler);

		self.code_handler.do_rename(
			Link(id, DataDrop::STATE_PORT),
			node.source,
			&self.data_handler,
		);
	}

	fn handle_node(&mut self, graph: &DataFlowGraph, id: u32, node: &Node) {
		match *node {
			Node::GammaIn(_) => {}

			Node::LambdaIn(_) => self.handle_lambda_in(),
			Node::LambdaOut(ref node) => self.handle_lambda_out(graph, node),
			Node::RegionIn(ref node) => self.handle_region_in(graph, id, node),
			Node::RegionOut(_) => self.handle_region_out(id),
			Node::GammaOut(ref node) => self.handle_gamma_out(graph, node),
			Node::ThetaIn(ref node) => self.handle_theta_in(id, node),
			Node::ThetaOut(ref node) => self.handle_theta_out(node),
			Node::OmegaIn(_) => self.handle_omega_in(),
			Node::OmegaOut(ref node) => self.handle_omega_out(node),

			Node::Import(ref node) => self.handle_import(id, node),
			Node::Host(ref node) => self.handle_host(id, node.as_ref()),
			Node::Trap => self.handle_trap(id),
			Node::Null => self.handle_null(id),
			Node::Identity(node) => self.handle_identity(id, node),
			Node::I32(i32) => self.handle_i32_const(id, i32),
			Node::I64(i64) => self.handle_i64_const(id, i64),
			Node::F32(f32) => self.handle_f32_const(id, f32),
			Node::F64(f64) => self.handle_f64_const(id, f64),

			Node::Call(ref node) => self.handle_call(id, node),
			Node::Merge(ref node) => self.handle_merge(node),
			Node::RefIsNull(node) => self.handle_ref_is_null(id, node),
			Node::IntegerUnaryOperation(node) => self.handle_integer_unary_operation(id, node),
			Node::IntegerBinaryOperation(node) => self.handle_integer_binary_operation(id, node),
			Node::IntegerCompareOperation(node) => self.handle_integer_compare_operation(id, node),
			Node::IntegerNarrow(node) => self.handle_integer_narrow(id, node),
			Node::IntegerWiden(node) => self.handle_integer_widen(id, node),
			Node::IntegerExtend(node) => self.handle_integer_extend(id, node),
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
			Node::GlobalNew(node) => self.handle_global_new(id, node),
			Node::GlobalGet(node) => self.handle_global_get(id, node),
			Node::GlobalSet(node) => self.handle_global_set(id, node),
			Node::TableNew(node) => self.handle_table_new(id, node),
			Node::TableGet(node) => self.handle_table_get(id, node),
			Node::TableSet(node) => self.handle_table_set(id, node),
			Node::TableSize(node) => self.handle_table_size(id, node),
			Node::TableGrow(node) => self.handle_table_grow(id, node),
			Node::TableFill(node) => self.handle_table_fill(id, node),
			Node::TableCopy(node) => self.handle_table_copy(id, node),
			Node::TableInit(node) => self.handle_table_init(id, node),
			Node::ElementsNew(ref node) => self.handle_elements_new(id, node),
			Node::ElementsDrop(node) => self.handle_elements_drop(id, node),
			Node::MemoryNew(node) => self.handle_memory_new(id, node),
			Node::MemoryLoad(node) => self.handle_memory_load(id, node),
			Node::MemoryStore(node) => self.handle_memory_store(id, node),
			Node::MemorySize(node) => self.handle_memory_size(id, node),
			Node::MemoryGrow(node) => self.handle_memory_grow(id, node),
			Node::MemoryFill(node) => self.handle_memory_fill(id, node),
			Node::MemoryCopy(node) => self.handle_memory_copy(id, node),
			Node::MemoryInit(node) => self.handle_memory_init(id, node),
			Node::DataNew(ref node) => self.handle_data_new(id, node),
			Node::DataDrop(node) => self.handle_data_drop(id, node),
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
