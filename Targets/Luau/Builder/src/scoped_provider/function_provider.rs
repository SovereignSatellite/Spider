use luau_tree::expression::Name;

use crate::place::Place;

use super::{index_provider::IndexProvider, table_provider::TableProvider};

// Luau has a max amount of local variables and no register allocator, so we must
// decide to spill to our own table per function after this count is reached.
const MAX_LOCAL_VARIABLES: usize = 199;

pub struct FunctionProvider {
	pub local_provider: IndexProvider,
	pub table_provider: TableProvider,
}

impl FunctionProvider {
	pub const fn new() -> Self {
		Self {
			local_provider: IndexProvider::new(),
			table_provider: TableProvider::Nothing,
		}
	}

	fn pull_fast(&mut self, until: u32) -> Place {
		let declares = self.local_provider.should_create();
		let name = Name {
			id: self.local_provider.pull(until),
		};

		if declares {
			Place::Definition { name }
		} else {
			Place::Assignment { name }
		}
	}

	fn pull_slow(&mut self, until: u32) -> Place {
		self.table_provider.pull(until, &mut self.local_provider)
	}

	pub fn pull(&mut self, until: u32) -> Place {
		if self.local_provider.should_exceed(MAX_LOCAL_VARIABLES) {
			self.pull_slow(until)
		} else {
			self.pull_fast(until)
		}
	}

	pub fn try_revive(&mut self, place: Place, until: u32) -> bool {
		match place {
			Place::Definition { name } | Place::Assignment { name } => {
				self.local_provider.try_revive(name.id, until)
			}
			Place::Overflow { table, index } => self.table_provider.try_revive(table, index, until),
		}
	}

	pub fn push_until(&mut self, end: u32) {
		self.local_provider.push_until(end);
		self.table_provider.push_until(end);
	}
}
