//! Post-order sorting of basic blocks using depth-first search.

use alloc::vec::Vec;
use set::Set;
use web_assembly_graph::BasicBlock;

struct DepthFirstSearcher {
	seen: Set,
	stack: Vec<(u16, bool)>,
}

impl DepthFirstSearcher {
	const fn new() -> Self {
		Self {
			seen: Set::new(),
			stack: Vec::new(),
		}
	}

	fn add_successor(&mut self, block_id: u16) {
		if self.seen.contains(block_id.into()) {
			return;
		}

		self.stack.push((block_id, false));
	}

	fn run<H>(&mut self, basic_blocks: &mut [BasicBlock], start: u16, mut handler: H)
	where
		H: FnMut(&mut [BasicBlock], u16),
	{
		self.seen.clear();

		self.add_successor(start);

		while let Some((block_id, visited)) = self.stack.pop() {
			let block_id_usize = block_id.into();

			if self.seen.grow_insert(block_id_usize) {
				if visited {
					handler(basic_blocks, block_id);
				}
			} else {
				let BasicBlock { successors, .. } = &basic_blocks[block_id_usize];

				self.stack.push((block_id, true));

				for &successor_id in successors {
					self.add_successor(successor_id);
				}
			}
		}
	}
}

pub struct PostOrderSorter {
	basic_blocks: Vec<BasicBlock>,
	id_to_post: Vec<u16>,

	depth_first_searcher: DepthFirstSearcher,
}

impl PostOrderSorter {
	pub const fn new() -> Self {
		Self {
			basic_blocks: Vec::new(),
			id_to_post: Vec::new(),

			depth_first_searcher: DepthFirstSearcher::new(),
		}
	}

	fn find_basic_blocks(&mut self, basic_blocks: &mut [BasicBlock], entry: u16) -> u16 {
		let mut post_index = 0;

		self.basic_blocks.clear();
		self.id_to_post.clear();
		self.id_to_post.resize(basic_blocks.len(), u16::MAX);

		self.depth_first_searcher
			.run(basic_blocks, entry, |basic_blocks, block_id| {
				let block_id_usize = usize::from(block_id);
				let basic_block = core::mem::take(&mut basic_blocks[block_id_usize]);

				self.basic_blocks.push(basic_block);
				self.id_to_post[block_id_usize] = post_index;

				post_index += 1;
			});

		post_index
	}

	fn patch_post(&mut self, post_count: u16) {
		self.id_to_post
			.iter_mut()
			.filter(|index| **index != u16::MAX)
			.for_each(|old| *old = post_count - 1 - *old);
	}

	fn patch_references(&mut self, basic_blocks: &mut Vec<BasicBlock>) {
		basic_blocks.clear();
		basic_blocks.extend(self.basic_blocks.drain(..).rev());

		for basic_block in basic_blocks {
			basic_block.replace_ids(|block_id| self.id_to_post[usize::from(block_id)]);
		}
	}

	pub fn run(&mut self, basic_blocks: &mut Vec<BasicBlock>, entry: u16) {
		let post_count = self.find_basic_blocks(basic_blocks, entry);

		self.patch_post(post_count);
		self.patch_references(basic_blocks);
	}
}
