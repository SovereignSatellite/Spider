use alloc::sync::{Arc, Weak};
use core::{iter, ops::Range};

use parking_lot::Mutex;
use wasmparser::{LocalsReader, ValType};

use ir_graph::{
	Link, Node,
	region::{Branch, Match, Repeat},
};
use web_assembly_control_flow::{
	ControlFlowGraph, LiveLocals,
	instruction::{Instruction, Reference},
};

use self::state::LoweringState;

mod state;

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

	pub fn read_into(kinds: &mut Vec<Self>, reader: LocalsReader<'_>) {
		kinds.clear();

		for (count, value_type) in reader.into_iter().map(Result::unwrap) {
			let kind = Self::classify(value_type);
			let count = count.try_into().unwrap();

			kinds.extend(iter::repeat_n(kind, count));
		}
	}
}

pub struct FunctionFrame<'function> {
	pub arguments: u32,
	pub argument_count: usize,
	pub result_count: usize,
	pub local_kinds: &'function [LocalKind],
	pub slot_count: u16,
}

#[derive(Clone, Copy)]
struct ControlFlowBody<'function> {
	graph: &'function ControlFlowGraph,
	live_locals: &'function LiveLocals,
}

#[derive(Clone, Copy)]
struct Diamond<'function> {
	body: ControlFlowBody<'function>,
	head: u16,
	merge: u16,
}

impl Diamond<'_> {
	fn arm_end(&self, entry: u16) -> u16 {
		self.body
			.graph
			.successors(self.head)
			.filter(|&other| other > entry)
			.min()
			.unwrap_or(self.merge)
	}
}

pub struct RegionLifter {
	state: LoweringState,
	live_union: Vec<u16>,
}

impl RegionLifter {
	pub const fn new() -> Self {
		Self {
			state: LoweringState::new(),
			live_union: Vec::new(),
		}
	}

	pub fn collect_references(&mut self, instructions: &[Instruction]) {
		self.state.collect_references(instructions);
	}

	pub fn references(&self) -> impl ExactSizeIterator<Item = Reference> + '_ {
		self.state.references()
	}

	fn lift_repeat(
		&mut self,
		nodes: &mut Vec<Node>,
		body: ControlFlowBody<'_>,
		header: u16,
		latch: u16,
	) -> u16 {
		let carried = body.live_locals.get(header);
		let arguments = self.state.capture_bindings(carried);

		let repeat = Repeat::add_into(nodes, arguments, |body_nodes, repeat_arguments| {
			self.state.rebind_bindings(repeat_arguments, carried);

			let condition = if header == latch {
				self.state
					.run_branch(body_nodes, body.graph.instructions(header))
			} else {
				let start = if body.graph.is_branch_start(header) {
					let condition = self
						.state
						.run_branch(body_nodes, body.graph.instructions(header));

					self.lift_match(body_nodes, body, header, condition)
				} else {
					self.state.run(body_nodes, body.graph.instructions(header));

					header + 1
				};

				self.lift_interval(body_nodes, body, start..latch);

				self.state
					.run_branch(body_nodes, body.graph.instructions(latch))
			};

			(self.state.capture_bindings(carried), condition)
		});

		self.state.rebind_bindings(repeat, carried);

		latch + 1
	}

	fn lift_arm(
		&mut self,
		parent: &Weak<Mutex<Match>>,
		argument_count: u16,
		diamond: Diamond<'_>,
		entry: u16,
	) -> Arc<Mutex<Branch>> {
		let Diamond { body, head, merge } = diamond;
		let end = diamond.arm_end(entry);

		debug_assert!(entry < end, "branch arms are never empty");

		Branch::create(
			Weak::clone(parent),
			argument_count,
			|arm_nodes, arm_arguments| {
				body.live_locals
					.get_union(body.graph.successors(head), &mut self.live_union);
				self.state.rebind_bindings(arm_arguments, &self.live_union);

				self.lift_interval(arm_nodes, body, entry..end);

				self.state.capture_bindings(body.live_locals.get(merge))
			},
		)
	}

	fn lift_match(
		&mut self,
		nodes: &mut Vec<Node>,
		body: ControlFlowBody<'_>,
		head: u16,
		condition: Link,
	) -> u16 {
		debug_assert!(
			body.graph.find_repeat_start(head).is_none(),
			"branch heads never carry back-edges"
		);

		let diamond = Diamond {
			body,
			head,
			merge: body.graph.branch_merge(head),
		};

		body.live_locals
			.get_union(body.graph.successors(head), &mut self.live_union);

		let arguments = self.state.capture_bindings(&self.live_union);

		let matcher = Match::add_into(nodes, arguments, condition, |parent, argument_count| {
			body.graph
				.successors(head)
				.map(|entry| self.lift_arm(parent, argument_count, diamond, entry))
				.collect()
		});

		self.state
			.rebind_bindings(matcher, body.live_locals.get(diamond.merge));

		diamond.merge
	}

	fn lift_unit_unbounded(
		&mut self,
		nodes: &mut Vec<Node>,
		body: ControlFlowBody<'_>,
		first: u16,
	) -> u16 {
		stacker::maybe_grow(0x1_0000, 0x10_0000, || self.lift_unit(nodes, body, first))
	}

	fn lift_unit(&mut self, nodes: &mut Vec<Node>, body: ControlFlowBody<'_>, first: u16) -> u16 {
		if let Some(latch) = body.graph.find_repeat_end(first) {
			return self.lift_repeat(nodes, body, first, latch);
		}

		if body.graph.is_branch_start(first) {
			let condition = self.state.run_branch(nodes, body.graph.instructions(first));

			return self.lift_match(nodes, body, first, condition);
		}

		self.state.run(nodes, body.graph.instructions(first));

		first + 1
	}

	fn lift_interval(
		&mut self,
		nodes: &mut Vec<Node>,
		body: ControlFlowBody<'_>,
		range: Range<u16>,
	) {
		let mut first = range.start;

		while first < range.end {
			first = self.lift_unit_unbounded(nodes, body, first);
		}

		debug_assert_eq!(first, range.end, "units must tile the interval exactly");
	}

	pub fn run(
		&mut self,
		nodes: &mut Vec<Node>,
		graph: &ControlFlowGraph,
		live_locals: &LiveLocals,
		frame: &FunctionFrame<'_>,
	) -> Vec<Link> {
		self.state.seed(nodes, frame);

		let body = ControlFlowBody { graph, live_locals };

		self.lift_interval(nodes, body, graph.block_ids());

		self.state.capture_outputs(nodes, frame.result_count)
	}
}
