use alloc::sync::{Arc, Weak};
use core::ops::Range;

use parking_lot::Mutex;

use ir_graph::{
	Link, Node,
	region::{Branch, Match, Repeat},
};
use web_assembly_graph::ControlFlowGraph;
use web_assembly_liveness::locals::Locals;

use super::slots::{FunctionHeader, SlotFile};

#[derive(Clone, Copy)]
struct Body<'function> {
	graph: &'function ControlFlowGraph,
	locals: &'function Locals,
}

#[derive(Clone, Copy)]
struct Diamond<'function> {
	body: Body<'function>,
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

pub struct IntervalLifter {
	slots: SlotFile,
	union: Vec<u16>,
}

impl IntervalLifter {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			slots: SlotFile::new(),
			union: Vec::new(),
		}
	}

	fn lift_loop(&mut self, nodes: &mut Vec<Node>, body: Body<'_>, header: u16, latch: u16) -> u16 {
		let carried = body.locals.get(header);
		let arguments = self.slots.capture_bindings(carried);

		let repeat = Repeat::add_into(nodes, arguments, |body_nodes, repeat_arguments| {
			self.slots.rebind_bindings(repeat_arguments, carried);
			self.slots.run(body_nodes, body.graph.instructions(header));

			let start = if body.graph.is_branch_start(header) {
				self.lift_match(body_nodes, body, header)
			} else {
				header + 1
			};

			// A header that is its own latch already ran its continuation branch.
			if header != latch {
				self.lift_interval(body_nodes, body, start..latch);
				self.slots.run(body_nodes, body.graph.instructions(latch));
			}

			(self.slots.capture_bindings(carried), self.slots.condition())
		});

		self.slots.rebind_bindings(repeat, carried);

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
				body.locals
					.get_union(body.graph.successors(head), &mut self.union);
				self.slots.rebind_bindings(arm_arguments, &self.union);

				self.lift_interval(arm_nodes, body, entry..end);

				self.slots.capture_bindings(body.locals.get(merge))
			},
		)
	}

	fn lift_match(&mut self, nodes: &mut Vec<Node>, body: Body<'_>, head: u16) -> u16 {
		debug_assert!(
			body.graph.find_repeat_start(head).is_none(),
			"branch heads never carry back-edges"
		);

		let condition = self.slots.condition();
		let diamond = Diamond {
			body,
			head,
			merge: body.graph.branch_merge(head),
		};

		body.locals
			.get_union(body.graph.successors(head), &mut self.union);

		let arguments = self.slots.capture_bindings(&self.union);

		let matcher = Match::add_into(nodes, arguments, condition, |parent, argument_count| {
			body.graph
				.successors(head)
				.map(|entry| self.lift_arm(parent, argument_count, diamond, entry))
				.collect()
		});

		self.slots
			.rebind_bindings(matcher, body.locals.get(diamond.merge));

		diamond.merge
	}

	fn lift_unit_unbounded(&mut self, nodes: &mut Vec<Node>, body: Body<'_>, first: u16) -> u16 {
		stacker::maybe_grow(0x1_0000, 0x10_0000, || self.lift_unit(nodes, body, first))
	}

	fn lift_unit(&mut self, nodes: &mut Vec<Node>, body: Body<'_>, first: u16) -> u16 {
		if let Some(latch) = body.graph.find_repeat_end(first) {
			return self.lift_loop(nodes, body, first, latch);
		}

		self.slots.run(nodes, body.graph.instructions(first));

		if body.graph.is_branch_start(first) {
			return self.lift_match(nodes, body, first);
		}

		first + 1
	}

	fn lift_interval(&mut self, nodes: &mut Vec<Node>, body: Body<'_>, range: Range<u16>) {
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
		locals: &Locals,
		header: &FunctionHeader<'_>,
	) -> Vec<Link> {
		self.slots.seed(nodes, header);

		let body = Body { graph, locals };

		self.lift_interval(nodes, body, graph.block_ids());

		self.slots.capture_outputs(nodes, header.result_count)
	}
}
