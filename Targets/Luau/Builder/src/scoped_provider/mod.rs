use alloc::vec::Vec;
use luau_tree::expression::Name;

use crate::place::Place;

use self::function_provider::FunctionProvider;

mod function_provider;
mod index_provider;
mod table_provider;

pub struct ScopedProvider {
	function_providers: Vec<FunctionProvider>,
	first_names: Vec<u32>,
}

impl ScopedProvider {
	pub const fn new() -> Self {
		Self {
			function_providers: Vec::new(),
			first_names: Vec::new(),
		}
	}

	pub fn pop_local_scope(&mut self) {
		let FunctionProvider { local_provider, .. } = self.function_providers.last_mut().unwrap();

		local_provider.forget_free(self.first_names.pop().unwrap());
	}

	pub fn push_local_scope(&mut self) {
		let FunctionProvider { local_provider, .. } = self.function_providers.last().unwrap();

		self.first_names.push(local_provider.get_names());
	}

	pub fn pop_function_scope(&mut self) -> Option<(Name, u32)> {
		let FunctionProvider {
			local_provider,
			table_provider,
		} = self.function_providers.pop().unwrap();

		if let Some(function_provider) = self.function_providers.last_mut() {
			function_provider
				.local_provider
				.set_names(local_provider.get_names());
		}

		table_provider.try_into_created()
	}

	pub fn push_function_scope(&mut self) {
		let mut function_provider = FunctionProvider::new();

		if let Some(FunctionProvider { local_provider, .. }) = self.function_providers.last() {
			function_provider
				.local_provider
				.set_names(local_provider.get_names());
		}

		self.function_providers.push(function_provider);
	}

	pub fn pull(&mut self, until: u32) -> Place {
		let function_provider = self.function_providers.last_mut().unwrap();

		function_provider.pull(until)
	}

	pub fn try_revive(&mut self, place: Place, until: u32) -> bool {
		let function_provider = self.function_providers.last_mut().unwrap();

		function_provider.try_revive(place, until)
	}

	pub fn push_until(&mut self, end: u32) {
		let function_provider = self.function_providers.last_mut().unwrap();

		function_provider.push_until(end);
	}
}
