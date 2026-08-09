//! Generates valid WebAssembly modules and compiles them to unoptimized `LuaJIT`.

#![expect(
	unused_crate_dependencies,
	reason = "this target only exercises LuaJIT output"
)]
#![no_main]

extern crate alloc;

#[path = "../support/web_assembly/generation.rs"]
mod generation;
#[path = "../support/web_assembly/lifting.rs"]
mod lifting;
#[path = "../support/luajit.rs"]
mod luajit;

use libfuzzer_sys::fuzz_target;

use ir_passes::catalog::Optimizations;

use self::generation::SupportedModule;

fuzz_target!(|module: SupportedModule| {
	let bytes = module.into_bytes();
	let root = lifting::lift(bytes, Optimizations::none());

	luajit::compile(&root);
});
