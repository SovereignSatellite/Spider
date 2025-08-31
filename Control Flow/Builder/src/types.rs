use alloc::vec::Vec;
use wasmparser::{BlockType, FuncType, RecGroup, SectionLimited, SubType};

pub struct Types {
	sub_types: Vec<SubType>,
	functions: Vec<u32>,
}

impl Types {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			sub_types: Vec::new(),
			functions: Vec::new(),
		}
	}

	pub fn clear(&mut self) {
		self.sub_types.clear();
		self.functions.clear();
	}

	pub fn add_sub_types(&mut self, section: SectionLimited<RecGroup>) {
		for group in section.into_iter().map(Result::unwrap) {
			self.sub_types.extend(group.into_types());
		}
	}

	pub fn add_function(&mut self, function: u32) {
		self.functions.push(function);
	}

	pub fn add_functions(&mut self, section: SectionLimited<u32>) {
		self.functions
			.extend(section.into_iter().map(Result::unwrap));
	}

	#[expect(clippy::missing_panics_doc)]
	#[must_use]
	pub fn get_function_index(&self, function: u32) -> u32 {
		self.functions[usize::try_from(function).unwrap()]
	}

	#[expect(clippy::missing_panics_doc)]
	#[must_use]
	pub fn get_type(&self, r#type: u32) -> &SubType {
		&self.sub_types[usize::try_from(r#type).unwrap()]
	}

	#[must_use]
	pub fn get_function_type(&self, function: u32) -> &FuncType {
		let function = self.get_function_index(function);

		self.get_type(function).unwrap_func()
	}

	#[must_use]
	pub fn get_parameter_count(&self, block_type: BlockType) -> usize {
		match block_type {
			BlockType::Empty | BlockType::Type(_) => 0,
			BlockType::FuncType(r#type) => self.get_type(r#type).unwrap_func().params().len(),
		}
	}

	#[must_use]
	pub fn get_result_count(&self, block_type: BlockType) -> usize {
		match block_type {
			BlockType::Empty => 0,
			BlockType::Type(_) => 1,
			BlockType::FuncType(r#type) => self.get_type(r#type).unwrap_func().results().len(),
		}
	}
}

impl Default for Types {
	fn default() -> Self {
		Self::new()
	}
}
