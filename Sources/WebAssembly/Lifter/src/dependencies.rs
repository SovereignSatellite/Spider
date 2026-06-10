use ir_graph::Link;
use web_assembly_graph::instruction::{Reference, ReferenceType};

pub struct DependencyMap {
	buffer: Vec<(Reference, Link)>,
}

impl DependencyMap {
	pub const fn new() -> Self {
		Self { buffer: Vec::new() }
	}

	pub fn fill_keys(&mut self, keys: &[Reference]) {
		let keys = keys.iter().map(|&key| (key, Link::DANGLING));

		self.buffer.clear();
		self.buffer.extend(keys);
	}

	#[must_use]
	pub const fn count(&self) -> usize {
		self.buffer.len()
	}

	fn position(&self, kind: ReferenceType, id: u16) -> usize {
		self.buffer
			.binary_search_by_key(&Reference { kind, id }, |&(reference, _)| reference)
			.unwrap()
	}

	pub fn get(&self, kind: ReferenceType, id: u16) -> Link {
		let position = self.position(kind, id);

		self.buffer[position].1
	}

	pub fn set(&mut self, kind: ReferenceType, id: u16, value: Link) {
		let position = self.position(kind, id);

		self.buffer[position].1 = value;
	}

	pub fn get_all_into(&self, target: &mut Vec<Link>) {
		target.extend(self.buffer.iter().map(|&(_, link)| link));
	}

	pub fn get_mutable_into(&self, target: &mut Vec<Link>) {
		target.extend(
			self.buffer
				.iter()
				.filter_map(|(Reference { kind, .. }, link)| kind.is_mutable().then_some(link)),
		);
	}

	pub fn set_all_from<I>(&mut self, values: I)
	where
		I: IntoIterator<Item = Link>,
	{
		self.buffer
			.iter_mut()
			.zip(values)
			.for_each(|((_, link), value)| *link = value);
	}

	pub fn set_mutable_from<I>(&mut self, values: I)
	where
		I: IntoIterator<Item = Link>,
	{
		self.buffer
			.iter_mut()
			.filter(|(Reference { kind, .. }, _)| kind.is_mutable())
			.zip(values)
			.for_each(|((_, link), value)| *link = value);
	}
}
