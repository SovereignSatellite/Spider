use alloc::sync::{Arc, Weak};
use core::ops::Range;

use parking_lot::Mutex;

use ir_graph::{
	Link, Node,
	region::{Branch, Match, Repeat},
};
use web_assembly_graph::ControlFlowGraph;
use web_assembly_liveness::{locals::Locals, references::Reference};

use super::{basic_block::BasicBlockLifter, function::LocalKind};

pub struct ControlFlowLifter {
	basic_block_lifter: BasicBlockLifter,
	successors: Vec<u16>,
	block_ids: Range<u16>,
}

#[derive(Clone, Copy)]
struct BranchPath<'path> {
	graph: &'path ControlFlowGraph,
	locals: &'path Locals,
	start: u16,
}

impl ControlFlowLifter {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			basic_block_lifter: BasicBlockLifter::new(),

			successors: Vec::new(),
			block_ids: 0..0,
		}
	}

	fn handle_repeat_start(&mut self, arguments: u32, locals: &[u16]) {
		self.basic_block_lifter
			.set_active_bindings(arguments, locals);
	}

	fn handle_repeat_body(
		&mut self,
		nodes: &mut Vec<Node>,
		graph: &ControlFlowGraph,
		locals: &Locals,
		id: u16,
	) {
		self.basic_block_lifter.run(nodes, graph.instructions(id));

		if graph.is_branch_start(id) {
			self.enter_match(nodes, graph, locals, id);
		}

		if graph.find_repeat_start(id).is_none() {
			self.handle_blocks(nodes, graph, locals);
		}
	}

	fn handle_repeat_end(&self, locals: &[u16]) -> (Vec<Link>, Link) {
		let condition = self.basic_block_lifter.get_condition();
		let results = self.basic_block_lifter.get_active_bindings(locals);

		(results, condition)
	}

	fn enter_repeat(
		&mut self,
		nodes: &mut Vec<Node>,
		graph: &ControlFlowGraph,
		locals: &Locals,
		id: u16,
	) {
		let repeat_locals = locals.get(id);
		let arguments = self.basic_block_lifter.get_active_bindings(repeat_locals);

		let repeat = Repeat::add_into(nodes, arguments, |nodes, repeat_arguments| {
			self.handle_repeat_start(repeat_arguments, repeat_locals);
			self.handle_repeat_body(nodes, graph, locals, id);
			self.handle_repeat_end(repeat_locals)
		});

		self.basic_block_lifter
			.set_active_bindings(repeat, repeat_locals);
	}

	fn handle_path_start(&mut self, path: BranchPath<'_>, arguments: u32) {
		path.locals
			.get_union(path.graph.successors(path.start), &mut self.successors);

		self.basic_block_lifter
			.set_active_bindings(arguments, &self.successors);
	}

	fn handle_path_end(&self, path: BranchPath<'_>) -> Vec<Link> {
		// The next block is the branch merge.
		let last = self.block_ids.start - 1;
		let end = path.graph.find_branch_end(last).unwrap();

		self.basic_block_lifter
			.get_active_bindings(path.locals.get(end))
	}

	fn handle_path(
		&mut self,
		parent: &Weak<Mutex<Match>>,
		argument_count: u16,
		path: BranchPath<'_>,
	) -> Arc<Mutex<Branch>> {
		Branch::create(
			Weak::clone(parent),
			argument_count,
			|path_nodes, branch_arguments| {
				self.handle_path_start(path, branch_arguments);
				self.handle_blocks(path_nodes, path.graph, path.locals);
				self.handle_path_end(path)
			},
		)
	}

	fn handle_branch_start(
		&mut self,
		graph: &ControlFlowGraph,
		locals: &Locals,
		start: u16,
	) -> (Link, Vec<Link>) {
		let condition = self.basic_block_lifter.get_condition();

		locals.get_union(graph.successors(start), &mut self.successors);

		let arguments = self
			.basic_block_lifter
			.get_active_bindings(&self.successors);

		(condition, arguments)
	}

	fn handle_branch_end(&mut self, id: u32, locals: &Locals) {
		// The match result becomes the active binding set at the merge block.
		let merge = self.block_ids.start;

		self.basic_block_lifter
			.set_active_bindings(id, locals.get(merge));
	}

	fn enter_match(
		&mut self,
		nodes: &mut Vec<Node>,
		graph: &ControlFlowGraph,
		locals: &Locals,
		start: u16,
	) {
		let (condition, arguments) = self.handle_branch_start(graph, locals, start);
		let path = BranchPath {
			graph,
			locals,
			start,
		};

		let id = Match::add_into(nodes, arguments, condition, |parent, argument_count| {
			graph
				.successors(start)
				.map(|_| self.handle_path(parent, argument_count, path))
				.collect()
		});

		self.handle_branch_end(id, locals);
	}

	fn handle_basic_block(
		&mut self,
		nodes: &mut Vec<Node>,
		graph: &ControlFlowGraph,
		locals: &Locals,
		id: u16,
	) -> bool {
		// A back-edge predecessor marks a loop header.
		if graph.find_repeat_end(id).is_some() {
			self.enter_repeat(nodes, graph, locals, id);

			return true;
		}

		self.basic_block_lifter.run(nodes, graph.instructions(id));

		// A multi-successor block starts a match region.
		if graph.is_branch_start(id) {
			self.enter_match(nodes, graph, locals, id);

			return true;
		}

		// A back-edge successor ends the current repeat body.
		if graph.find_repeat_start(id).is_some() {
			return false;
		}

		// A branch-end successor stops the current path before its merge block.
		if graph.find_branch_end(id).is_some() {
			return false;
		}

		true
	}

	fn handle_blocks(&mut self, nodes: &mut Vec<Node>, graph: &ControlFlowGraph, locals: &Locals) {
		while let Some(id) = self.block_ids.next() {
			if !self.handle_basic_block(nodes, graph, locals, id) {
				return;
			}
		}
	}

	#[expect(
		clippy::too_many_arguments,
		reason = "lifter setup requires all parameters"
	)]
	pub fn set_function_data(
		&mut self,
		nodes: &mut Vec<Node>,
		arguments: u32,
		argument_count: usize,
		stack_size: u16,
		local_kinds: &[LocalKind],
		dependencies: &[Reference],
	) {
		self.basic_block_lifter
			.set_function_inputs(nodes, arguments, argument_count, dependencies);

		self.basic_block_lifter.set_local_types(nodes, local_kinds);

		self.basic_block_lifter.set_stack_size(nodes, stack_size);
	}

	pub fn run(
		&mut self,
		nodes: &mut Vec<Node>,
		control_flow_graph: &ControlFlowGraph,
		result_count: usize,
		locals: &Locals,
	) -> Vec<Link> {
		self.block_ids = control_flow_graph.block_ids();

		self.handle_blocks(nodes, control_flow_graph, locals);

		self.basic_block_lifter
			.get_function_outputs(nodes, result_count)
	}
}
