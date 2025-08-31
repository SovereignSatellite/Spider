#![no_std]

extern crate alloc;

pub mod expression;
pub mod statement;
pub mod visitor;

use alloc::vec::Vec;

use self::{
	expression::Name,
	statement::{Export, Sequence},
};

pub struct LuauTree {
	pub environment: Name,
	pub code: Sequence,
	pub exports: Vec<Export>,
}
