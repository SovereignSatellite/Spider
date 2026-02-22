use hashbrown::{HashMap, hash_map::Entry};
use ir_graph::Link;
use luajit_tree::expression::{Local, Name};

use super::index_provider::IndexProvider;

// LuaJIT has a max amount of local variables and no register allocator, so we must
// decide to spill to our own table per function after this count is reached.
const MAX_LOCAL_VARIABLES: usize = 197;

pub struct LocalProvider {
	local_provider: IndexProvider,
	table_provider: IndexProvider,
}

impl LocalProvider {
	pub const fn new() -> Self {
		Self {
			local_provider: IndexProvider::new(),
			table_provider: IndexProvider::new(),
		}
	}

	pub fn clear(&mut self) {
		self.local_provider.set_names(0);
		self.local_provider.forget_names();

		self.table_provider.set_names(0);
		self.table_provider.forget_names();
	}

	pub fn refresh(&mut self) {
		self.local_provider.forget_names();

		self.table_provider.set_names(0);
		self.table_provider.forget_names();
	}

	pub const fn get_names(&self) -> (u32, u32) {
		(
			self.local_provider.get_names(),
			self.table_provider.get_names(),
		)
	}

	fn pull(&mut self, start: u32) -> Local {
		if self.local_provider.should_exceed(MAX_LOCAL_VARIABLES) {
			let offset = self.table_provider.pull(start).try_into().unwrap();

			Local::Slow { offset }
		} else {
			let name = Name {
				id: self.local_provider.pull(start),
			};

			Local::Fast { name }
		}
	}

	fn try_revive(&mut self, local: Local, start: u32) -> bool {
		match local {
			Local::Fast { name } => self.local_provider.try_revive(name.id, start),
			Local::Slow { offset } => self.table_provider.try_revive(offset.into(), start),
		}
	}

	pub fn try_revive_into(
		&mut self,
		assignments: &mut HashMap<Link, Local>,
		destination: Link,
		preferred: Link,
	) -> bool {
		let Some(&local) = assignments.get(&preferred) else {
			return false;
		};

		let Entry::Vacant(entry) = assignments.entry(destination) else {
			return true;
		};

		if !self.try_revive(local, destination.0) {
			return false;
		}

		entry.insert(local);

		true
	}

	pub fn try_pull_into(&mut self, assignments: &mut HashMap<Link, Local>, link: Link) {
		let Entry::Vacant(entry) = assignments.entry(link) else {
			return;
		};

		let local = self.pull(link.0);

		entry.insert(local);
	}

	pub fn push_until(&mut self, start: u32) {
		self.local_provider.push_until(start);
		self.table_provider.push_until(start);
	}
}
