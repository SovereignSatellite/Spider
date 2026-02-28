use alloc::vec::Vec;
use ir_graph::Link;
use web_assembly_liveness::references::{Reference, ReferenceType};

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

	fn position(&self, r#type: ReferenceType, id: u16) -> usize {
		self.buffer
			.binary_search_by_key(&Reference { r#type, id }, |data| data.0)
			.unwrap()
	}

	pub fn get(&self, r#type: ReferenceType, id: u16) -> Link {
		let position = self.position(r#type, id);

		self.buffer[position].1
	}

	pub fn set(&mut self, r#type: ReferenceType, id: u16, value: Link) {
		let position = self.position(r#type, id);

		self.buffer[position].1 = value;
	}

	pub fn get_all_into(&self, target: &mut Vec<Link>) {
		let iter = self.buffer.iter().map(|data| data.1);

		target.extend(iter);
	}

	pub fn get_mutable_into(&self, target: &mut Vec<Link>) {
		let iter = self
			.buffer
			.iter()
			.filter_map(|(Reference { r#type, .. }, link)| r#type.is_mutable().then_some(link));

		target.extend(iter);
	}

	pub fn set_all_from<I>(&mut self, values: I)
	where
		I: IntoIterator<Item = Link>,
	{
		self.buffer
			.iter_mut()
			.zip(values)
			.for_each(|(reference, value)| reference.1 = value);
	}

	pub fn set_mutable_from<I>(&mut self, values: I)
	where
		I: IntoIterator<Item = Link>,
	{
		self.buffer
			.iter_mut()
			.filter(|(Reference { r#type, .. }, _)| r#type.is_mutable())
			.zip(values)
			.for_each(|(reference, value)| reference.1 = value);
	}
}
