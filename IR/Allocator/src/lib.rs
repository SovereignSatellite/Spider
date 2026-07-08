//! Register allocation for IR regions.

#![no_std]

extern crate alloc;

use ir_graph::{Node, region::Function};

use self::{classes::Classes, coalescer::Coalescer, collector::Collector, sweeper::Sweeper};

pub use self::{
	arena::{Arena, DEFERRED},
	hint::reuse_hint,
	policy::Policy,
};

mod arena;
mod classes;
mod coalescer;
mod collector;
mod hint;
mod policy;
mod sweeper;
mod value;

/// Coalesce-first register allocator over exact value intervals for RVSDG functions.
pub struct Allocator {
	collector: Collector,
	classes: Classes,
	coalescer: Coalescer,
	sweeper: Sweeper,
}

impl Allocator {
	/// Creates a new allocator.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			collector: Collector::new(),
			classes: Classes::new(),
			coalescer: Coalescer::new(),
			sweeper: Sweeper::new(),
		}
	}

	/// Allocates registers for one emitted body and returns the populated arena
	/// together with the register count.
	pub fn run(&mut self, policy: &dyn Policy, nodes: &[Node]) -> (Arena, u32) {
		self.collector.run(policy, nodes);

		let parameter_count = nodes[Function::ARGUMENTS_ID as usize].result_count();

		self.classes.reset(self.collector.values(), parameter_count);
		self.coalescer
			.run(policy, &mut self.classes, self.collector.copies());

		let assignments = self.sweeper.run(
			policy,
			parameter_count,
			self.collector.values(),
			&mut self.classes,
		);

		self.collector.resolve(assignments)
	}
}

impl Default for Allocator {
	fn default() -> Self {
		Self::new()
	}
}
