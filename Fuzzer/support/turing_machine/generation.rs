use core::fmt::{Debug, Formatter, Result as FormatResult};
use core::ops::ControlFlow;

use arbitrary::{Arbitrary, Result, Unstructured};

const OPERATORS: [char; 6] = ['>', '<', '+', '-', ',', '.'];

const MAXIMUM_BLOCK_ITEMS: u32 = 256;
const MAXIMUM_SOURCE_ITEMS: u32 = 4096;

pub struct SupportedSource {
	block: Block,
}

impl Debug for SupportedSource {
	fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
		f.debug_struct("SupportedSource").finish_non_exhaustive()
	}
}

impl<'data> Arbitrary<'data> for SupportedSource {
	fn arbitrary(u: &mut Unstructured<'data>) -> Result<Self> {
		let mut budget = MAXIMUM_SOURCE_ITEMS;
		let block = Block::arbitrary(u, &mut budget)?;

		Ok(Self { block })
	}
}

impl SupportedSource {
	#[must_use]
	pub fn into_string(self) -> String {
		let mut source = String::new();

		self.block.write(&mut source);

		source
	}
}

struct Block {
	items: Vec<Item>,
}

impl Block {
	fn arbitrary(unstructured: &mut Unstructured<'_>, budget: &mut u32) -> Result<Self> {
		let mut items = Vec::new();

		unstructured.arbitrary_loop(None, Some(MAXIMUM_BLOCK_ITEMS), |data| {
			if *budget == 0 {
				return Ok(ControlFlow::Break(()));
			}

			*budget -= 1;

			items.push(Item::arbitrary(data, budget)?);

			Ok(ControlFlow::Continue(()))
		})?;

		Ok(Self { items })
	}

	fn write(&self, source: &mut String) {
		for item in &self.items {
			item.write(source);
		}
	}
}

enum Item {
	Operator(char),
	Loop(Block),
}

impl Item {
	fn arbitrary(unstructured: &mut Unstructured<'_>, budget: &mut u32) -> Result<Self> {
		let variant = unstructured.int_in_range(0..=OPERATORS.len())?;

		match OPERATORS.get(variant) {
			Some(&operator) => Ok(Self::Operator(operator)),
			None => Block::arbitrary(unstructured, budget).map(Self::Loop),
		}
	}

	fn write(&self, source: &mut String) {
		match self {
			Self::Operator(operator) => source.push(*operator),
			Self::Loop(block) => {
				source.push('[');
				block.write(source);
				source.push(']');
			}
		}
	}
}
