#![no_std]
#![expect(clippy::missing_panics_doc)]

extern crate alloc;

mod basic_block;
mod dot;

pub mod instruction;

use alloc::vec::Vec;

use self::instruction::{I32Constant, Instruction, LocalBranch, Name};

pub use self::{basic_block::BasicBlock, dot::Dot};

/// A directed graph of basic blocks containing instructions.
///
/// Note that after construction, it is expected that the following invariants hold:
///
/// * All branches are diamond shaped
/// * All loops are tail controlled
/// * Without back-edges, the graph is in topological order
pub struct ControlFlowGraph {
	pub instructions: Vec<Instruction>,
	pub basic_blocks: Vec<BasicBlock>,
}

impl ControlFlowGraph {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			instructions: Vec::new(),
			basic_blocks: Vec::new(),
		}
	}

	#[must_use]
	pub fn block_ids(&self) -> core::ops::Range<u16> {
		0..self.basic_blocks.len().try_into().unwrap()
	}

	#[must_use]
	pub fn offsets(&self, id: u16) -> core::ops::Range<usize> {
		self.basic_blocks[usize::from(id)].range()
	}

	#[must_use]
	pub fn instructions(&self, id: u16) -> &[Instruction] {
		let offsets = self.offsets(id);

		&self.instructions[offsets]
	}

	pub fn predecessors(&self, id: u16) -> impl Iterator<Item = u16> + '_ {
		self.basic_blocks[usize::from(id)]
			.predecessors
			.iter()
			.copied()
	}

	pub fn predecessors_acyclic(&self, id: u16) -> impl Iterator<Item = u16> + '_ {
		self.predecessors(id).filter(move |&id_2| id > id_2)
	}

	pub fn successors(&self, id: u16) -> impl Iterator<Item = u16> + '_ {
		self.basic_blocks[usize::from(id)]
			.successors
			.iter()
			.copied()
	}

	pub fn successors_acyclic(&self, id: u16) -> impl Iterator<Item = u16> + '_ {
		self.successors(id).filter(move |&id_2| id < id_2)
	}

	#[must_use]
	pub fn is_branch_start(&self, id: u16) -> bool {
		let mut successors = self.successors_acyclic(id);

		successors.next().is_some() && successors.next().is_some()
	}

	#[must_use]
	pub fn is_branch_end(&self, id: u16) -> bool {
		let mut predecessors = self.predecessors_acyclic(id);

		predecessors.next().is_some() && predecessors.next().is_some()
	}

	#[must_use]
	pub fn find_branch_start(&self, id: u16) -> Option<u16> {
		self.predecessors_acyclic(id)
			.find(|&id_2| self.is_branch_start(id_2))
	}

	#[must_use]
	pub fn find_branch_end(&self, id: u16) -> Option<u16> {
		self.successors_acyclic(id)
			.find(|&id_2| self.is_branch_end(id_2))
	}

	#[must_use]
	pub fn find_repeat_start(&self, id: u16) -> Option<u16> {
		self.successors(id).find(|&id_2| id >= id_2)
	}

	#[must_use]
	pub fn find_repeat_end(&self, id: u16) -> Option<u16> {
		self.predecessors(id).find(|&id_2| id <= id_2)
	}

	#[must_use]
	pub fn has_single_entry(&self) -> bool {
		let mut basic_blocks = self.basic_blocks.iter();

		basic_blocks.any(BasicBlock::is_source) && !basic_blocks.any(BasicBlock::is_source)
	}

	#[must_use]
	pub fn has_repeats(&self) -> bool {
		self.block_ids()
			.any(|id| self.find_repeat_end(id).is_some())
	}

	pub fn add_edge(&mut self, from: u16, to: u16) {
		let from_usize = usize::from(from);
		let to_usize = usize::from(to);

		self.basic_blocks[to_usize].predecessors.push(from);
		self.basic_blocks[from_usize].successors.push(to);
	}

	pub fn replace_edge(&mut self, from: u16, to: u16, new: u16) {
		let from_usize = usize::from(from);
		let to_usize = usize::from(to);
		let new_usize = usize::from(new);

		let successor = self.successors(from).position(|id| id == to).unwrap();

		self.basic_blocks[from_usize].successors[successor] = new;
		self.basic_blocks[new_usize].predecessors.push(from);

		let predecessor = self.predecessors(to).position(|id| id == from).unwrap();

		self.basic_blocks[to_usize].predecessors.remove(predecessor);
	}

	fn add_instruction(&mut self, instruction: Instruction) -> u16 {
		let id = self.basic_blocks.len().try_into().unwrap();
		let position = self.instructions.len().try_into().unwrap();

		self.instructions.push(instruction);

		self.basic_blocks
			.push(BasicBlock::from_range(position, position + 1));

		id
	}

	#[must_use]
	pub fn has_assignment(&self, id: u16, name: Name) -> bool {
		matches!(self.instructions(id), &[Instruction::I32Constant(I32Constant { destination, .. })] if destination == name as u16)
	}

	pub fn add_no_operation(&mut self) -> u16 {
		let id = self.basic_blocks.len().try_into().unwrap();
		let position = self.instructions.len().try_into().unwrap();

		self.basic_blocks
			.push(BasicBlock::from_range(position, position));

		id
	}

	pub fn add_selection(&mut self, name: Name) -> u16 {
		let local_branch = Instruction::LocalBranch(LocalBranch {
			source: name as u16,
		});

		self.add_instruction(local_branch)
	}

	pub fn add_assignment(&mut self, name: Name, value: u16) -> u16 {
		let i32_constant = Instruction::I32Constant(I32Constant {
			destination: name as u16,
			data: value.into(),
		});

		self.add_instruction(i32_constant)
	}
}

impl Default for ControlFlowGraph {
	fn default() -> Self {
		Self::new()
	}
}
