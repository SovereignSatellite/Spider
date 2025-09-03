#![no_std]

use alloc::vec::Vec;
use data_flow_graph::{
	DataFlowGraph, Link, Node,
	mvp::{
		Call, DataDrop, DataNew, ElementsDrop, ElementsNew, GlobalGet, GlobalNew, GlobalSet, Host,
		Identity, IntegerBinaryOperation, IntegerCompareOperation, IntegerConvertToNumber,
		IntegerExtend, IntegerNarrow, IntegerTransmuteToNumber, IntegerUnaryOperation,
		IntegerWiden, MemoryCopy, MemoryFill, MemoryGrow, MemoryInit, MemoryLoad, MemoryNew,
		MemorySize, MemoryStore, Merge, NumberBinaryOperation, NumberCompareOperation,
		NumberNarrow, NumberTransmuteToInteger, NumberTruncateToInteger, NumberUnaryOperation,
		NumberWiden, RefIsNull, TableCopy, TableFill, TableGet, TableGrow, TableInit, TableNew,
		TableSet, TableSize,
	},
	nested::{
		GammaIn, GammaOut, Import, LambdaIn, LambdaOut, OmegaIn, OmegaOut, RegionOut, ThetaIn,
		ThetaOut,
	},
};
use hashbrown::HashMap;
use luau_tree::{
	LuauTree,
	expression::{Expression, Local, Name},
	statement::{AssignAll, Export, FastDefine, Sequence},
};

use self::{
	code_handler::CodeHandler,
	data_handler::DataHandler,
	local_allocator::LocalAllocator,
	place::{Place, Table},
	reference_finder::ReferenceFinder,
};

extern crate alloc;

mod code_handler;
mod data_handler;
mod local_allocator;
mod place;
mod reference_finder;
mod scoped_provider;

pub struct LuauBuilder {
	reference_finder: ReferenceFinder,
	local_allocator: LocalAllocator,
	locals: HashMap<Link, Place>,
	tables: HashMap<u32, Table>,

	code_handler: CodeHandler,
	data_handler: DataHandler,

	environment: Name,
	exports: Vec<Export>,

	regions: HashMap<u32, Sequence>,
}

impl LuauBuilder {
	#[must_use]
	pub fn new() -> Self {
		Self {
			reference_finder: ReferenceFinder::new(),
			local_allocator: LocalAllocator::new(),
			locals: HashMap::new(),
			tables: HashMap::new(),

			code_handler: CodeHandler::new(),
			data_handler: DataHandler::new(),

			environment: Name { id: 0 },
			exports: Vec::new(),

			regions: HashMap::new(),
		}
	}

	fn do_fast_definition(&mut self, link: Link, name: Name, expression: Expression) {
		let local = Local::Fast { name };

		self.data_handler.alias(link, local);
		self.code_handler.do_fast_define(name, expression);
	}

	fn do_fast_assignment(&mut self, link: Link, name: Name, expression: Expression) {
		let local = Local::Fast { name };

		self.data_handler.alias(link, local);
		self.code_handler.do_assign(local, expression);
	}

	fn do_slow_assignment(&mut self, link: Link, table: Name, index: u16, expression: Expression) {
		let local = Local::Slow { table, index };

		self.data_handler.alias(link, local);
		self.code_handler.do_assign(local, expression);
	}

	fn do_place_expression(&mut self, link: Link, place: Place, expression: Expression) {
		match place {
			Place::Definition { name } => self.do_fast_definition(link, name, expression),
			Place::Assignment { name } => self.do_fast_assignment(link, name, expression),
			Place::Overflow { table, index } => {
				self.do_slow_assignment(link, table, index, expression);
			}
		}
	}

	fn do_set_expression(&mut self, id: u32, expression: Expression) {
		if let Some(&place) = self.locals.get(&Link(id, 0)) {
			self.do_place_expression(Link(id, 0), place, expression);
		} else {
			self.data_handler.define(id, expression);
		}
	}

	fn do_rename(&mut self, from: Link, to: Link) {
		let from = self.data_handler.load_local(from).unwrap();

		self.data_handler.alias(to, from);
	}

	fn handle_table_spill(&mut self, id: u32) {
		let Some(table) = self.tables.get(&id) else {
			return;
		};

		self.code_handler.do_slow_define(table.name, table.len);
	}

	fn handle_lambda_in(&mut self, id: u32, lambda_in: &LambdaIn) {
		const TRAP_LOCAL: Local = Local::Fast {
			name: Name { id: u32::MAX },
		};

		self.data_handler.push_scope();
		self.code_handler.push_scope();

		for link in lambda_in.output_ports().map(|port| Link(id, port)) {
			let name = self.locals[&link].into_definition();

			self.data_handler.alias(link, Local::Fast { name });
		}

		let trap = lambda_in.output_ports().end;

		self.data_handler.alias(Link(id, trap), TRAP_LOCAL);

		self.handle_table_spill(id);
	}

	fn handle_lambda_out(&mut self, graph: &DataFlowGraph, id: u32, lambda_out: &LambdaOut) {
		let LambdaOut { results, input } = lambda_out;
		let lambda_in @ LambdaIn {
			r#type,
			dependencies,
			..
		} = graph.get(*input).as_lambda_in().unwrap();

		let arguments = lambda_in
			.argument_ports()
			.map(|port| Link(*input, port))
			.map(|link| self.locals[&link])
			.map(Place::into_definition)
			.collect();

		let mut returns = self.data_handler.load_locals(results);

		returns.truncate(r#type.results.len());

		self.data_handler.pop_scope();

		let code = self.code_handler.pop_scope();

		let dependencies = self.data_handler.load_sources(dependencies);
		let dependencies = lambda_in
			.dependency_ports()
			.map(|port| Link(*input, port))
			.map(|link| self.locals[&link])
			.map(Place::into_definition)
			.zip(dependencies)
			.map(|(name, source)| FastDefine { name, source })
			.collect();

		let function = DataHandler::load_scoped(dependencies, arguments, code, returns);

		self.do_set_expression(id, function);
	}

	fn handle_region_in(&mut self) {
		self.data_handler.push_scope();
		self.code_handler.push_scope();
	}

	fn load_assignment_list(&mut self, id: u32, sources: &[Link]) -> Vec<(Local, Local)> {
		let destinations = (0..sources.len().try_into().unwrap())
			.map(|port| Link(id, port))
			.map(|link| self.locals[&link])
			.map(Place::into_local);

		let sources = self.data_handler.load_locals(sources).into_iter();

		let mut assignments: Vec<_> = destinations.zip(sources).collect();

		assignments.retain(|(destination, source)| destination != source);
		assignments.sort_unstable();

		assignments
	}

	fn handle_region_out(&mut self, id: u32, region_out: &RegionOut) {
		let RegionOut {
			output, results, ..
		} = region_out;

		let assignments = self.load_assignment_list(*output, results);

		self.code_handler.do_assign_all(assignments);
		self.data_handler.pop_scope();

		let code = self.code_handler.pop_scope();

		self.regions.insert(id, code);
	}

	fn handle_gamma_in(&mut self, graph: &DataFlowGraph, gamma_in: &GammaIn) {
		let GammaIn {
			output, arguments, ..
		} = gamma_in;

		let GammaOut { regions, .. } = graph.get(*output).as_gamma_out().unwrap();

		self.data_handler.push_scope();

		for &region in regions {
			let RegionOut { input, .. } = *graph.get(region).as_region_out().unwrap();

			for (to, &from) in (0..u16::MAX).map(|port| Link(input, port)).zip(arguments) {
				self.do_rename(from, to);
			}
		}
	}

	fn do_definition_list(&mut self, id: u32, count: u16) {
		for link in (0..count).map(|port| Link(id, port)) {
			let place = self.locals[&link];

			self.data_handler.alias(link, place.into_local());

			if let Place::Definition { name } = place {
				self.code_handler.do_fast_define(name, Expression::Null);
			}
		}
	}

	fn handle_gamma_out(&mut self, graph: &DataFlowGraph, id: u32, gamma_out: &GammaOut) {
		let GammaOut { input, regions } = gamma_out;
		let GammaIn { condition, .. } = graph.get(*input).as_gamma_in().unwrap();

		let condition = self.data_handler.load(*condition).unwrap();

		let branches = regions
			.iter()
			.map(|id| self.regions.remove(id).unwrap())
			.collect();

		self.data_handler.pop_scope();

		let count = graph.get(regions[0]).as_region_out().unwrap().results.len();

		self.do_definition_list(id, count.try_into().unwrap());

		self.code_handler.do_match(branches, condition);
	}

	fn handle_theta_in(&mut self, id: u32, theta_in: &ThetaIn) {
		let arguments = self.data_handler.load_sources(&theta_in.arguments);

		self.data_handler.push_scope();

		for (link, argument) in (0..u16::MAX).map(|port| Link(id, port)).zip(arguments) {
			let place = self.locals[&link];

			self.do_place_expression(link, place, argument);
		}

		self.code_handler.push_scope();
	}

	fn handle_theta_out(&mut self, id: u32, theta_out: &ThetaOut) {
		let ThetaOut {
			input,
			condition,
			results,
		} = theta_out;

		let condition = self.data_handler.load(*condition).unwrap();

		let post = AssignAll {
			assignments: self.load_assignment_list(*input, results),
		};

		self.data_handler.pop_scope();

		let code = self.code_handler.pop_scope();

		self.code_handler.do_repeat(code, post, condition);

		for (from, to) in
			(0..results.len().try_into().unwrap()).map(|port| (Link(*input, port), Link(id, port)))
		{
			let from = self.locals[&from].into_local();

			self.data_handler.alias(to, from);
		}
	}

	fn handle_omega_in(&mut self, id: u32) {
		let environment = Link(id, OmegaIn::ENVIRONMENT_PORT);
		let state = Link(id, OmegaIn::STATE_PORT);

		self.environment = self.locals.get(&environment).unwrap().into_definition();

		let local = Local::Fast {
			name: self.environment,
		};

		self.data_handler.alias(environment, local);
		self.data_handler.alias(state, local);

		self.handle_table_spill(id);
	}

	fn handle_omega_out(&mut self, omega_out: &OmegaOut) {
		let exports = omega_out
			.exports
			.iter()
			.map(|export| self.data_handler.load_export(export));

		self.exports.extend(exports);
	}

	fn handle_import(&mut self, id: u32, import: &Import) {
		let import = self.data_handler.load_import(import);

		self.do_set_expression(id, import);
	}

	#[expect(clippy::unused_self, clippy::needless_pass_by_ref_mut)]
	fn handle_host(&mut self, id: u32, host: &dyn Host) {
		panic!("unknown host operation {id} `{}`", host.identifier());
	}

	fn handle_trap(&mut self, id: u32) {
		self.do_set_expression(id, Expression::Trap);
	}

	fn handle_null(&mut self, id: u32) {
		self.do_set_expression(id, Expression::Null);
	}

	fn handle_identity(&mut self, id: u32, identity: Identity) {
		let identity = self.data_handler.load_identity(identity);

		self.do_set_expression(id, identity);
	}

	fn handle_i32_const(&mut self, id: u32, value: i32) {
		self.do_set_expression(id, Expression::I32(value));
	}

	fn handle_i64_const(&mut self, id: u32, value: i64) {
		self.do_set_expression(id, Expression::I64(value));
	}

	fn handle_f32_const(&mut self, id: u32, value: f32) {
		self.do_set_expression(id, Expression::F32(value));
	}

	fn handle_f64_const(&mut self, id: u32, value: f64) {
		self.do_set_expression(id, Expression::F64(value));
	}

	fn handle_call_statement(&mut self, id: u32, call: &Call) {
		let results = (0..call.results)
			.map(|port| Link(id, port))
			.map(|link| self.locals[&link])
			.map(Place::into_local)
			.collect();

		self.do_definition_list(id, call.results);

		self.code_handler
			.do_call(call, results, &mut self.data_handler);
	}

	fn handle_call_expression(&mut self, id: u32, call: &Call) {
		let call = self.data_handler.load_call(call);

		self.do_set_expression(id, call);
	}

	fn handle_call(&mut self, id: u32, call: &Call) {
		if call.results == 0 || self.locals.contains_key(&Link(id, 0)) {
			self.handle_call_statement(id, call);
		} else {
			self.handle_call_expression(id, call);
		}

		let Call {
			arguments,
			results,
			states,
			..
		} = call;

		for (&argument, port) in arguments.iter().rev().zip((0..*states).rev()) {
			self.do_rename(argument, Link(id, *results + port));
		}
	}

	fn handle_merge(&mut self, merge: &Merge) {
		let Merge { states } = merge;

		for &source in states {
			self.data_handler.load(source).expect("state should exist");
		}
	}

	fn handle_ref_is_null(&mut self, id: u32, ref_is_null: RefIsNull) {
		let ref_is_null = self.data_handler.load_ref_is_null(ref_is_null);

		self.do_set_expression(id, ref_is_null);
	}

	fn handle_integer_unary_operation(&mut self, id: u32, operation: IntegerUnaryOperation) {
		let operation = self.data_handler.load_integer_unary_operation(operation);

		self.do_set_expression(id, operation);
	}

	fn handle_integer_binary_operation(&mut self, id: u32, operation: IntegerBinaryOperation) {
		let operation = self.data_handler.load_integer_binary_operation(operation);

		self.do_set_expression(id, operation);
	}

	fn handle_integer_compare_operation(&mut self, id: u32, operation: IntegerCompareOperation) {
		let operation = self.data_handler.load_integer_compare_operation(operation);

		self.do_set_expression(id, operation);
	}

	fn handle_integer_narrow(&mut self, id: u32, operation: IntegerNarrow) {
		let operation = self.data_handler.load_integer_narrow(operation);

		self.do_set_expression(id, operation);
	}

	fn handle_integer_widen(&mut self, id: u32, operation: IntegerWiden) {
		let operation = self.data_handler.load_integer_widen(operation);

		self.do_set_expression(id, operation);
	}

	fn handle_integer_extend(&mut self, id: u32, operation: IntegerExtend) {
		let operation = self.data_handler.load_integer_extend(operation);

		self.do_set_expression(id, operation);
	}

	fn handle_integer_convert_to_number(&mut self, id: u32, operation: IntegerConvertToNumber) {
		let operation = self.data_handler.load_integer_convert_to_number(operation);

		self.do_set_expression(id, operation);
	}

	fn handle_integer_transmute_to_number(&mut self, id: u32, operation: IntegerTransmuteToNumber) {
		let operation = self
			.data_handler
			.load_integer_transmute_to_number(operation);

		self.do_set_expression(id, operation);
	}

	fn handle_number_unary_operation(&mut self, id: u32, operation: NumberUnaryOperation) {
		let operation = self.data_handler.load_number_unary_operation(operation);

		self.do_set_expression(id, operation);
	}

	fn handle_number_binary_operation(&mut self, id: u32, operation: NumberBinaryOperation) {
		let operation = self.data_handler.load_number_binary_operation(operation);

		self.do_set_expression(id, operation);
	}

	fn handle_number_compare_operation(&mut self, id: u32, operation: NumberCompareOperation) {
		let operation = self.data_handler.load_number_compare_operation(operation);

		self.do_set_expression(id, operation);
	}

	fn handle_number_narrow(&mut self, id: u32, operation: NumberNarrow) {
		let operation = self.data_handler.load_number_narrow(operation);

		self.do_set_expression(id, operation);
	}

	fn handle_number_widen(&mut self, id: u32, operation: NumberWiden) {
		let operation = self.data_handler.load_number_widen(operation);

		self.do_set_expression(id, operation);
	}

	fn handle_number_truncate_to_integer(&mut self, id: u32, operation: NumberTruncateToInteger) {
		let operation = self.data_handler.load_number_truncate_to_integer(operation);

		self.do_set_expression(id, operation);
	}

	fn handle_number_transmute_to_integer(&mut self, id: u32, operation: NumberTransmuteToInteger) {
		let operation = self
			.data_handler
			.load_number_transmute_to_integer(operation);

		self.do_set_expression(id, operation);
	}

	fn handle_global_new(&mut self, id: u32, global_new: GlobalNew) {
		let global_new = self.data_handler.load_global_new(global_new);

		self.do_set_expression(id, global_new);
	}

	fn handle_global_get(&mut self, id: u32, global_get: GlobalGet) {
		let result = self.data_handler.load_global_get(global_get);

		self.do_set_expression(id, result);
		self.do_rename(global_get.source, Link(id, GlobalGet::STATE_PORT));
	}

	fn handle_global_set(&mut self, id: u32, global_set: GlobalSet) {
		self.code_handler
			.do_global_set(global_set, &mut self.data_handler);

		self.do_rename(global_set.destination, Link(id, GlobalSet::STATE_PORT));
	}

	fn handle_table_new(&mut self, id: u32, table_new: TableNew) {
		let table_new = self.data_handler.load_table_new(table_new);

		self.do_set_expression(id, table_new);
	}

	fn handle_table_get(&mut self, id: u32, table_get: TableGet) {
		let result = self.data_handler.load_table_get(table_get);

		self.do_set_expression(id, result);
		self.do_rename(table_get.source.reference, Link(id, TableGet::STATE_PORT));
	}

	fn handle_table_set(&mut self, id: u32, table_set: TableSet) {
		self.code_handler
			.do_table_set(table_set, &mut self.data_handler);

		self.do_rename(
			table_set.destination.reference,
			Link(id, TableSet::STATE_PORT),
		);
	}

	fn handle_table_size(&mut self, id: u32, table_size: TableSize) {
		let result = self.data_handler.load_table_size(table_size);

		self.do_set_expression(id, result);
		self.do_rename(table_size.source, Link(id, TableSize::STATE_PORT));
	}

	fn handle_table_grow(&mut self, id: u32, table_grow: TableGrow) {
		let result = self.data_handler.load_table_grow(table_grow);

		self.do_set_expression(id, result);

		self.do_rename(table_grow.destination, Link(id, MemoryGrow::STATE_PORT));
	}

	fn handle_table_fill(&mut self, id: u32, table_fill: TableFill) {
		self.code_handler
			.do_table_fill(table_fill, &mut self.data_handler);

		self.do_rename(
			table_fill.destination.reference,
			Link(id, TableFill::STATE_PORT),
		);
	}

	fn handle_table_copy(&mut self, id: u32, table_copy: TableCopy) {
		self.code_handler
			.do_table_copy(table_copy, &mut self.data_handler);

		self.do_rename(
			table_copy.destination.reference,
			Link(id, TableCopy::DESTINATION_STATE_PORT),
		);

		self.do_rename(
			table_copy.source.reference,
			Link(id, TableCopy::SOURCE_STATE_PORT),
		);
	}

	fn handle_table_init(&mut self, id: u32, table_init: TableInit) {
		self.code_handler
			.do_table_init(table_init, &mut self.data_handler);

		self.do_rename(
			table_init.destination.reference,
			Link(id, TableInit::DESTINATION_STATE_PORT),
		);

		self.do_rename(
			table_init.source.reference,
			Link(id, TableInit::SOURCE_STATE_PORT),
		);
	}

	fn handle_elements_new(&mut self, id: u32, elements_new: &ElementsNew) {
		let elements_new = self.data_handler.load_elements_new(elements_new);

		self.do_set_expression(id, elements_new);
	}

	fn handle_elements_drop(&mut self, id: u32, elements_drop: ElementsDrop) {
		self.code_handler
			.do_elements_drop(elements_drop, &mut self.data_handler);

		self.do_rename(elements_drop.source, Link(id, ElementsDrop::STATE_PORT));
	}

	fn handle_memory_new(&mut self, id: u32, memory_new: MemoryNew) {
		let memory_new = Expression::MemoryNew(memory_new);

		self.do_set_expression(id, memory_new);
	}

	fn handle_memory_load(&mut self, id: u32, memory_load: MemoryLoad) {
		let result = self.data_handler.load_memory_load(memory_load);

		self.do_set_expression(id, result);

		self.do_rename(
			memory_load.source.reference,
			Link(id, MemoryLoad::STATE_PORT),
		);
	}

	fn handle_memory_store(&mut self, id: u32, memory_store: MemoryStore) {
		self.code_handler
			.do_memory_store(memory_store, &mut self.data_handler);

		self.do_rename(
			memory_store.destination.reference,
			Link(id, MemoryStore::STATE_PORT),
		);
	}

	fn handle_memory_size(&mut self, id: u32, memory_size: MemorySize) {
		let result = self.data_handler.load_memory_size(memory_size);

		self.do_set_expression(id, result);
		self.do_rename(memory_size.source, Link(id, MemorySize::STATE_PORT));
	}

	fn handle_memory_grow(&mut self, id: u32, memory_grow: MemoryGrow) {
		let result = self.data_handler.load_memory_grow(memory_grow);

		self.do_set_expression(id, result);

		self.do_rename(memory_grow.destination, Link(id, MemoryGrow::STATE_PORT));
	}

	fn handle_memory_fill(&mut self, id: u32, memory_fill: MemoryFill) {
		self.code_handler
			.do_memory_fill(memory_fill, &mut self.data_handler);

		self.do_rename(
			memory_fill.destination.reference,
			Link(id, MemoryFill::STATE_PORT),
		);
	}

	fn handle_memory_copy(&mut self, id: u32, memory_copy: MemoryCopy) {
		self.code_handler
			.do_memory_copy(memory_copy, &mut self.data_handler);

		self.do_rename(
			memory_copy.destination.reference,
			Link(id, MemoryCopy::DESTINATION_STATE_PORT),
		);

		self.do_rename(
			memory_copy.source.reference,
			Link(id, MemoryCopy::SOURCE_STATE_PORT),
		);
	}

	fn handle_memory_init(&mut self, id: u32, memory_init: MemoryInit) {
		self.code_handler
			.do_memory_init(memory_init, &mut self.data_handler);

		self.do_rename(
			memory_init.destination.reference,
			Link(id, MemoryInit::DESTINATION_STATE_PORT),
		);

		self.do_rename(
			memory_init.source.reference,
			Link(id, MemoryInit::SOURCE_STATE_PORT),
		);
	}

	fn handle_data_new(&mut self, id: u32, data_new: &DataNew) {
		let data_new = Expression::DataNew(data_new.clone());

		self.do_set_expression(id, data_new);
	}

	fn handle_data_drop(&mut self, id: u32, data_drop: DataDrop) {
		self.code_handler
			.do_data_drop(data_drop, &mut self.data_handler);

		self.do_rename(data_drop.source, Link(id, DataDrop::STATE_PORT));
	}

	fn handle_node(&mut self, graph: &DataFlowGraph, id: u32, node: &Node) {
		match *node {
			Node::LambdaIn(ref lambda_in) => self.handle_lambda_in(id, lambda_in),
			Node::LambdaOut(ref lambda_out) => self.handle_lambda_out(graph, id, lambda_out),
			Node::RegionIn(_) => self.handle_region_in(),
			Node::RegionOut(ref region_out) => self.handle_region_out(id, region_out),
			Node::GammaIn(ref gamma_in) => self.handle_gamma_in(graph, gamma_in),
			Node::GammaOut(ref gamma_out) => self.handle_gamma_out(graph, id, gamma_out),
			Node::ThetaIn(ref theta_in) => self.handle_theta_in(id, theta_in),
			Node::ThetaOut(ref theta_out) => self.handle_theta_out(id, theta_out),
			Node::OmegaIn(_) => self.handle_omega_in(id),
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
		self.reference_finder.run(graph);
		self.local_allocator.run(
			&mut self.tables,
			&mut self.locals,
			graph,
			&self.reference_finder,
		);

		for (node, id) in graph.nodes().zip(0..) {
			self.handle_node(graph, id, node);
		}

		self.data_handler.pop_scope();

		LuauTree {
			environment: self.environment,
			code: self.code_handler.pop_scope(),
			exports: core::mem::take(&mut self.exports),
		}
	}
}

impl Default for LuauBuilder {
	fn default() -> Self {
		Self::new()
	}
}
