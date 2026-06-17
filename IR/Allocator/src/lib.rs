//! Register allocation for IR regions.

#![no_std]

extern crate alloc;

use ir_graph::{Node, region::Function};

use self::{collector::Collector, sweeper::Sweeper};

pub use self::{
	arena::{Arena, DEFERRED},
	policy::Policy,
};

mod arena;
mod collector;
mod policy;
mod sweeper;
mod value;

/// Interval register allocator over exact value intervals for RVSDG functions.
pub struct Allocator {
	collector: Collector,
	sweeper: Sweeper,
}

impl Allocator {
	/// Creates a new allocator.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			collector: Collector::new(),
			sweeper: Sweeper::new(),
		}
	}

	/// Allocates registers for one emitted body and returns the populated arena
	/// together with the register count.
	pub fn run(&mut self, policy: &dyn Policy, nodes: &[Node]) -> (Arena, u32) {
		self.collector.run(policy, nodes);

		let parameter_count = nodes[Function::ARGUMENTS_ID as usize].result_count();
		let assignments = self
			.sweeper
			.run(policy, parameter_count, self.collector.values());

		self.collector.resolve(assignments)
	}
}

impl Default for Allocator {
	fn default() -> Self {
		Self::new()
	}
}
