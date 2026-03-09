//! The Luau tree intermediate representation.

#![no_std]
#![expect(
	clippy::multiple_inherent_impl,
	reason = "visitor accept methods are in a separate file from the type definitions"
)]

extern crate alloc;

pub mod expression;
pub mod statement;
pub mod visitor;

use alloc::vec::Vec;

use self::{
	expression::Name,
	statement::{Export, Sequence},
};

/// The root tree node for a Luau module.
pub struct LuauTree {
	/// The environment variable name.
	pub environment: Name,
	/// The local variable names.
	pub locals: Vec<Name>,
	/// The stack size.
	pub stack: u16,

	/// The main code sequence.
	pub code: Sequence,
	/// The export declarations.
	pub exports: Vec<Export>,
}
