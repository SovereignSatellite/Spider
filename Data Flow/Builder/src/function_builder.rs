use alloc::vec::Vec;
use control_flow_builder::{ControlFlowBuilder, Types};
use control_flow_graph::ControlFlowGraph;
use control_flow_liveness::{
	locals::{LocalTracker, Locals},
	references::{self, Reference},
};
use data_flow_graph::{
	nested::{FunctionType, ValueType},
	DataFlowGraph, Link,
};
use list::resizable::Resizable;
use wasmparser::{BlockType, FunctionBody, LocalsReader, OperatorsReader, ValType};

use crate::{control_flow_converter::ControlFlowConverter, global_state::GlobalState};

fn web_type_to_data_type(r#type: ValType) -> ValueType {
	match r#type {
		ValType::I32 => ValueType::I32,
		ValType::I64 => ValueType::I64,
		ValType::F32 => ValueType::F32,
		ValType::F64 => ValueType::F64,
		ValType::Ref(_) => ValueType::Reference,

		ValType::V128 => unimplemented!("`V128` types"),
	}
}

fn load_type_from_function(function: u32, types: &Types) -> FunctionType {
	let r#type = types.get_type(function).unwrap_func();

	FunctionType {
		arguments: r#type
			.params()
			.iter()
			.copied()
			.map(web_type_to_data_type)
			.collect(),

		results: r#type
			.results()
			.iter()
			.copied()
			.map(web_type_to_data_type)
			.collect(),
	}
}

fn load_type_from_result(result: ValType) -> FunctionType {
	let result = web_type_to_data_type(result);

	FunctionType {
		arguments: Resizable::new(),
		results: core::iter::once(result).collect(),
	}
}

fn read_local_types_into(local_types: &mut Vec<ValueType>, reader: LocalsReader) {
	local_types.clear();

	for (count, r#type) in reader.into_iter().map(Result::unwrap) {
		let r#type = web_type_to_data_type(r#type);
		let count = count.try_into().unwrap();

		local_types.extend(core::iter::repeat_n(r#type, count));
	}
}

pub struct FunctionBuilder {
	converter: ControlFlowConverter,
	builder: ControlFlowBuilder,
	local_finder: LocalTracker,

	graph: ControlFlowGraph,

	local_types: Vec<ValueType>,
	locals: Locals,
	dependencies: Vec<Reference>,
}

impl FunctionBuilder {
	pub const fn new() -> Self {
		Self {
			converter: ControlFlowConverter::new(),
			builder: ControlFlowBuilder::new(),
			local_finder: LocalTracker::new(),

			graph: ControlFlowGraph::new(),

			dependencies: Vec::new(),
			locals: Locals::new(),
			local_types: Vec::new(),
		}
	}

	pub fn build_data_flow(
		&mut self,
		graph: &mut DataFlowGraph,
		r#type: FunctionType,
		global_state: &GlobalState,
	) -> u32 {
		references::track(&mut self.dependencies, &self.graph.instructions);

		let dependencies = global_state.get_dependencies(&self.dependencies);
		let stack_size = self.local_finder.run(
			&mut self.locals,
			&self.graph,
			r#type.results.len().try_into().unwrap(),
		);

		let lambda_in = graph.add_lambda_in(r#type.into(), dependencies);

		self.converter.set_function_data(
			graph,
			lambda_in,
			stack_size,
			&self.local_types,
			&self.dependencies,
		);

		let results = self
			.converter
			.run(graph, &self.graph, lambda_in, &self.locals);

		graph.add_lambda_out(lambda_in, results)
	}

	pub fn build_function(
		&mut self,
		graph: &mut DataFlowGraph,
		body: &FunctionBody,
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
		operators: OperatorsReader,
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
		let call = graph.add_call(Link(function, 0), Vec::new(), 1, 0);

		Link(call, 0)
	}
}
