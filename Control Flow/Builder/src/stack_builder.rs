use alloc::vec::Vec;
use control_flow_graph::instruction::Name;
use list::resizable::Resizable;
use wasmparser::{BlockType, FuncType};

use crate::types::Types;

pub const SHARED_LOCAL: u16 = Name::D as u16;
pub const LOCAL_BASE: u16 = Name::COUNT;

pub struct Jump {
	pub stack: u16,
	pub source: u16,
	pub branch: u16,
}

pub struct Level {
	pub parameters: u16,
	pub results: u16,
	pub base: u16,

	pub destination: Option<u16>,
	pub jumps: Resizable<Jump, 4>,
}

pub struct StackBuilder {
	levels: Vec<Level>,

	top: u16,
}

impl StackBuilder {
	pub const fn new() -> Self {
		Self {
			levels: Vec::new(),

			top: 0,
		}
	}

	pub fn set_function_data(&mut self, types: &Types, function_type: BlockType, locals: u16) {
		let parameters = types.get_parameter_count(function_type).try_into().unwrap();
		let results = types.get_result_count(function_type).try_into().unwrap();

		self.top = LOCAL_BASE;

		self.levels.push(Level {
			parameters,
			results,
			base: self.top,

			destination: None,
			jumps: Resizable::new(),
		});

		self.top += parameters + locals;
	}

	pub fn push_level(&mut self, types: &Types, block_type: BlockType, destination: Option<u16>) {
		let parameters = types.get_parameter_count(block_type).try_into().unwrap();
		let results = types.get_result_count(block_type).try_into().unwrap();

		self.levels.push(Level {
			parameters,
			results,
			base: self.top - parameters,

			destination,
			jumps: Resizable::new(),
		});
	}

	pub fn pull_level(&mut self) -> Level {
		let level @ Level { base, results, .. } = self.levels.pop().unwrap();

		self.top = base + results;

		level
	}

	pub fn peek_level_mut(&mut self) -> &mut Level {
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

	pub fn load_function_type(&mut self, r#type: &FuncType) -> ((u16, u16), (u16, u16)) {
		let sources = self.pull_locals(r#type.params().len().try_into().unwrap());
		let destinations = self.push_locals(r#type.results().len().try_into().unwrap());

		(destinations, sources)
	}

	pub const fn get_top(&self) -> u16 {
		self.top
	}

	pub const fn set_top(&mut self, top: u16) {
		self.top = top;
	}

	pub fn jump_to_level(&mut self, source: u16, branch: usize, destination: usize) {
		let branch = branch.try_into().unwrap();

		self.levels[destination].jumps.push(Jump {
			stack: self.top,
			source,
			branch,
		});
	}

	pub fn jump_to_depth(&mut self, source: u16, branch: usize, depth: u32) {
		let depth = usize::try_from(depth).unwrap();

		self.jump_to_level(source, branch, self.levels.len() - depth - 1);
	}
}
