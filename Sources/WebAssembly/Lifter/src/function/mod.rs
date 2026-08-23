use core::mem;

use wasmparser::{BlockType, FunctionBody};

use ir_graph::{Link, Node, operation::Aggregate, region::Function};
use web_assembly_control_flow::{ControlFlowGraph, LiveLocals, LocalLiveness};

use self::{
	decode::FunctionDecoder,
	lowering::{FunctionFrame, LocalKind, RegionLifter},
};
use crate::module::{ModuleBindings, TypeRegistry};

mod decode;
mod lowering;

pub struct FunctionLifter {
	decoder: FunctionDecoder,
	region_lifter: RegionLifter,
	local_liveness: LocalLiveness,

	live_locals: LiveLocals,
	local_kinds: Vec<LocalKind>,
}

impl FunctionLifter {
	pub const fn new() -> Self {
		Self {
			decoder: FunctionDecoder::new(),
			region_lifter: RegionLifter::new(),
			local_liveness: LocalLiveness::new(),

			live_locals: LiveLocals::new(),
			local_kinds: Vec::new(),
		}
	}

	pub fn take_graph(&mut self) -> ControlFlowGraph {
		mem::take(self.decoder.graph_mut())
	}

	fn build_data_flow(
		&mut self,
		nodes: &mut Vec<Node>,
		argument_count: u16,
		result_count: u16,
		bindings: &ModuleBindings,
	) -> Link {
		let Self {
			decoder,
			region_lifter,
			local_liveness,
			live_locals,
			local_kinds,
		} = self;
		let graph = decoder.graph();

		region_lifter.collect_references(&graph.instructions);

		let slot_count = local_liveness.run(live_locals, graph, result_count);
		let Some(total_argument_count) = argument_count.checked_add(2) else {
			unreachable!()
		};

		let function = Function::add_into(nodes, total_argument_count, |inner_nodes, arguments| {
			let frame = FunctionFrame {
				arguments,
				argument_count: argument_count.into(),
				result_count: result_count.into(),
				local_kinds,
				slot_count,
			};

			region_lifter.run(inner_nodes, graph, live_locals, &frame)
		});

		let mut closure = Vec::with_capacity(region_lifter.references().len() + 1);

		closure.push(function);
		bindings.collect_dependencies_into(region_lifter.references(), &mut closure);

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
		types: &TypeRegistry,
		bindings: &ModuleBindings,
	) -> Link {
		let function = types.get_function_index(function);

		LocalKind::read_into(&mut self.local_kinds, body.get_locals_reader().unwrap());

		self.decoder.run(
			types,
			BlockType::FuncType(function),
			self.local_kinds.len().try_into().unwrap(),
			body.get_operators_reader().unwrap(),
		);

		let (argument_count, result_count) = types.get_arity(function);

		self.build_data_flow(nodes, argument_count, result_count, bindings)
	}

	pub fn build_synthesized(
		&mut self,
		nodes: &mut Vec<Node>,
		graph: ControlFlowGraph,
		bindings: &ModuleBindings,
	) -> Link {
		*self.decoder.graph_mut() = graph;
		self.local_kinds.clear();

		self.build_data_flow(nodes, 0, 0, bindings)
	}
}
