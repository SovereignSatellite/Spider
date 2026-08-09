use core::fmt::{Debug, Formatter, Result as FormatResult};
use core::ops::ControlFlow;

use arbitrary::{Arbitrary, Result, Unstructured};

const OPERATORS: [char; 6] = ['>', '<', '+', '-', ',', '.'];

const MAXIMUM_BLOCK_ITEMS: u32 = 256;
const MAXIMUM_SOURCE_ITEMS: u32 = 4096;

pub struct SupportedSource {
	source: String,
}

impl Debug for SupportedSource {
	fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
		f.debug_struct("SupportedSource").finish_non_exhaustive()
	}
}

impl<'data> Arbitrary<'data> for SupportedSource {
	fn arbitrary(u: &mut Unstructured<'data>) -> Result<Self> {
		let mut budget = MAXIMUM_SOURCE_ITEMS;
		let mut source = String::new();

		write_block(u, &mut budget, &mut source)?;

		Ok(Self { source })
	}
}

impl SupportedSource {
	#[must_use]
	pub fn into_string(self) -> String {
		self.source
	}
}

fn write_block(
	unstructured: &mut Unstructured<'_>,
	budget: &mut u32,
	source: &mut String,
) -> Result<()> {
	unstructured.arbitrary_loop(None, Some(MAXIMUM_BLOCK_ITEMS), |data| {
		if *budget == 0 {
			return Ok(ControlFlow::Break(()));
		}

		*budget -= 1;

		let variant = data.int_in_range(0..=OPERATORS.len())?;

		if let Some(&operator) = OPERATORS.get(variant) {
			source.push(operator);
		} else {
			source.push('[');
			write_block(data, budget, source)?;
			source.push(']');
		}

		Ok(ControlFlow::Continue(()))
	})
}
