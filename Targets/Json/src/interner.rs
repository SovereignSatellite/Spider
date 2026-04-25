use alloc::sync::Arc;

use hashbrown::HashMap;

pub struct Interner {
	positions: HashMap<Arc<str>, u32>,
	strings: Vec<Arc<str>>,
}

impl Interner {
	pub fn new() -> Self {
		Self {
			positions: HashMap::new(),
			strings: Vec::new(),
		}
	}

	pub fn strings(&self) -> &[Arc<str>] {
		&self.strings
	}

	pub fn clear(&mut self) {
		self.positions.clear();
		self.strings.clear();
	}

	pub fn resolve(&mut self, key: &str) -> u32 {
		if let Some(&position) = self.positions.get(key) {
			return position;
		}

		let position = self.strings.len().try_into().unwrap();
		let source = Arc::<str>::from(key);

		self.strings.push(Arc::clone(&source));
		self.positions.insert(source, position);

		position
	}
}
