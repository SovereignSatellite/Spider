use alloc::collections::binary_heap::{BinaryHeap, PeekMut};

#[derive(PartialEq, Eq)]
struct Hold {
	name: u32,
	until: u32,
}

impl PartialOrd for Hold {
	fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Hold {
	fn cmp(&self, other: &Self) -> core::cmp::Ordering {
		let name = self.name.cmp(&other.name);
		let until = self.until.cmp(&other.until);

		// We keep the ordering of `name` because pushing to `free`
		// in ascending order is worst case.
		until.reverse().then(name)
	}
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

	pub fn should_create(&self) -> bool {
		self.free.is_empty()
	}

	pub fn should_exceed(&self, count: usize) -> bool {
		self.should_create() && self.holds.len() >= count
	}

	pub const fn get_names(&self) -> u32 {
		self.names
	}

	pub fn set_names(&mut self, names: u32) {
		self.names = names;
	}

	pub fn forget_free(&mut self, last: u32) {
		self.free.retain(|&name| name < last);
	}

	pub fn pull(&mut self, until: u32) -> u32 {
		let name = self.free.pop().unwrap_or_else(|| {
			let name = self.names;

			self.names = name + 1;

			name
		});

		self.holds.push(Hold { name, until });

		name
	}

	pub fn try_revive(&mut self, name: u32, until: u32) -> bool {
		if let Some(position) = self.free.iter().position(|&other| name == other) {
			let mut free = core::mem::take(&mut self.free).into_vec();

			free.swap_remove(position);

			self.free = free.into();

			self.holds.push(Hold { name, until });

			true
		} else {
			false
		}
	}

	pub fn push_until(&mut self, end: u32) {
		while let Some(peek) = self.holds.peek_mut() {
			if peek.until != end {
				break;
			}

			let peek = PeekMut::pop(peek);

			self.free.push(peek.name);
		}
	}
}
