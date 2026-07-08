//! Eliminates unreachable blocks and sorts the survivors in reverse post-order.

use core::mem;

use web_assembly_graph::BasicBlock;

/// Eliminate unreachable blocks and compact the remainder in reverse
/// post-order.
pub struct TopologicalCompactor {
	blocks: Vec<BasicBlock>,
	ids: Vec<u16>,
}

impl TopologicalCompactor {
	/// Creates a new compactor.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			blocks: Vec::new(),
			ids: Vec::new(),
		}
	}

	fn handle_block(&mut self, blocks: &mut Vec<BasicBlock>, id: u16) {
		let id = usize::from(id);

		if self.ids[id] != u16::MAX {
			return;
		}

		self.ids[id] = u16::MAX - 1;

		let block = mem::take(&mut blocks[id]);

		stacker::maybe_grow(0x1_0000, 0x10_0000, || {
			for &successor in block.successors.iter().rev() {
				self.handle_block(blocks, successor);
			}
		});

		self.ids[id] = self.blocks.len().try_into().unwrap();

		self.blocks.push(block);
	}

	fn handle_blocks(&mut self, blocks: &mut Vec<BasicBlock>, entry: u16) {
		self.blocks.clear();

		self.ids.clear();
		self.ids.resize(blocks.len(), u16::MAX);

		self.handle_block(blocks, entry);

		self.blocks.reverse();

		let count: u16 = self.blocks.len().try_into().unwrap();

		for id in &mut self.ids {
			if *id != u16::MAX {
				*id = count - 1 - *id;
			}
		}

		mem::swap(blocks, &mut self.blocks);
	}

	fn handle_edges(&self, blocks: &mut [BasicBlock]) {
		for block in blocks {
			block.replace_ids(|id| self.ids[usize::from(id)]);
		}
	}

	/// Compacts a set of basic blocks in place, starting from the given entry.
	pub fn run(&mut self, blocks: &mut Vec<BasicBlock>, entry: u16) {
		self.handle_blocks(blocks, entry);
		self.handle_edges(blocks);
	}
}

impl Default for TopologicalCompactor {
	fn default() -> Self {
		Self::new()
	}
}
