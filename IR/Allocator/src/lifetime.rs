use alloc::vec::Vec;
use core::cmp::Reverse;

use ir_graph::{Link, Node};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Interval {
	pub source: Link,
	pub end: u32,
}

pub struct Lifetimes {
	intervals: Vec<Interval>,
}

impl Lifetimes {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			intervals: Vec::new(),
		}
	}

	#[must_use]
	pub fn intervals(&self) -> &[Interval] {
		&self.intervals
	}

	fn push_output_fallbacks(&mut self, id: u32, result_count: u16) {
		for port in 0..result_count {
			self.intervals.push(Interval {
				source: Link(id, port),
				end: id,
			});
		}
	}

	fn push_source_pairs(&mut self, id: u32, node: &Node) {
		node.for_each_outer(|source| {
			self.intervals.push(Interval { source, end: id });
		});
	}

	fn push_repeat_arguments(&mut self, nodes: &[Node], results_id: u32) {
		let arguments = nodes
			.first()
			.unwrap_or_else(|| unreachable!("repeat body must have arguments"));

		for port in 0..arguments.result_count() {
			self.intervals.push(Interval {
				source: Link(0, port),
				end: results_id,
			});
		}
	}

	fn reduce_to_maxima(&mut self) {
		self.intervals
			.sort_unstable_by_key(|interval| Reverse(*interval));
		self.intervals.dedup_by_key(|interval| interval.source);
		self.intervals.reverse();
	}

	fn collect_region(&mut self, nodes: &[Node]) {
		self.intervals.clear();

		for (node, id) in nodes.iter().zip(0_u32..) {
			self.push_output_fallbacks(id, node.result_count());
			self.push_source_pairs(id, node);
		}
	}

	pub fn run(&mut self, nodes: &[Node]) {
		self.collect_region(nodes);
		self.reduce_to_maxima();
	}

	pub fn run_repeat(&mut self, nodes: &[Node], results_id: u32) {
		self.collect_region(nodes);
		self.push_repeat_arguments(nodes, results_id);

		self.reduce_to_maxima();
	}
}

impl Default for Lifetimes {
	fn default() -> Self {
		Self::new()
	}
}
