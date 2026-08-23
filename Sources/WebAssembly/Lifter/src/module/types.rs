use wasmparser::{BlockType, FuncType, RecGroup, SectionLimited, SubType};

pub struct TypeRegistry {
	sub_types: Vec<SubType>,
	functions: Vec<u32>,
}

impl TypeRegistry {
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

	pub fn add_sub_types(&mut self, section: SectionLimited<'_, RecGroup>) {
		for group in section.into_iter().map(Result::unwrap) {
			self.sub_types.extend(group.into_types());
		}
	}

	pub fn add_function(&mut self, function: u32) {
		self.functions.push(function);
	}

	pub fn add_functions(&mut self, section: SectionLimited<'_, u32>) {
		self.functions
			.extend(section.into_iter().map(Result::unwrap));
	}

	#[must_use]
	pub fn get_function_index(&self, function: u32) -> u32 {
		self.functions[usize::try_from(function).unwrap_or_else(|_| unreachable!())]
	}

	#[must_use]
	pub fn get_type(&self, index: u32) -> &SubType {
		&self.sub_types[usize::try_from(index).unwrap_or_else(|_| unreachable!())]
	}

	#[must_use]
	pub fn get_function_type(&self, function: u32) -> &FuncType {
		let index = self.get_function_index(function);

		self.get_type(index).unwrap_func()
	}

	#[must_use]
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
	pub fn get_parameter_count(&self, block_type: BlockType) -> usize {
		match block_type {
			BlockType::Empty | BlockType::Type(_) => 0,
			BlockType::FuncType(kind) => self.get_type(kind).unwrap_func().params().len(),
		}
	}

	#[must_use]
	pub fn get_result_count(&self, block_type: BlockType) -> usize {
		match block_type {
			BlockType::Empty => 0,
			BlockType::Type(_) => 1,
			BlockType::FuncType(kind) => self.get_type(kind).unwrap_func().results().len(),
		}
	}
}
