//! Generates valid Turing Machine sources and compiles them to optimized Luau.

#![expect(
	unused_crate_dependencies,
	reason = "this target only exercises Luau output"
)]
#![no_main]

extern crate alloc;

#[path = "../support/turing_machine/generation.rs"]
mod generation;
#[path = "../support/luau.rs"]
mod luau;
#[path = "../support/turing_machine/optimization.rs"]
mod optimization;

use libfuzzer_sys::fuzz_target;

use self::generation::SupportedSource;

fuzz_target!(|source: SupportedSource| {
	let bytes = source.to_bytes();

	luau::compile(&bytes, true);
});
