#![expect(
	clippy::absolute_paths,
	clippy::cognitive_complexity,
	clippy::collapsible_if,
	clippy::collapsible_match,
	clippy::equatable_if_let,
	clippy::excessive_nesting,
	clippy::match_ref_pats,
	clippy::match_wildcard_for_single_variants,
	clippy::needless_return,
	clippy::too_many_lines,
	clippy::trivially_copy_pass_by_ref,
	clippy::wildcard_enum_match_arm,
	dead_code,
	non_snake_case,
	unreachable_patterns,
	unused_qualifications,
	unused_variables,
	reason = "generated ISLE code does not conform to workspace lint rules"
)]

use ir_graph::{
	Link,
	list::{self, fixed::Fixed},
	operation::{
		LoadType, StoreType,
		integer::{
			BinaryOperator as IntegerBinaryOperator, CompareOperator as IntegerCompareOperator,
			Type as IntegerType,
		},
		number::{Type as NumberType, UnaryOperator as NumberUnaryOperator},
	},
};

use super::luau::{
	Bit32BinaryOperator, Bit32UnaryOperator, LuauArithmeticOperator, LuauBinaryOperator,
	LuauCompareOperator, LuauUnaryOperator,
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
