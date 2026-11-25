//! Generates valid Turing Machine sources and compiles them to optimized `LuaJIT`.

#![no_main]

extern crate alloc;

#[path = "../support/turing_machine/generation.rs"]
mod generation;
#[path = "../support/turing_machine/lifting.rs"]
mod lifting;
#[path = "../support/luajit.rs"]
mod luajit;

use libfuzzer_sys::fuzz_target;

use ir_passes::catalog::Optimizations;

use self::generation::SupportedSource;

fuzz_target!(|source: SupportedSource| {
	let source = source.into_string();
	let root = lifting::lift(source, Optimizations::all());

	luajit::compile(root);
});
