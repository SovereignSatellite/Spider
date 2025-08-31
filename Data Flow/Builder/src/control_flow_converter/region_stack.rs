use alloc::vec::Vec;

pub struct RegionStack {
	gammas: Vec<usize>,
	ids: Vec<u32>,
}

impl RegionStack {
	pub const fn new() -> Self {
		Self {
			gammas: Vec::new(),
			ids: Vec::new(),
		}
	}

	pub fn push_gamma(&mut self) {
		self.gammas.push(self.ids.len());
	}

	pub fn peek_gamma(&self) -> u32 {
		let start = *self.gammas.last().unwrap();

		self.ids[start]
	}

	pub fn pop_gamma(&mut self) -> Vec<u32> {
		let start = self.gammas.pop().unwrap() + 1;

		self.ids.split_off(start)
	}

	pub fn push(&mut self, id: u32) {
		self.ids.push(id);
	}

	pub fn pop(&mut self) -> u32 {
		self.ids.pop().unwrap()
	}
}
