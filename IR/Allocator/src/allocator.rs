//! Greedy register allocator driver.

use alloc::sync::Arc;

use hashbrown::HashMap;
use parking_lot::Mutex;

use ir_graph::{
	Link, Node,
	region::{Branch, Match, Repeat},
};

use crate::{
	coloring::Coloring,
	lifetime::{Interval, Lifetimes},
	policy::Policy,
};

/// Greedy register allocator for RVSDG regions.
pub struct Allocator {
	coloring: Coloring,
}

struct Allocation<'alloc> {
	coloring: &'alloc mut Coloring,
	policy: &'alloc dyn Policy,
	registers: &'alloc mut HashMap<(Link, usize), u32>,
}

impl Allocation<'_> {
	fn allocate_port(&mut self, scope: usize, interval: Interval) {
		if !self.policy.should_materialize(scope, interval.source) {
			return;
		}

		let register = self
			.coloring
			.allocate(self.policy, scope, interval.source, interval.end);

		self.registers.insert((interval.source, scope), register);
	}

	fn descend_branch(&mut self, arc: &Arc<Mutex<Branch>>) {
		let scope = Arc::as_ptr(arc) as usize;
		let guard = arc.lock();

		self.allocate_region(scope, &guard.nodes);
	}

	fn descend_match(&mut self, arc: &Arc<Mutex<Match>>) {
		let matcher = arc.lock();
		let entry = self.coloring.snapshot();

		for branch in &matcher.branches {
			self.coloring.restore(&entry);

			self.descend_branch(branch);
		}

		drop(matcher);

		self.coloring.restore(&entry);
	}

	fn descend_repeat(&mut self, arc: &Arc<Mutex<Repeat>>) {
		let scope = Arc::as_ptr(arc) as usize;
		let guard = arc.lock();
		let (_, results_id) = guard.roots();
		let entry = self.coloring.snapshot();

		self.allocate_repeat_region(scope, results_id, &guard.nodes);

		drop(guard);

		self.coloring.restore(&entry);
	}

	#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
	fn descend_children(&mut self, node: &Node) {
		match node {
			Node::Match(arc) => self.descend_match(arc),
			Node::Repeat(arc) => self.descend_repeat(arc),
			Node::Function(_)
			| Node::ModuleArguments(_)
			| Node::ModuleResults(_)
			| Node::FunctionArguments(_)
			| Node::FunctionResults(_)
			| Node::BranchArguments(_)
			| Node::BranchResults(_)
			| Node::RepeatArguments(_)
			| Node::RepeatResults(_)
			| Node::Foreign(_)
			| Node::Trap
			| Node::Null
			| Node::I32(_)
			| Node::I64(_)
			| Node::F32(_)
			| Node::F64(_)
			| Node::Identity(_)
			| Node::Fence(_)
			| Node::Apply(_)
			| Node::RefIsNull(_)
			| Node::IntegerUnaryOperation(_)
			| Node::IntegerBinaryOperation(_)
			| Node::IntegerCompareOperation(_)
			| Node::IntegerNarrow(_)
			| Node::IntegerWiden(_)
			| Node::IntegerSignExtend(_)
			| Node::IntegerConvertToNumber(_)
			| Node::IntegerTransmuteToNumber(_)
			| Node::NumberUnaryOperation(_)
			| Node::NumberBinaryOperation(_)
			| Node::NumberCompareOperation(_)
			| Node::NumberNarrow(_)
			| Node::NumberWiden(_)
			| Node::NumberTruncateToInteger(_)
			| Node::NumberTransmuteToInteger(_)
			| Node::MutableNew(_)
			| Node::MutableGet(_)
			| Node::MutableSet(_)
			| Node::Aggregate(_)
			| Node::Extract(_)
			| Node::TableNew(_)
			| Node::TableGet(_)
			| Node::TableSet(_)
			| Node::TableSize(_)
			| Node::TableGrow(_)
			| Node::TableFill(_)
			| Node::TableCopy(_)
			| Node::TableDrop(_)
			| Node::MemoryNew(_)
			| Node::MemoryLoad(_)
			| Node::MemoryStore(_)
			| Node::MemorySize(_)
			| Node::MemoryGrow(_)
			| Node::MemoryFill(_)
			| Node::MemoryCopy(_)
			| Node::MemoryDrop(_) => {}
		}
	}

	fn allocate_intervals(&mut self, scope: usize, nodes: &[Node], intervals: &[Interval]) {
		let active_start = self.coloring.active_len();
		let mut cursor = 0;

		for (node, id) in nodes.iter().zip(0_u32..) {
			self.coloring.retire_before(active_start, id);

			while cursor < intervals.len() && intervals[cursor].source.0 == id {
				let interval = intervals[cursor];

				cursor += 1;

				self.allocate_port(scope, interval);
			}

			self.descend_children(node);
			self.coloring.retire_finished(active_start, id);
		}
	}

	fn allocate_region(&mut self, scope: usize, nodes: &[Node]) {
		let mut lifetimes = Lifetimes::new();

		lifetimes.run(nodes);
		self.allocate_intervals(scope, nodes, lifetimes.intervals());
	}

	fn allocate_repeat_region(&mut self, scope: usize, results_id: u32, nodes: &[Node]) {
		let mut lifetimes = Lifetimes::new();

		lifetimes.run_repeat(nodes, results_id);
		self.allocate_intervals(scope, nodes, lifetimes.intervals());
	}
}

impl Allocator {
	/// Creates a new allocator.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			coloring: Coloring::new(),
		}
	}

	/// Allocates registers for one emitted body and returns its register count.
	pub fn run(
		&mut self,
		policy: &dyn Policy,
		scope: usize,
		nodes: &[Node],
		registers: &mut HashMap<(Link, usize), u32>,
	) -> u32 {
		registers.clear();

		self.coloring.reset(policy);

		Allocation {
			coloring: &mut self.coloring,
			policy,
			registers,
		}
		.allocate_region(scope, nodes);

		registers.values().max().map_or(0, |&register| register + 1)
	}
}

impl Default for Allocator {
	fn default() -> Self {
		Self::new()
	}
}
