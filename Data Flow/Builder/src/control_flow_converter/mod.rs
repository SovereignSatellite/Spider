use alloc::vec::Vec;
use control_flow_graph::ControlFlowGraph;
use control_flow_liveness::{locals::Locals, references::Reference};
use data_flow_graph::{
	nested::{LambdaIn, ValueType},
	DataFlowGraph, Link,
};

use self::{basic_block_converter::BasicBlockConverter, region_stack::RegionStack};

mod basic_block_converter;
mod dependency_map;
mod region_stack;

pub struct ControlFlowConverter {
	basic_block_converter: BasicBlockConverter,

	region_stack: RegionStack,
	successors: Vec<u16>,
}

impl ControlFlowConverter {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			basic_block_converter: BasicBlockConverter::new(),

			region_stack: RegionStack::new(),
			successors: Vec::new(),
		}
	}

	fn handle_repeat_start(&mut self, graph: &mut DataFlowGraph, locals: &[u16]) {
		let arguments = self.basic_block_converter.get_active_bindings(locals);

		let theta_in = graph.add_theta_in(arguments);

		self.basic_block_converter
			.set_active_bindings(theta_in, locals);

		self.region_stack.push(theta_in);
	}

	fn handle_repeat_end(&mut self, graph: &mut DataFlowGraph, condition: Link, locals: &[u16]) {
		let results = self.basic_block_converter.get_active_bindings(locals);

		let theta_in = self.region_stack.pop();
		let theta_out = graph.add_theta_out(theta_in, condition, results);

		self.basic_block_converter
			.set_active_bindings(theta_out, locals);
	}

	fn handle_branch_start(&mut self, graph: &mut DataFlowGraph, condition: Link) {
		let arguments = self
			.basic_block_converter
			.get_active_bindings(&self.successors);

		let gamma_in = graph.add_gamma_in(condition, arguments);

		self.region_stack.push_gamma();
		self.region_stack.push(gamma_in);
	}

	fn handle_branch_end(&mut self, graph: &mut DataFlowGraph, locals: &[u16]) {
		let regions = self.region_stack.pop_gamma();

		let gamma_in = self.region_stack.pop();
		let gamma_out = graph.add_gamma_out(gamma_in, regions);

		self.basic_block_converter
			.set_active_bindings(gamma_out, locals);
	}

	fn handle_path_start(&mut self, graph: &mut DataFlowGraph) {
		let gamma_in = self.region_stack.peek_gamma();
		let region_in = graph.add_region_in(gamma_in);

		self.region_stack.push(region_in);

		self.basic_block_converter
			.set_active_bindings(region_in, &self.successors);
	}

	fn handle_path_end(&mut self, graph: &mut DataFlowGraph, locals: &[u16]) {
		let results = self.basic_block_converter.get_active_bindings(locals);

		let region_in = self.region_stack.pop();
		let region_out = graph.add_region_out(region_in, results);

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

		self.basic_block_converter
			.run(data_flow_graph, control_flow_graph.instructions(id));

		// We just started a branch region.
		if control_flow_graph.is_branch_start(id) {
			let condition = self.basic_block_converter.get_condition();

			locals.get_union(control_flow_graph.successors(id), &mut self.successors);

			self.handle_branch_start(data_flow_graph, condition);

			return;
		}

		// We just ended a repeat region.
		if let Some(start) = control_flow_graph.find_repeat_start(id) {
			let condition = self.basic_block_converter.get_condition();

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
		let LambdaIn { r#type, .. } = graph.get(lambda_in).as_lambda_in().unwrap();

		let arguments = r#type.arguments.len();

		self.basic_block_converter
			.set_function_inputs(lambda_in, arguments, dependencies);

		self.basic_block_converter
			.set_local_types(graph, local_types);

		self.basic_block_converter.set_stack_size(graph, stack_size);
	}

	pub fn run(
		&mut self,
		data_flow_graph: &mut DataFlowGraph,
		control_flow_graph: &ControlFlowGraph,
		lambda_in: u32,
		locals: &Locals,
	) -> Vec<Link> {
		let LambdaIn { r#type, .. } = data_flow_graph.get(lambda_in).as_lambda_in().unwrap();

		let results = r#type.results.len();

		for id in control_flow_graph.block_ids() {
			self.handle_basic_block(data_flow_graph, control_flow_graph, id, locals);
		}

		self.basic_block_converter.get_function_outputs(results)
	}
}
