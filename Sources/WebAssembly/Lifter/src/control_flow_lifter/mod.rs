mod basic_block_lifter;
mod dependency_map;
mod region_stack;

use alloc::vec::Vec;
use ir_graph::{
	DataFlowGraph, Link,
	control::{GammaIn, GammaOut, LambdaIn, RegionIn, RegionOut, ThetaIn, ThetaOut, ValueType},
};
use web_assembly_graph::ControlFlowGraph;
use web_assembly_liveness::{locals::Locals, references::Reference};

use self::{basic_block_lifter::BasicBlockLifter, region_stack::RegionStack};

pub struct ControlFlowLifter {
	basic_block_lifter: BasicBlockLifter,
	region_stack: RegionStack,
	successors: Vec<u16>,
}

impl ControlFlowLifter {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			basic_block_lifter: BasicBlockLifter::new(),

			region_stack: RegionStack::new(),
			successors: Vec::new(),
		}
	}

	fn handle_repeat_start(&mut self, graph: &mut DataFlowGraph, locals: &[u16]) {
		let arguments = self.basic_block_lifter.get_active_bindings(locals);

		let theta_in = ThetaIn::add_into(graph, arguments);

		self.basic_block_lifter
			.set_active_bindings(theta_in, locals);

		self.region_stack.push(theta_in);
	}

	fn handle_repeat_end(&mut self, graph: &mut DataFlowGraph, condition: Link, locals: &[u16]) {
		let results = self.basic_block_lifter.get_active_bindings(locals);

		let theta_in = self.region_stack.pop();
		let theta_out = ThetaOut::add_into(graph, theta_in, results, condition);

		self.basic_block_lifter
			.set_active_bindings(theta_out, locals);
	}

	fn handle_branch_start(&mut self, graph: &mut DataFlowGraph, condition: Link) {
		let arguments = self
			.basic_block_lifter
			.get_active_bindings(&self.successors);

		let gamma_in = GammaIn::add_into(graph, arguments, condition);

		self.region_stack.push_gamma();
		self.region_stack.push(gamma_in);
	}

	fn handle_branch_end(&mut self, graph: &mut DataFlowGraph, locals: &[u16]) {
		let regions = self.region_stack.pop_gamma();

		let gamma_in = self.region_stack.pop();
		let gamma_out = GammaOut::add_into(graph, gamma_in, regions);

		self.basic_block_lifter
			.set_active_bindings(gamma_out, locals);
	}

	fn handle_path_start(&mut self, graph: &mut DataFlowGraph) {
		let gamma_in = self.region_stack.peek_gamma();
		let region_in = RegionIn::add_into(graph, gamma_in);

		self.region_stack.push(region_in);

		self.basic_block_lifter
			.set_active_bindings(region_in, &self.successors);
	}

	fn handle_path_end(&mut self, graph: &mut DataFlowGraph, locals: &[u16]) {
		let results = self.basic_block_lifter.get_active_bindings(locals);

		let region_in = self.region_stack.pop();
		let region_out = RegionOut::add_into(graph, region_in, results);

		self.region_stack.push(region_out);
	}

	fn handle_basic_block(
		&mut self,
		data_flow_graph: &mut DataFlowGraph,
		control_flow_graph: &ControlFlowGraph,
		id: u16,
		locals: &Locals,
	) {
		// We just started down the paths in a branch.
		if let Some(start) = control_flow_graph.find_branch_start(id) {
			locals.get_union(control_flow_graph.successors(start), &mut self.successors);

			self.handle_path_start(data_flow_graph);
		}
		// We just finished a branch region.
		else if control_flow_graph.is_branch_end(id) {
			self.handle_branch_end(data_flow_graph, locals.get(id));
		}

		// We just started a repeat region.
		if control_flow_graph.find_repeat_end(id).is_some() {
			self.handle_repeat_start(data_flow_graph, locals.get(id));
		}

		self.basic_block_lifter
			.run(data_flow_graph, control_flow_graph.instructions(id));

		// We just started a branch region.
		if control_flow_graph.is_branch_start(id) {
			let condition = self.basic_block_lifter.get_condition();

			locals.get_union(control_flow_graph.successors(id), &mut self.successors);

			self.handle_branch_start(data_flow_graph, condition);

			return;
		}

		// We just ended a repeat region.
		if let Some(start) = control_flow_graph.find_repeat_start(id) {
			let condition = self.basic_block_lifter.get_condition();

			self.handle_repeat_end(data_flow_graph, condition, locals.get(start));
		}

		// We just finished a path in a branch.
		if let Some(end) = control_flow_graph.find_branch_end(id) {
			self.handle_path_end(data_flow_graph, locals.get(end));
		}
	}

	pub fn set_function_data(
		&mut self,
		graph: &mut DataFlowGraph,
		lambda_in: u32,
		stack_size: u16,
		local_types: &[ValueType],
		dependencies: &[Reference],
	) {
		let LambdaIn { kind, .. } = graph.get(lambda_in).as_lambda_in().unwrap();

		let arguments = kind.arguments.len();

		self.basic_block_lifter
			.set_function_inputs(lambda_in, arguments, dependencies);

		self.basic_block_lifter.set_local_types(graph, local_types);

		self.basic_block_lifter.set_stack_size(graph, stack_size);
	}

	pub fn run(
		&mut self,
		data_flow_graph: &mut DataFlowGraph,
		control_flow_graph: &ControlFlowGraph,
		lambda_in: u32,
		locals: &Locals,
	) -> Vec<Link> {
		let LambdaIn { kind, .. } = data_flow_graph.get(lambda_in).as_lambda_in().unwrap();

		let results = kind.results.len();

		for id in control_flow_graph.block_ids() {
			self.handle_basic_block(data_flow_graph, control_flow_graph, id, locals);
		}

		self.basic_block_lifter
			.get_function_outputs(data_flow_graph, results)
	}
}
