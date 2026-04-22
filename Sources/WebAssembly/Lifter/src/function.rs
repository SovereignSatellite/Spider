use core::iter;

use wasmparser::{BlockType, FunctionBody, LocalsReader, OperatorsReader, ValType};

use ir_graph::{Link, Node, operation::Apply, region::Function};
use web_assembly_builder::{ControlFlowBuilder, Types};
use web_assembly_graph::ControlFlowGraph;
use web_assembly_liveness::{
	locals::{LocalTracker, Locals},
	references::{self, Reference},
};

use super::{control_flow::ControlFlowLifter, global_state::GlobalState};

/// A lifter-local value kind used only for tracking local-variable layout.
#[derive(Clone, Copy)]
pub enum LocalKind {
	/// A 32-bit integer local.
	I32,
	/// A 64-bit integer local.
	I64,
	/// A 32-bit floating-point local.
	F32,
	/// A 64-bit floating-point local.
	F64,
	/// A reference local.
	Reference,
}

fn classify(kind: ValType) -> LocalKind {
	match kind {
		ValType::I32 => LocalKind::I32,
		ValType::I64 => LocalKind::I64,
		ValType::F32 => LocalKind::F32,
		ValType::F64 => LocalKind::F64,
		ValType::Ref(_) => LocalKind::Reference,

		ValType::V128 => unimplemented!("`V128` types"),
	}
}

fn function_arity(function: u32, types: &Types) -> (u16, u16) {
	let function = types.get_type(function).unwrap_func();
	let arguments = u16::try_from(function.params().len()).unwrap();
	let results = u16::try_from(function.results().len()).unwrap();

	(arguments, results)
}

fn read_local_kinds_into(local_kinds: &mut Vec<LocalKind>, reader: LocalsReader<'_>) {
	local_kinds.clear();

	for (count, val_type) in reader.into_iter().map(Result::unwrap) {
		let kind = classify(val_type);
		let count = count.try_into().unwrap();

		local_kinds.extend(iter::repeat_n(kind, count));
	}
}

pub struct FunctionLifter {
	builder: ControlFlowBuilder,
	lifter: ControlFlowLifter,
	local_tracker: LocalTracker,

	graph: ControlFlowGraph,

	dependencies: Vec<Reference>,
	locals: Locals,
	local_kinds: Vec<LocalKind>,
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
			local_kinds: Vec::new(),
		}
	}

	pub fn build_data_flow(
		&mut self,
		nodes: &mut Vec<Node>,
		argument_count: u16,
		result_count: u16,
		global_state: &GlobalState,
	) -> Link {
		references::track(&mut self.dependencies, &self.graph.instructions);

		let captures = global_state.get_dependencies(&self.dependencies);
		let total_result_count = result_count
			.checked_add(1)
			.unwrap_or_else(|| unreachable!());
		let stack_size = self
			.local_tracker
			.run(&mut self.locals, &self.graph, total_result_count);
		let total_argument_count = argument_count
			.checked_add(1)
			.unwrap_or_else(|| unreachable!());

		Function::add_into(
			nodes,
			total_argument_count,
			captures,
			|inner_nodes, captures, arguments| {
				self.lifter.set_function_data(
					inner_nodes,
					captures,
					arguments,
					argument_count.into(),
					stack_size,
					&self.local_kinds,
					&self.dependencies,
				);

				self.lifter
					.run(inner_nodes, &self.graph, result_count.into(), &self.locals)
			},
		)
	}

	#[expect(
		clippy::too_many_arguments,
		reason = "lifter setup requires all parameters"
	)]
	pub fn build_function(
		&mut self,
		nodes: &mut Vec<Node>,
		body: &FunctionBody<'_>,
		function: u32,
		types: &Types,
		global_state: &GlobalState,
	) -> Link {
		let function = types.get_function_index(function);

		read_local_kinds_into(&mut self.local_kinds, body.get_locals_reader().unwrap());

		self.builder.run(
			&mut self.graph,
			types,
			BlockType::FuncType(function),
			self.local_kinds.len().try_into().unwrap(),
			body.get_operators_reader().unwrap(),
		);

		let (argument_count, result_count) = function_arity(function, types);

		self.build_data_flow(nodes, argument_count, result_count, global_state)
	}

	#[expect(
		clippy::too_many_arguments,
		reason = "lifter setup requires all parameters"
	)]
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

		self.local_kinds.clear();

		let function = self.build_data_flow(nodes, 0, 1, global_state);
		let apply = Apply::add_into(nodes, function, Vec::new(), 1);

		Link(apply, 0)
	}
}
