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

	/// Returns whether this block is a source (has no predecessors).
	#[must_use]
	pub const fn is_source(&self) -> bool {
		self.predecessors.is_empty()
	}

	/// Returns the instruction index range.
	///
	/// # Panics
	///
	/// Panics if the indices cannot be converted; if this happens, it is a bug.
	#[must_use]
	pub fn range(&self) -> core::ops::Range<usize> {
		self.start.try_into().unwrap()..self.end.try_into().unwrap()
	}

	/// Replaces block IDs using the given mapping function.
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
