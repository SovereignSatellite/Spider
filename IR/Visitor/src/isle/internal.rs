#![expect(
	clippy::match_ref_pats,
	clippy::pedantic,
	dead_code,
	non_snake_case,
	unused_imports,
	unused_variables
)]

use ir_graph::{
	Link,
	list::{self, fixed::Fixed},
	simple::{IntegerBinaryOperator, IntegerType},
};

include!(concat!(env!("OUT_DIR"), "/isle.rs"));

impl Links {
	pub fn as_fixed(&self) -> Fixed<Link, 2> {
		match *self {
			Self::N1 { field_1 } => list::fixed![field_1],
			Self::N2 { field_1, field_2 } => list::fixed![field_1, field_2],
		}
	}
}
