//! Generates valid WebAssembly modules and compiles them to unoptimized `LuaJIT`.

#![expect(
	unused_crate_dependencies,
	reason = "this target only exercises LuaJIT output"
)]
#![no_main]

extern crate alloc;

#[path = "../support/generation.rs"]
mod generation;
#[path = "../support/luajit.rs"]
mod luajit;
#[path = "../support/optimization.rs"]
mod optimization;

use libfuzzer_sys::fuzz_target;

use self::generation::SupportedModule;

fuzz_target!(|module: SupportedModule| {
	let bytes = module.to_bytes();

	luajit::compile(&bytes, false);
});
