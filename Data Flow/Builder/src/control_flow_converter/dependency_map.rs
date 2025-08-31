use alloc::vec::Vec;
use control_flow_liveness::references::{Reference, ReferenceType};
use data_flow_graph::Link;

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

	pub fn fill_values<I>(&mut self, values: I)
	where
		I: IntoIterator<Item = Link>,
	{
		self.buffer
			.iter_mut()
			.zip(values)
			.for_each(|(item, value)| item.1 = value);
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

	pub fn extend_into(&self, links: &mut Vec<Link>) {
		links.extend(self.buffer.iter().map(|item| item.1));
	}
}
