#![expect(
	clippy::collapsible_if,
	clippy::equatable_if_let,
	clippy::match_ref_pats,
	clippy::needless_return,
	clippy::trivially_copy_pass_by_ref,
	clippy::wildcard_enum_match_arm,
	dead_code,
	non_snake_case,
	unused_variables,
	reason = "generated ISLE code does not conform to workspace lint rules"
)]

use ir_graph::{
	Link,
	list::{self, fixed::Fixed},
	simple::{IntegerBinaryOperator, IntegerType, LoadType, StoreType},
};

include!(concat!(env!("OUT_DIR"), "/isle.rs"));

impl Links {
	/// Converts these links into a fixed-size array.
	pub fn as_fixed(&self) -> Fixed<Link, 2> {
		match *self {
			Self::N1 { field_1 } => list::fixed![field_1],
			Self::N2 { field_1, field_2 } => list::fixed![field_1, field_2],
		}
	}
}
