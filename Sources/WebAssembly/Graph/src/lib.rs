//! A WebAssembly control flow graph representation.

#![no_std]

extern crate alloc;

mod basic_block;
mod dot;

/// Instruction types used in the control flow graph.
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
	/// The instructions stored across all basic blocks.
	pub instructions: Vec<Instruction>,
	/// The basic blocks in the graph.
	pub basic_blocks: Vec<BasicBlock>,
}

impl ControlFlowGraph {
	/// Creates a new empty control flow graph.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			instructions: Vec::new(),
			basic_blocks: Vec::new(),
		}
	}

	/// Returns a range of all basic block identifiers.
	///
	/// # Panics
	///
	/// Panics if the number of basic blocks exceeds `u16::MAX`.
	#[must_use]
	pub fn block_ids(&self) -> core::ops::Range<u16> {
		0..self.basic_blocks.len().try_into().unwrap()
	}

	/// Returns the instruction offset range for the given basic block.
	#[must_use]
	pub fn offsets(&self, id: u16) -> core::ops::Range<usize> {
		self.basic_blocks[usize::from(id)].range()
	}

	/// Returns the instructions in the given basic block.
	#[must_use]
	pub fn instructions(&self, id: u16) -> &[Instruction] {
		let offsets = self.offsets(id);

		&self.instructions[offsets]
	}

	/// Returns an iterator over the predecessor block identifiers.
	pub fn predecessors(&self, id: u16) -> impl Iterator<Item = u16> + '_ {
		self.basic_blocks[usize::from(id)]
			.predecessors
			.iter()
			.copied()
	}

	/// Returns an iterator over the acyclic predecessor block identifiers.
	pub fn predecessors_acyclic(&self, id: u16) -> impl Iterator<Item = u16> + '_ {
		self.predecessors(id).filter(move |&id_2| id > id_2)
	}

	/// Returns an iterator over the successor block identifiers.
	pub fn successors(&self, id: u16) -> impl Iterator<Item = u16> + '_ {
		self.basic_blocks[usize::from(id)]
			.successors
			.iter()
			.copied()
	}

	/// Returns an iterator over the acyclic successor block identifiers.
	pub fn successors_acyclic(&self, id: u16) -> impl Iterator<Item = u16> + '_ {
		self.successors(id).filter(move |&id_2| id < id_2)
	}

	/// Returns `true` if the given block is the start of a branch (diamond).
	#[must_use]
	pub fn is_branch_start(&self, id: u16) -> bool {
		let mut successors = self.successors_acyclic(id);

		successors.next().is_some() && successors.next().is_some()
	}

	/// Returns `true` if the given block is the end of a branch (diamond).
	#[must_use]
	pub fn is_branch_end(&self, id: u16) -> bool {
		let mut predecessors = self.predecessors_acyclic(id);

		predecessors.next().is_some() && predecessors.next().is_some()
	}

	/// Finds the start of a branch (diamond) containing the given block.
	#[must_use]
	pub fn find_branch_start(&self, id: u16) -> Option<u16> {
		self.predecessors_acyclic(id)
			.find(|&id_2| self.is_branch_start(id_2))
	}

	/// Finds the end of a branch (diamond) containing the given block.
	#[must_use]
	pub fn find_branch_end(&self, id: u16) -> Option<u16> {
		self.successors_acyclic(id)
			.find(|&id_2| self.is_branch_end(id_2))
	}

	/// Finds the start of a repeat (loop) containing the given block.
	#[must_use]
	pub fn find_repeat_start(&self, id: u16) -> Option<u16> {
		self.successors(id).find(|&id_2| id >= id_2)
	}

	/// Finds the end of a repeat (loop) containing the given block.
	#[must_use]
	pub fn find_repeat_end(&self, id: u16) -> Option<u16> {
		self.predecessors(id).find(|&id_2| id <= id_2)
	}

	/// Returns `true` if the graph has exactly one entry block.
	#[must_use]
	pub fn has_single_entry(&self) -> bool {
		let mut basic_blocks = self.basic_blocks.iter();

		basic_blocks.any(BasicBlock::is_source) && !basic_blocks.any(BasicBlock::is_source)
	}

	/// Returns `true` if the graph contains any repeat (loop) edges.
	#[must_use]
	pub fn has_repeats(&self) -> bool {
		self.block_ids()
			.any(|id| self.find_repeat_end(id).is_some())
	}

	/// Adds an edge between two basic blocks.
	pub fn add_edge(&mut self, from: u16, to: u16) {
		let from_usize = usize::from(from);
		let to_usize = usize::from(to);

		self.basic_blocks[to_usize].predecessors.push(from);
		self.basic_blocks[from_usize].successors.push(to);
	}

	/// Replaces an edge from one block to another with a new target.
	///
	/// # Panics
	///
	/// Panics if the edge from `from` to `to` does not exist.
	pub fn replace_edge(&mut self, from: u16, to: u16, new: u16) {
		let from_usize = usize::from(from);
		let to_usize = usize::from(to);
		let new_usize = usize::from(new);

		let successor = self.successors(from).position(|id| id == to).unwrap();

		self.basic_blocks[from_usize].successors[successor] = new;
		self.basic_blocks[new_usize].predecessors.push(from);

		let predecessor = self.predecessors(to).position(|id| id == from).unwrap();

		let _removed = self.basic_blocks[to_usize].predecessors.remove(predecessor);
	}

	fn add_instruction(&mut self, instruction: Instruction) -> u16 {
		let id = self.basic_blocks.len().try_into().unwrap();
		let position = self.instructions.len().try_into().unwrap();

		self.instructions.push(instruction);

		self.basic_blocks
			.push(BasicBlock::from_range(position, position + 1));

		id
	}

	/// Returns `true` if the given block has an assignment to the named variable.
	#[must_use]
	pub fn has_assignment(&self, id: u16, name: Name) -> bool {
		matches!(self.instructions(id), &[Instruction::I32Constant(I32Constant { destination, .. })] if destination == name as u16)
	}

	/// Adds a no-operation basic block and returns its block identifier.
	///
	/// # Panics
	///
	/// Panics if the number of basic blocks exceeds `u16::MAX`.
	pub fn add_no_operation(&mut self) -> u16 {
		let id = self.basic_blocks.len().try_into().unwrap();
		let position = self.instructions.len().try_into().unwrap();

		self.basic_blocks
			.push(BasicBlock::from_range(position, position));

		id
	}

	/// Adds a selection (branch) instruction and returns its block identifier.
	///
	/// # Panics
	///
	/// Panics if the number of basic blocks or instructions exceeds `u16::MAX`.
	pub fn add_selection(&mut self, name: Name) -> u16 {
		let local_branch = Instruction::LocalBranch(LocalBranch {
			source: name as u16,
		});

		self.add_instruction(local_branch)
	}

	/// Adds an assignment instruction and returns its block identifier.
	///
	/// # Panics
	///
	/// Panics if the number of basic blocks or instructions exceeds `u16::MAX`.
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
