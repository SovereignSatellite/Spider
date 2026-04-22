//! The `LuaJIT` tree intermediate representation.

#![no_std]
#![expect(
	clippy::multiple_inherent_impl,
	reason = "visitor accept methods are in a separate file from the type definitions"
)]

extern crate alloc;

use self::statement::Sequence;

pub mod expression;
pub mod statement;
pub mod visitor;

/// The root tree node for a `LuaJIT` module.
pub struct LuaJITTree {
	/// The stack size.
	pub stack: u16,

	/// The main code sequence.
	pub code: Sequence,
}
