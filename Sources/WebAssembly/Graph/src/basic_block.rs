use core::ops::Range;

use list::resizable::Resizable;

/// A basic block in the control flow graph.
pub struct BasicBlock {
	/// The predecessor block indices.
	pub predecessors: Resizable<u16, 7>,
	/// The successor block indices.
	pub successors: Resizable<u16, 7>,
	/// The start instruction index (inclusive).
	pub start: u32,
	/// The end instruction index (exclusive).
	pub end: u32,
}

impl BasicBlock {
	/// Creates a new basic block from an instruction range.
	#[must_use]
	pub const fn from_range(start: u32, end: u32) -> Self {
		Self {
			predecessors: Resizable::new(),
			successors: Resizable::new(),
			start,
			end,
		}
	}

	/// Returns the instruction index range.
	#[must_use]
	pub fn range(&self) -> Range<usize> {
		let Ok(start) = self.start.try_into() else {
			unreachable!()
		};
		let Ok(end) = self.end.try_into() else {
			unreachable!()
		};

		start..end
	}

	/// Remaps both neighbour lists via the mapping function and drops predecessors whose remapped id is `u16::MAX`.
	pub fn replace_ids<M: Fn(u16) -> u16>(&mut self, map: M) {
		for id in &mut self.predecessors {
			*id = map(*id);
		}

		for id in &mut self.successors {
			*id = map(*id);
		}

		self.predecessors.retain(|&id| id != u16::MAX);
	}
}

impl Default for BasicBlock {
	fn default() -> Self {
		Self::from_range(0, 0)
	}
}
