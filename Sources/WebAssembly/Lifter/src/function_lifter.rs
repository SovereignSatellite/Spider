use list::resizable::Resizable;
use wasmparser::{BlockType, FunctionBody, LocalsReader, OperatorsReader, ValType};

use ir_graph::{
	Link, Node,
	control::{Function, ValueType},
	simple::Apply,
};
use web_assembly_builder::{ControlFlowBuilder, Types};
use web_assembly_graph::ControlFlowGraph;
use web_assembly_liveness::{
	locals::{LocalTracker, Locals},
	references::{self, Reference},
};

use super::{control_flow_lifter::ControlFlowLifter, global_state::GlobalState};

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

fn load_type_from_function(
	function: u32,
	types: &Types,
) -> (Resizable<ValueType, 15>, Resizable<ValueType, 15>) {
	fn load_types(types: &[ValType]) -> Resizable<ValueType, 15> {
		types.iter().copied().map(web_type_to_data_type).collect()
	}

	let function = types.get_type(function).unwrap_func();

	(
		load_types(function.params()),
		load_types(function.results()),
	)
}

fn load_type_from_result(result: ValType) -> (Resizable<ValueType, 15>, Resizable<ValueType, 15>) {
	let result = web_type_to_data_type(result);

	(Resizable::new(), list::resizable![result])
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
		nodes: &mut Vec<Node>,
		mut argument_types: Resizable<ValueType, 15>,
		mut result_types: Resizable<ValueType, 15>,
		global_state: &GlobalState,
	) -> Link {
		references::track(&mut self.dependencies, &self.graph.instructions);

		let captures = global_state.get_dependencies(&self.dependencies);
		let stack_size = self.local_tracker.run(
			&mut self.locals,
			&self.graph,
			result_types.len().try_into().unwrap(),
		);

		let argument_count = argument_types.len();
		let result_count = result_types.len();

		// We add a "trap state" as part of the function signature
		argument_types.push(ValueType::Reference);
		result_types.push(ValueType::Reference);

		Function::add_into(
			nodes,
			argument_types,
			result_types,
			captures,
			|inner_nodes, captures, arguments| {
				self.lifter.set_function_data(
					inner_nodes,
					captures,
					arguments,
					argument_count,
					stack_size,
					&self.local_types,
					&self.dependencies,
				);

				self.lifter
					.run(inner_nodes, &self.graph, result_count, &self.locals)
			},
		)
	}

	pub fn build_function(
		&mut self,
		nodes: &mut Vec<Node>,
		body: &FunctionBody<'_>,
		function: u32,
		types: &Types,
		global_state: &GlobalState,
	) -> Link {
		let function = types.get_function_index(function);

		read_local_types_into(&mut self.local_types, body.get_locals_reader().unwrap());

		self.builder.run(
			&mut self.graph,
			types,
			BlockType::FuncType(function),
			self.local_types.len().try_into().unwrap(),
			body.get_operators_reader().unwrap(),
		);

		let (argument_types, result_types) = load_type_from_function(function, types);

		self.build_data_flow(nodes, argument_types, result_types, global_state)
	}

	pub fn build_expression(
		&mut self,
		nodes: &mut Vec<Node>,
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

		let (argument_types, result_types) = load_type_from_result(result);
		let function = self.build_data_flow(nodes, argument_types, result_types, global_state);
		let apply = Apply::add_into(nodes, function, Vec::new(), 1);

		Link(apply, 0)
	}
}
