use luau_tree::expression::Name;

use crate::place::Place;

use super::index_provider::IndexProvider;

pub enum TableProvider {
	Just {
		provider: IndexProvider,
		table: Name,
	},
	Nothing,
}

impl TableProvider {
	fn get_or_create(&mut self, provider: &mut IndexProvider) -> (&mut IndexProvider, Name) {
		if matches!(self, Self::Nothing) {
			let id = provider.pull(u32::MAX);

			*self = Self::Just {
				provider: IndexProvider::new(),
				table: Name { id },
			};
		}

		match self {
			Self::Just { provider, table } => (provider, *table),
			Self::Nothing => unreachable!(),
		}
	}

	pub fn pull(&mut self, until: u32, provider: &mut IndexProvider) -> Place {
		let (provider, table) = self.get_or_create(provider);

		Place::Overflow {
			table,
			index: provider.pull(until).try_into().unwrap(),
		}
	}

	pub fn try_revive(&mut self, table: Name, index: u16, until: u32) -> bool {
		if let Self::Just {
			provider,
			table: other,
		} = self
		{
			table.id == other.id && provider.try_revive(index.into(), until)
		} else {
			false
		}
	}

	pub fn push_until(&mut self, end: u32) {
		let Self::Just { provider, .. } = self else {
			return;
		};

		provider.push_until(end);
	}

	pub fn try_into_created(self) -> Option<(Name, u32)> {
		match self {
			Self::Just { provider, table } => Some((table, provider.get_names())),
			Self::Nothing => None,
		}
	}
}
