use core::fmt::{Debug, Formatter, Result as FormatResult};
use core::ops::ControlFlow;

use arbitrary::{Arbitrary, Result, Unstructured};

const OPERATORS: [char; 6] = ['>', '<', '+', '-', ',', '.'];

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
		let block = Block::arbitrary(u)?;

		Ok(Self { block })
	}
}

impl SupportedSource {
	#[must_use]
	pub fn to_bytes(&self) -> Vec<u8> {
		let mut source = String::new();

		self.block.write(&mut source);

		source.into_bytes()
	}
}

struct Block {
	items: Vec<Item>,
}

impl<'data> Arbitrary<'data> for Block {
	fn arbitrary(u: &mut Unstructured<'data>) -> Result<Self> {
		let mut items = Vec::new();

		u.arbitrary_loop(None, None, |data| {
			items.push(Item::arbitrary(data)?);

			Ok(ControlFlow::Continue(()))
		})?;

		Ok(Self { items })
	}
}

impl Block {
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

impl<'data> Arbitrary<'data> for Item {
	fn arbitrary(u: &mut Unstructured<'data>) -> Result<Self> {
		let variant = u.int_in_range(0..=OPERATORS.len())?;

		match OPERATORS.get(variant) {
			Some(&operator) => Ok(Self::Operator(operator)),
			None => Block::arbitrary(u).map(Self::Loop),
		}
	}
}

impl Item {
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
