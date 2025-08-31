use alloc::vec::Vec;
use control_flow_graph::BasicBlock;
use set::Set;

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

	fn add_successor(&mut self, id: u16) {
		if self.seen.contains(id.into()) {
			return;
		}

		self.stack.push((id, false));
	}

	fn run<H>(&mut self, basic_blocks: &mut [BasicBlock], start: u16, mut handler: H)
	where
		H: FnMut(&mut [BasicBlock], u16),
	{
		self.seen.clear();

		self.add_successor(start);

		while let Some((id, post)) = self.stack.pop() {
			let id_usize = id.into();

			if post {
				handler(basic_blocks, id);
			} else if !self.seen.grow_insert(id_usize) {
				self.stack.push((id, true));

				for &id in &basic_blocks[id_usize].successors {
					self.add_successor(id);
				}
			}
		}
	}
}

pub struct Sorter {
	basic_blocks: Vec<BasicBlock>,
	id_to_post: Vec<u16>,

	depth_first_searcher: DepthFirstSearcher,
}

impl Sorter {
	pub const fn new() -> Self {
		Self {
			basic_blocks: Vec::new(),
			id_to_post: Vec::new(),

			depth_first_searcher: DepthFirstSearcher::new(),
		}
	}

	fn find_basic_blocks(&mut self, basic_blocks: &mut [BasicBlock], entry: u16) -> u16 {
		let mut post = 0;

		self.basic_blocks.clear();
		self.id_to_post.clear();
		self.id_to_post.resize(basic_blocks.len(), u16::MAX);

		self.depth_first_searcher
			.run(basic_blocks, entry, |basic_blocks, id| {
				let id = usize::from(id);
				let basic_block = core::mem::take(&mut basic_blocks[id]);

				self.basic_blocks.push(basic_block);
				self.id_to_post[id] = post;

				post += 1;
			});

		post
	}

	fn patch_post(&mut self, post: u16) {
		self.id_to_post
			.iter_mut()
			.filter(|post| **post != u16::MAX)
			.for_each(|old| *old = post - 1 - *old);
	}

	fn patch_references(&mut self, basic_blocks: &mut Vec<BasicBlock>) {
		basic_blocks.clear();
		basic_blocks.extend(self.basic_blocks.drain(..).rev());

		for basic_block in basic_blocks {
			basic_block.replace_ids(|id| self.id_to_post[usize::from(id)]);
		}
	}

	pub fn run(&mut self, basic_blocks: &mut Vec<BasicBlock>, entry: u16) {
		let post = self.find_basic_blocks(basic_blocks, entry);

		self.patch_post(post);
		self.patch_references(basic_blocks);
	}
}
