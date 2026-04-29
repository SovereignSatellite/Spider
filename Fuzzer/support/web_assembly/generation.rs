use core::fmt::{Debug, Formatter, Result as FormatResult};

use arbitrary::{Arbitrary, Result, Unstructured};
use wasm_smith::{Config, Module};

pub struct SupportedModule {
	module: Module,
}

impl Debug for SupportedModule {
	fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
		f.debug_struct("SupportedModule").finish_non_exhaustive()
	}
}

impl<'data> Arbitrary<'data> for SupportedModule {
	fn arbitrary(u: &mut Unstructured<'data>) -> Result<Self> {
		let config = create_config(u)?;
		let module = Module::new(config, u)?;

		Ok(Self { module })
	}
}

fn create_config(data: &mut Unstructured<'_>) -> Result<Config> {
	let mut config = Config::arbitrary(data)?;

	disable_unsupported_proposals(&mut config);

	Ok(config)
}

const fn disable_unsupported_proposals(config: &mut Config) {
	config.allow_invalid_funcs = false;
	config.exceptions_enabled = false;
	config.gc_enabled = false;
	config.custom_descriptors_enabled = false;
	config.memory64_enabled = false;
	config.multi_value_enabled = false;
	config.relaxed_simd_enabled = false;
	config.shared_everything_threads_enabled = false;
	config.simd_enabled = false;
	config.tail_call_enabled = false;
	config.threads_enabled = false;
	config.wide_arithmetic_enabled = false;
}

impl SupportedModule {
	#[must_use]
	pub fn into_bytes(self) -> Vec<u8> {
		self.module.to_bytes()
	}
}
