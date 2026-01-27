#![no_std]
#![expect(clippy::missing_panics_doc)]

extern crate alloc;

pub mod expression;
pub mod statement;
pub mod visitor;

use alloc::vec::Vec;

use self::{
	expression::Name,
	statement::{Export, Sequence},
};

pub struct LuaJITTree {
	pub environment: Name,
	pub locals: Vec<Name>,
	pub stack: u16,

	pub code: Sequence,
	pub exports: Vec<Export>,
}
