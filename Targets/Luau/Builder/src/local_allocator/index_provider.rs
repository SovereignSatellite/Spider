use alloc::collections::binary_heap::{BinaryHeap, PeekMut};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Hold {
	start: u32,
	name: u32,
}

pub struct IndexProvider {
	holds: BinaryHeap<Hold>,
	free: BinaryHeap<u32>,

	names: u32,
}

impl IndexProvider {
	pub const fn new() -> Self {
		Self {
			holds: BinaryHeap::new(),
			free: BinaryHeap::new(),

			names: 0,
		}
	}

	pub const fn get_names(&self) -> u32 {
		self.names
	}

	pub const fn set_names(&mut self, names: u32) {
		self.names = names;
	}

	pub fn forget_names(&mut self) {
		self.holds.clear();
		self.free.clear();
	}

	pub fn should_exceed(&self, count: usize) -> bool {
		self.holds.len() >= count
	}

	const fn pull_name(&mut self) -> u32 {
		let name = self.names;

		self.names = name + 1;

		name
	}

	pub fn pull(&mut self, start: u32) -> u32 {
		let name = self.free.pop().unwrap_or_else(|| self.pull_name());

		self.holds.push(Hold { start, name });

		name
	}

	fn remove_free_if_found(&mut self, name: u32) -> bool {
		if let Some(position) = self.free.iter().position(|&other| name == other) {
			let mut free = core::mem::take(&mut self.free).into_vec();

			free.swap_remove(position);

			self.free = free.into();

			true
		} else {
			false
		}
	}

	pub fn try_revive(&mut self, name: u32, start: u32) -> bool {
		if self.remove_free_if_found(name) {
			self.holds.push(Hold { start, name });

			true
		} else {
			false
		}
	}

	pub fn push_until(&mut self, start: u32) {
		while let Some(peek) = self.holds.peek_mut() {
			if peek.start < start {
				break;
			}

			let peek = PeekMut::pop(peek);

			self.free.push(peek.name);
		}
	}
}
