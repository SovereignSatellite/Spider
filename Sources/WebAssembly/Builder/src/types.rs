//! WebAssembly type information for function signatures and block types.

use wasmparser::{BlockType, FuncType, RecGroup, SectionLimited, SubType};

/// WebAssembly type information.
pub struct Types {
	sub_types: Vec<SubType>,
	functions: Vec<u32>,
}

impl Types {
	#[must_use]
	/// Creates a new empty type collection.
	pub const fn new() -> Self {
		Self {
			sub_types: Vec::new(),
			functions: Vec::new(),
		}
	}

	/// Clears all stored type information.
	pub fn clear(&mut self) {
		self.sub_types.clear();
		self.functions.clear();
	}

	/// Adds sub types from a recursion group section.
	pub fn add_sub_types(&mut self, section: SectionLimited<'_, RecGroup>) {
		for group in section.into_iter().map(Result::unwrap) {
			self.sub_types.extend(group.into_types());
		}
	}

	/// Adds a function type index.
	pub fn add_function(&mut self, function: u32) {
		self.functions.push(function);
	}

	/// Adds function type indices from a section.
	pub fn add_functions(&mut self, section: SectionLimited<'_, u32>) {
		self.functions
			.extend(section.into_iter().map(Result::unwrap));
	}

	#[must_use]
	/// Returns the type index for a function.
	pub fn get_function_index(&self, function: u32) -> u32 {
		self.functions[usize::try_from(function).unwrap_or_else(|_| unreachable!())]
	}

	#[must_use]
	/// Returns the sub type at the given index.
	pub fn get_type(&self, index: u32) -> &SubType {
		&self.sub_types[usize::try_from(index).unwrap_or_else(|_| unreachable!())]
	}

	#[must_use]
	/// Returns the function type for a function.
	pub fn get_function_type(&self, function: u32) -> &FuncType {
		let index = self.get_function_index(function);

		self.get_type(index).unwrap_func()
	}

	#[must_use]
	/// Returns the parameter and result counts for the function type at the given index.
	pub fn get_arity(&self, index: u32) -> (u16, u16) {
		let function = self.get_type(index).unwrap_func();
		let parameters = function.params().len();
		let results = function.results().len();

		(
			parameters.try_into().unwrap_or_else(|_| unreachable!()),
			results.try_into().unwrap_or_else(|_| unreachable!()),
		)
	}

	#[must_use]
	/// Returns the parameter count for a block type.
	pub fn get_parameter_count(&self, block_type: BlockType) -> usize {
		match block_type {
			BlockType::Empty | BlockType::Type(_) => 0,
			BlockType::FuncType(kind) => self.get_type(kind).unwrap_func().params().len(),
		}
	}

	#[must_use]
	/// Returns the result count for a block type.
	pub fn get_result_count(&self, block_type: BlockType) -> usize {
		match block_type {
			BlockType::Empty => 0,
			BlockType::Type(_) => 1,
			BlockType::FuncType(kind) => self.get_type(kind).unwrap_func().results().len(),
		}
	}
}

impl Default for Types {
	fn default() -> Self {
		Self::new()
	}
}
