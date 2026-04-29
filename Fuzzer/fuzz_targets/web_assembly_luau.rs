//! Generates valid WebAssembly modules and compiles them to unoptimized Luau.

#![expect(
	unused_crate_dependencies,
	reason = "this target only exercises Luau output"
)]
#![no_main]

extern crate alloc;

#[path = "../support/web_assembly/generation.rs"]
mod generation;
#[path = "../support/web_assembly/lifting.rs"]
mod lifting;
#[path = "../support/luau.rs"]
mod luau;

use libfuzzer_sys::fuzz_target;

use self::generation::SupportedModule;

fuzz_target!(|module: SupportedModule| {
	let bytes = module.into_bytes();
	let root = lifting::lift(&bytes, false);

	luau::compile(&root);
});
