use core::ops::Range;

use data_flow_graph::Link;
use hashbrown::HashMap;
use luau_tree::expression::Name;

use crate::{place::Place, scoped_provider::ScopedProvider};

pub struct LocalProvider {
	lifetimes: HashMap<Link, u32>,

	provider: ScopedProvider,
}

impl LocalProvider {
	pub fn new() -> Self {
		Self {
			lifetimes: HashMap::new(),

			provider: ScopedProvider::new(),
		}
	}

	pub fn lifetimes_mut(&mut self) -> &mut HashMap<Link, u32> {
		&mut self.lifetimes
	}

	pub fn pop_local_scope(&mut self) {
		self.provider.pop_local_scope();
	}

	pub fn push_local_scope(&mut self) {
		self.provider.push_local_scope();
	}

	pub fn pop_function_scope(&mut self) -> Option<(Name, u32)> {
		self.provider.pop_function_scope()
	}

	pub fn push_function_scope(&mut self) {
		self.provider.push_function_scope();
	}

	fn get_lifetime(&self, link: Link) -> u32 {
		self.lifetimes.get(&link).copied().unwrap_or(link.0 + 1)
	}

	fn pull(&mut self, link: Link) -> Place {
		let until = self.get_lifetime(link);

		self.provider.pull(until)
	}

	pub fn pull_all_into(&mut self, from: Range<u16>, to: u32, locals: &mut HashMap<Link, Place>) {
		let iter = from.map(|port| Link(to, port)).map(|link| {
			let place = self.pull(link);

			(link, place)
		});

		locals.extend(iter);
	}

	fn try_revive(&mut self, link: Link, place: Place) -> Option<Place> {
		let until = self.get_lifetime(link);

		self.provider.try_revive(place, until).then_some(
			if let Place::Definition { name } = place {
				Place::Assignment { name }
			} else {
				place
			},
		)
	}

	pub fn try_revive_all_into(
		&mut self,
		from: &[Link],
		to: u32,
		locals: &mut HashMap<Link, Place>,
	) {
		for (to, &from) in (0..).map(|port| Link(to, port)).zip(from) {
			let Some(place) = locals
				.get(&from)
				.and_then(|&place| self.try_revive(to, place))
			else {
				continue;
			};

			locals.insert(to, place);
		}
	}

	fn try_pull_all_into(&mut self, from: Range<u16>, to: u32, locals: &mut HashMap<Link, Place>) {
		for to in from.map(|port| Link(to, port)) {
			locals.entry(to).or_insert_with(|| self.pull(to));
		}
	}

	pub fn revive_all_into(&mut self, from: &[Link], to: u32, locals: &mut HashMap<Link, Place>) {
		let count = from.len().try_into().unwrap();

		// We first try to allocate variables to their old places to avoid moves.
		self.try_revive_all_into(from, to, locals);

		// Then, any remaining ones are given new places.
		self.try_pull_all_into(0..count, to, locals);
	}

	pub fn push_until(&mut self, end: u32) {
		self.provider.push_until(end);
	}
}
