//! Generates valid WebAssembly modules and lifts them into Spider IR.

#![expect(
	unused_crate_dependencies,
	reason = "this target only exercises lifting"
)]
#![no_main]

#[path = "../support/web_assembly/generation.rs"]
mod generation;
#[path = "../support/web_assembly/lifting.rs"]
mod lifting;

use libfuzzer_sys::fuzz_target;

use self::generation::SupportedModule;

fuzz_target!(|module: SupportedModule| {
	let bytes = module.to_bytes();

	lifting::lift(&bytes);
});
