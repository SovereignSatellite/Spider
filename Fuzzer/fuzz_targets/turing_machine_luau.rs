//! Generates valid Turing Machine sources and compiles them to unoptimized Luau.

#![expect(
	unused_crate_dependencies,
	reason = "this target only exercises Luau output"
)]
#![no_main]

extern crate alloc;

#[path = "../support/turing_machine/generation.rs"]
mod generation;
#[path = "../support/turing_machine/lifting.rs"]
mod lifting;
#[path = "../support/luau.rs"]
mod luau;

use libfuzzer_sys::fuzz_target;

use ir_passes::catalog::Optimizations;

use self::generation::SupportedSource;

fuzz_target!(|source: SupportedSource| {
	let source = source.into_string();
	let root = lifting::lift(source, Optimizations::none());

	luau::compile(&root);
});
