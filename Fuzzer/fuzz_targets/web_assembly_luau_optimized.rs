//! Generates valid WebAssembly modules and compiles them to optimized Luau.

#![expect(
	unused_crate_dependencies,
	reason = "this target only exercises Luau output"
)]
#![no_main]

extern crate alloc;

#[path = "../support/generation.rs"]
mod generation;
#[path = "../support/luau.rs"]
mod luau;
#[path = "../support/optimization.rs"]
mod optimization;

use libfuzzer_sys::fuzz_target;

use self::generation::SupportedModule;

fuzz_target!(|module: SupportedModule| {
	let bytes = module.to_bytes();

	luau::compile(&bytes, true);
});
