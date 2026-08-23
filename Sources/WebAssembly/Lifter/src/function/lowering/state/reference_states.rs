use ir_graph::Link;
use web_assembly_control_flow::instruction::{Instruction, Reference, ReferenceType};

pub struct ReferenceStates {
	states: Vec<(Reference, Link)>,
}

impl ReferenceStates {
	pub const fn new() -> Self {
		Self { states: Vec::new() }
	}

	pub fn collect_references(&mut self, instructions: &[Instruction]) {
		self.states.clear();

		for &instruction in instructions {
			instruction
				.for_each_reference(|reference| self.states.push((reference, Link::DANGLING)));
		}

		self.states
			.sort_unstable_by_key(|&(reference, _)| reference);
		self.states.dedup_by_key(|state| state.0);
	}

	pub const fn count(&self) -> usize {
		self.states.len()
	}

	pub fn references(&self) -> impl ExactSizeIterator<Item = Reference> + '_ {
		self.states.iter().map(|&(reference, _)| reference)
	}

	fn position(&self, kind: ReferenceType, id: u16) -> usize {
		self.states
			.binary_search_by_key(&Reference { kind, id }, |&(reference, _)| reference)
			.unwrap()
	}

	pub fn get(&self, kind: ReferenceType, id: u16) -> Link {
		let position = self.position(kind, id);

		self.states[position].1
	}

	pub fn set(&mut self, kind: ReferenceType, id: u16, value: Link) {
		let position = self.position(kind, id);

		self.states[position].1 = value;
	}

	pub fn get_all_into(&self, target: &mut Vec<Link>) {
		target.extend(self.states.iter().map(|&(_, link)| link));
	}

	pub fn get_mutable_into(&self, target: &mut Vec<Link>) {
		target.extend(
			self.states
				.iter()
				.filter_map(|&(reference, link)| reference.kind.is_mutable().then_some(link)),
		);
	}

	pub fn set_all_from<Links>(&mut self, values: Links)
	where
		Links: IntoIterator<Item = Link>,
	{
		self.states
			.iter_mut()
			.zip(values)
			.for_each(|((_, link), value)| *link = value);
	}

	pub fn set_mutable_from<Links>(&mut self, values: Links)
	where
		Links: IntoIterator<Item = Link>,
	{
		self.states
			.iter_mut()
			.filter(|(reference, _)| reference.kind.is_mutable())
			.zip(values)
			.for_each(|((_, link), value)| *link = value);
	}
}
