use list::resizable::Resizable;

pub struct BasicBlock {
	pub predecessors: Resizable<u16, 7>,
	pub successors: Resizable<u16, 7>,

	pub start: u32,
	pub end: u32,
}

impl BasicBlock {
	#[must_use]
	pub const fn from_range(start: u32, end: u32) -> Self {
		Self {
			predecessors: Resizable::new(),
			successors: Resizable::new(),

			start,
			end,
		}
	}

	pub(crate) fn range(&self) -> core::ops::Range<usize> {
		self.start.try_into().unwrap()..self.end.try_into().unwrap()
	}

	pub(crate) fn is_source(&self) -> bool {
		self.predecessors.is_empty()
	}

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
