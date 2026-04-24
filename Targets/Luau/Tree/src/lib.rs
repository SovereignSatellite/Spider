//! The Luau tree intermediate representation.

#![no_std]
#![expect(
	clippy::multiple_inherent_impl,
	reason = "visitor accept methods are in a separate file from the type definitions"
)]

extern crate alloc;

use alloc::vec::Vec;

use self::{expression::Name, statement::Sequence};

pub mod expression;
pub mod statement;
pub mod visitor;

/// The root tree node for a Luau module.
pub struct LuauTree {
	/// The declared fast local names.
	pub locals: Vec<Name>,

	/// The stack size.
	pub stack: u16,

	/// The main code sequence.
	pub code: Sequence,
}
