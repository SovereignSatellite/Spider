use alloc::sync::Arc;

use hashbrown::HashMap;

pub struct Interner {
	map: HashMap<Arc<str>, u32>,
	list: Vec<Arc<str>>,
}

impl Interner {
	pub fn new() -> Self {
		Self {
			map: HashMap::new(),
			list: Vec::new(),
		}
	}

	pub fn list(&self) -> &[Arc<str>] {
		&self.list
	}

	pub fn clear(&mut self) {
		self.map.clear();
		self.list.clear();
	}

	pub fn resolve(&mut self, key: &str) -> u32 {
		if let Some(&position) = self.map.get(key) {
			return position;
		}

		let position = self.list.len().try_into().unwrap();
		let source = Arc::<str>::from(key);

		self.list.push(Arc::clone(&source));
		self.map.insert(source, position);

		position
	}
}
