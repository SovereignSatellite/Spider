//! Generates valid Turing Machine sources and lifts them into Spider IR.

#![expect(
	unused_crate_dependencies,
	reason = "this target only exercises lifting"
)]
#![no_main]

#[path = "../support/turing_machine/generation.rs"]
mod generation;
#[path = "../support/turing_machine/lifting.rs"]
mod lifting;

use libfuzzer_sys::fuzz_target;

use self::generation::SupportedSource;

fuzz_target!(|source: SupportedSource| {
	let bytes = source.to_bytes();

	lifting::lift(&bytes);
});
