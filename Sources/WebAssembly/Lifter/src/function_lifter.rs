use alloc::vec::Vec;
use ir_graph::{
	DataFlowGraph, Link,
	control::{FunctionType, LambdaIn, LambdaOut, ValueType},
	simple::Apply,
};
use list::resizable::Resizable;
use wasmparser::{BlockType, FunctionBody, LocalsReader, OperatorsReader, ValType};
use web_assembly_builder::{ControlFlowBuilder, Types};
use web_assembly_graph::ControlFlowGraph;
use web_assembly_liveness::{
	locals::{LocalTracker, Locals},
	references::{self, Reference},
};

use crate::{control_flow_lifter::ControlFlowLifter, global_state::GlobalState};

fn web_type_to_data_type(kind: ValType) -> ValueType {
	match kind {
		ValType::I32 => ValueType::I32,
		ValType::I64 => ValueType::I64,
		ValType::F32 => ValueType::F32,
		ValType::F64 => ValueType::F64,
		ValType::Ref(_) => ValueType::Reference,

		ValType::V128 => unimplemented!("`V128` types"),
	}
}

fn load_type_from_function(function: u32, types: &Types) -> FunctionType {
	fn load_types(types: &[ValType]) -> Resizable<ValueType, 15> {
		types.iter().copied().map(web_type_to_data_type).collect()
	}

	let function = types.get_type(function).unwrap_func();

	FunctionType {
		arguments: load_types(function.params()),
		results: load_types(function.results()),
	}
}

fn load_type_from_result(result: ValType) -> FunctionType {
	let result = web_type_to_data_type(result);

	FunctionType {
		arguments: Resizable::new(),
		results: list::resizable![result],
	}
}

fn read_local_types_into(local_types: &mut Vec<ValueType>, reader: LocalsReader<'_>) {
	local_types.clear();

	for (count, val_type) in reader.into_iter().map(Result::unwrap) {
		let val_type = web_type_to_data_type(val_type);
		let count = count.try_into().unwrap();

		local_types.extend(core::iter::repeat_n(val_type, count));
	}
}

pub struct FunctionLifter {
	builder: ControlFlowBuilder,
	lifter: ControlFlowLifter,
	local_tracker: LocalTracker,

	graph: ControlFlowGraph,

	dependencies: Vec<Reference>,
	locals: Locals,
	local_types: Vec<ValueType>,
}

impl FunctionLifter {
	pub const fn new() -> Self {
		Self {
			builder: ControlFlowBuilder::new(),
			lifter: ControlFlowLifter::new(),
			local_tracker: LocalTracker::new(),

			graph: ControlFlowGraph::new(),

			dependencies: Vec::new(),
			locals: Locals::new(),
			local_types: Vec::new(),
		}
	}

	pub fn build_data_flow(
		&mut self,
		graph: &mut DataFlowGraph,
		kind: FunctionType,
		global_state: &GlobalState,
	) -> u32 {
		references::track(&mut self.dependencies, &self.graph.instructions);

		let dependencies = global_state.get_dependencies(&self.dependencies);
		let stack_size = self.local_tracker.run(
			&mut self.locals,
			&self.graph,
			kind.results.len().try_into().unwrap(),
		);

		let lambda_in = LambdaIn::add_into(graph, kind.into(), dependencies);

		self.lifter.set_function_data(
			graph,
			lambda_in,
			stack_size,
			&self.local_types,
			&self.dependencies,
		);

		let results = self.lifter.run(graph, &self.graph, lambda_in, &self.locals);

		let LambdaIn {
			kind: lambda_type, ..
		} = graph.get_mut(lambda_in).as_mut_lambda_in().unwrap();

		// We add a "trap state" as part of the function signature
		lambda_type.arguments.push(ValueType::Reference);
		lambda_type.results.push(ValueType::Reference);

		LambdaOut::add_into(graph, lambda_in, results)
	}

	pub fn build_function(
		&mut self,
		graph: &mut DataFlowGraph,
		body: &FunctionBody<'_>,
		function: u32,
		types: &Types,
		global_state: &GlobalState,
	) -> u32 {
		let function = types.get_function_index(function);

		read_local_types_into(&mut self.local_types, body.get_locals_reader().unwrap());

		self.builder.run(
			&mut self.graph,
			types,
			BlockType::FuncType(function),
			self.local_types.len().try_into().unwrap(),
			body.get_operators_reader().unwrap(),
		);

		let function_type = load_type_from_function(function, types);

		self.build_data_flow(graph, function_type, global_state)
	}

	pub fn build_expression(
		&mut self,
		graph: &mut DataFlowGraph,
		operators: OperatorsReader<'_>,
		result: ValType,
		types: &Types,
		global_state: &GlobalState,
	) -> Link {
		self.builder.run(
			&mut self.graph,
			types,
			BlockType::Type(result),
			0,
			operators,
		);

		self.local_types.clear();

		let function_type = load_type_from_result(result);
		let function = self.build_data_flow(graph, function_type, global_state);
		let apply = Apply::add_into(graph, Link(function, 0), Vec::new(), 1);

		Link(apply, 0)
	}
}
