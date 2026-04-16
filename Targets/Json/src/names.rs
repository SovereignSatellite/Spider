pub struct Names {
	mapping: Vec<u32>,
	scopes: Vec<usize>,
	next_id: u32,
}

impl Names {
	pub const fn new() -> Self {
		Self {
			mapping: Vec::new(),
			scopes: Vec::new(),
			next_id: 0,
		}
	}

	pub fn clear(&mut self) {
		self.mapping.clear();
		self.scopes.clear();
		self.next_id = 0;
	}

	pub fn enter_scope(&mut self) {
		self.scopes.push(self.mapping.len());
	}

	pub fn leave_scope(&mut self) {
		let base = self.scopes.pop().unwrap();

		self.mapping.truncate(base);
	}

	pub fn assign(&mut self) -> u32 {
		let global = self.next_id;

		self.next_id += 1;
		self.mapping.push(global);

		global
	}

	pub fn entry(&self) -> u32 {
		self.mapping[*self.scopes.last().unwrap()]
	}

	pub fn exit(&self) -> u32 {
		*self.mapping.last().unwrap()
	}

	pub fn resolve(&self, local: usize) -> u32 {
		let base = *self.scopes.last().unwrap();

		self.mapping[base + local]
	}
}
