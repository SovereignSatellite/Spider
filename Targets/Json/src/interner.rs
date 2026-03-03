use std::rc::Rc;

use hashbrown::HashMap;

pub struct Interner {
	map: HashMap<Rc<str>, u32>,
	list: Vec<Rc<str>>,
}

impl Interner {
	pub fn new() -> Self {
		Self {
			map: HashMap::new(),
			list: Vec::new(),
		}
	}

	pub fn list(&self) -> &[Rc<str>] {
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
		let source = Rc::<str>::from(key);

		self.list.push(Rc::clone(&source));
		self.map.insert(source, position);

		position
	}
}
