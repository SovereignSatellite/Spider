#![expect(
	clippy::absolute_paths,
	clippy::cognitive_complexity,
	clippy::collapsible_if,
	clippy::collapsible_match,
	clippy::doc_markdown,
	clippy::equatable_if_let,
	clippy::excessive_nesting,
	clippy::match_ref_pats,
	clippy::needless_return,
	clippy::similar_names,
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
		ExtendType, LoadType, StoreType,
		integer::{
			BinaryOperator as IntegerBinaryOperator, CompareOperator as IntegerCompareOperator,
			Type as IntegerType, UnaryOperator as IntegerUnaryOperator,
		},
		number::{
			CompareOperator as NumberCompareOperator, Type as NumberType,
			UnaryOperator as NumberUnaryOperator,
		},
	},
};

use super::luau::{
	Bit32BinaryOperator, Bit32UnaryOperator, LuauArithmeticOperator, LuauBinaryOperator,
	LuauCompareOperator, LuauUnaryOperator,
};

include!(concat!(env!("OUT_DIR"), "/isle.rs"));

impl LinkPair {
	pub fn as_fixed(&self) -> Fixed<Link, 2> {
		let Self::Pair { first, second } = *self;

		list::fixed![first, second]
	}
}
