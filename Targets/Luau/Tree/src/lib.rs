//! The Luau tree intermediate representation.

#![expect(
	clippy::multiple_inherent_impl,
	reason = "visitor accept methods are in a separate file from the type definitions"
)]

extern crate alloc;

pub mod expression;
pub mod statement;
pub mod visitor;

const STACK_RED_ZONE: usize = 64 * 1024;
const STACK_SEGMENT: usize = 1024 * 1024;
