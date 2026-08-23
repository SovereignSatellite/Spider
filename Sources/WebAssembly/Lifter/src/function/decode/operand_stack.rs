use list::resizable::Resizable;
use wasmparser::{BlockType, FuncType};

use super::LOCAL_BASE;
use crate::module::TypeRegistry;

pub struct PendingJump {
	pub stack_top: u16,
	pub source_block: u16,
	pub successor_index: u16,
}

#[derive(Clone, Copy)]
pub enum ControlLevelKind {
	Forward,
	Loop { entry: u16 },
}

pub struct ControlLevel {
	pub parameters: u16,
	pub results: u16,
	pub base: u16,

	pub kind: ControlLevelKind,
	pub pending_jumps: Resizable<PendingJump, 4>,
}

pub struct OperandStack {
	levels: Vec<ControlLevel>,
	top: u16,
}

impl OperandStack {
	pub const fn new() -> Self {
		Self {
			levels: Vec::new(),
			top: 0,
		}
	}

	pub fn set_function_data(
		&mut self,
		types: &TypeRegistry,
		function_type: BlockType,
		locals: u16,
	) {
		let parameters = types.get_parameter_count(function_type).try_into().unwrap();
		let results = types.get_result_count(function_type).try_into().unwrap();

		self.top = LOCAL_BASE;

		self.levels.push(ControlLevel {
			parameters,
			results,
			base: self.top,

			kind: ControlLevelKind::Forward,
			pending_jumps: Resizable::new(),
		});

		self.top += parameters + locals;
	}

	pub fn push_level(
		&mut self,
		types: &TypeRegistry,
		block_type: BlockType,
		kind: ControlLevelKind,
	) {
		let parameters = types.get_parameter_count(block_type).try_into().unwrap();
		let results = types.get_result_count(block_type).try_into().unwrap();
		let base = self.top.wrapping_sub(parameters);

		self.levels.push(ControlLevel {
			parameters,
			results,
			base,

			kind,
			pending_jumps: Resizable::new(),
		});
	}

	pub fn pull_level(&mut self) -> ControlLevel {
		let level @ ControlLevel { base, results, .. } = self.levels.pop().unwrap();

		self.top = base.wrapping_add(results);

		level
	}

	pub fn peek_level_mut(&mut self) -> &mut ControlLevel {
		self.levels.last_mut().unwrap()
	}

	const fn push_locals(&mut self, count: u16) -> (u16, u16) {
		let top = self.top;

		self.top = top.wrapping_add(count);

		(top, self.top)
	}

	pub const fn push_local(&mut self) -> u16 {
		self.push_locals(1).0
	}

	const fn pull_locals(&mut self, count: u16) -> (u16, u16) {
		let top = self.top;

		self.top = top.wrapping_sub(count);

		(self.top, top)
	}

	pub const fn pull_local(&mut self) -> u16 {
		self.pull_locals(1).0
	}

	pub fn load_function_type(&mut self, kind: &FuncType) -> ((u16, u16), (u16, u16)) {
		let sources = self.pull_locals(kind.params().len().try_into().unwrap());
		let destinations = self.push_locals(kind.results().len().try_into().unwrap());

		(destinations, sources)
	}

	pub const fn get_top(&self) -> u16 {
		self.top
	}

	pub const fn peek_local(&self) -> u16 {
		self.top.wrapping_sub(1)
	}

	pub const fn load_unary_operation(&self) -> (u16, u16) {
		let source = self.peek_local();

		(source, source)
	}

	pub const fn load_binary_operation(&mut self) -> (u16, u16, u16) {
		let rhs = self.pull_local();
		let lhs = self.peek_local();

		(lhs, lhs, rhs)
	}

	pub const fn load_ternary_operation(&mut self) -> (u16, u16, u16, u16) {
		let third = self.pull_local();
		let second = self.pull_local();
		let first = self.peek_local();

		(first, first, second, third)
	}

	pub const fn set_top(&mut self, top: u16) {
		self.top = top;
	}

	pub fn jump_to_level(&mut self, source_block: u16, successor_index: usize, level_index: usize) {
		let successor_index = successor_index.try_into().unwrap();

		self.levels[level_index].pending_jumps.push(PendingJump {
			stack_top: self.top,
			source_block,
			successor_index,
		});
	}

	pub fn jump_to_depth(&mut self, source_block: u16, successor_index: usize, depth: u32) {
		let depth = usize::try_from(depth).unwrap();

		self.jump_to_level(source_block, successor_index, self.levels.len() - depth - 1);
	}
}
