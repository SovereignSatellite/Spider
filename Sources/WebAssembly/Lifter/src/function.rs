use core::iter;

use wasmparser::{BlockType, FunctionBody, LocalsReader, ValType};

use ir_graph::{Link, Node, operation::Aggregate, region::Function};
use web_assembly_builder::{ControlFlowBuilder, Types};
use web_assembly_graph::{ControlFlowGraph, instruction::Reference};
use web_assembly_liveness::{
	locals::{LocalTracker, Locals},
	references,
};

use super::{entities::Entities, interval::IntervalLifter, slots::FunctionHeader};

#[derive(Clone, Copy)]
pub enum LocalKind {
	I32,
	I64,
	F32,
	F64,
	Reference,
}

impl LocalKind {
	fn classify(kind: ValType) -> Self {
		match kind {
			ValType::I32 => Self::I32,
			ValType::I64 => Self::I64,
			ValType::F32 => Self::F32,
			ValType::F64 => Self::F64,
			ValType::V128 => unimplemented!("`V128` types"),
			ValType::Ref(_) => Self::Reference,
		}
	}

	fn read_into(kinds: &mut Vec<Self>, reader: LocalsReader<'_>) {
		kinds.clear();

		for (count, value_type) in reader.into_iter().map(Result::unwrap) {
			let kind = Self::classify(value_type);
			let count = count.try_into().unwrap();

			kinds.extend(iter::repeat_n(kind, count));
		}
	}
}

pub struct FunctionLifter {
	builder: ControlFlowBuilder,
	lifter: IntervalLifter,
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
			lifter: IntervalLifter::new(),
			local_tracker: LocalTracker::new(),

			graph: ControlFlowGraph::new(),

			dependencies: Vec::new(),
			locals: Locals::new(),
			local_kinds: Vec::new(),
		}
	}

	fn build_data_flow(
		&mut self,
		nodes: &mut Vec<Node>,
		argument_count: u16,
		result_count: u16,
		entities: &Entities,
	) -> Link {
		references::track(&mut self.dependencies, &self.graph.instructions);

		let captures = entities.get_dependencies(&self.dependencies);
		let stack_size = self
			.local_tracker
			.run(&mut self.locals, &self.graph, result_count);
		let Some(total_argument_count) = argument_count.checked_add(2) else {
			unreachable!()
		};

		let function = Function::add_into(nodes, total_argument_count, |inner_nodes, arguments| {
			let header = FunctionHeader {
				arguments,
				argument_count: argument_count.into(),
				result_count: result_count.into(),
				local_kinds: &self.local_kinds,
				stack_size,
				dependencies: &self.dependencies,
			};

			self.lifter
				.run(inner_nodes, &self.graph, &self.locals, &header)
		});

		let closure = iter::once(function).chain(captures).collect();

		Aggregate::add_into(nodes, closure)
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
		entities: &Entities,
	) -> Link {
		let function = types.get_function_index(function);

		LocalKind::read_into(&mut self.local_kinds, body.get_locals_reader().unwrap());

		self.builder.run(
			&mut self.graph,
			types,
			BlockType::FuncType(function),
			self.local_kinds.len().try_into().unwrap(),
			body.get_operators_reader().unwrap(),
		);

		let (argument_count, result_count) = types.get_arity(function);

		self.build_data_flow(nodes, argument_count, result_count, entities)
	}

	pub fn build_synthesized(
		&mut self,
		nodes: &mut Vec<Node>,
		graph: ControlFlowGraph,
		entities: &Entities,
	) -> Link {
		self.graph = graph;
		self.local_kinds.clear();

		self.build_data_flow(nodes, 0, 0, entities)
	}
}
