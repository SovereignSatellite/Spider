//! Optimization identifiers, selection policy, and execution-stage metadata.

pub use self::taxonomy::{OptimizationDomain, OptimizationIntent, OptimizationLevel};

mod selectors;
mod taxonomy;

/// Describe one independently selectable transformation.
#[derive(Clone, Copy, Debug)]
pub struct OptimizationDescriptor {
	/// Name the long CLI flag without leading hyphens.
	pub flag: &'static str,
	/// Identify the lowest standard optimization level containing this transformation.
	pub level: OptimizationLevel,
	/// Identify the intent family containing this transformation.
	pub intent: OptimizationIntent,
	/// Provide the searchable CLI help sentence.
	pub help: &'static str,
}

#[derive(Clone, Copy)]
enum OptimizationStage {
	ControlFolder,
	Isle,
	MatchOutputReducer,
	TargetLowering,
}

/// Summarize the pass stages selected for one optimizer run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[expect(
	clippy::struct_excessive_bools,
	reason = "each flag identifies an independently selected pass stage; `any_optimization` also covers direct graph passes"
)]
pub struct OptimizationStages {
	/// Report whether any transformation is selected.
	pub any_optimization: bool,
	/// Report whether control folding is selected.
	pub control_folder: bool,
	/// Report whether generic ISLE rewriting is selected.
	pub isle_rewrites: bool,
	/// Report whether Match-output reduction is selected.
	pub match_output_reductions: bool,
	/// Report whether target lowering is selected.
	pub target_lowering: bool,
}

macro_rules! define_optimizations {
	(
		$(
			(
				[$($stage:ident),*],
				$field:ident,
				$variant:ident,
				$flag:literal,
				$level:ident,
				$intent:ident,
				$help:literal
			)
		),+
		$(,)?
	) => {
		/// Identify one independently selectable semantic transformation.
		#[derive(Clone, Copy, Debug, PartialEq, Eq)]
		pub enum Optimization {
			$(#[doc = $help] $variant),+
		}

		impl Optimization {
			/// List every transformation in declaration order.
			pub const ALL: &'static [Self] = &[$(Self::$variant),+];

			/// Return the selection and CLI metadata for this transformation.
			#[must_use]
			pub const fn descriptor(self) -> OptimizationDescriptor {
				match self {
					$(Self::$variant => OptimizationDescriptor {
						flag: $flag,
						level: OptimizationLevel::$level,
						intent: OptimizationIntent::$intent,
						help: $help,
					}),+
				}
			}

			const fn uses_stage(self, stage: OptimizationStage) -> bool {
				match self {
					$(Self::$variant => false $(|| matches!(
						stage,
						OptimizationStage::$stage
					))*),+
				}
			}

		}

		/// Select the transformations available to an optimizer run.
		#[derive(Debug, PartialEq, Eq)]
		#[expect(
			missing_copy_implementations,
			reason = "the resolved policy is passed by shared reference to avoid implicit large copies"
		)]
		pub struct Optimizations {
			$(#[doc = $help] pub $field: bool),+
		}

		impl Optimizations {
			/// Disable every transformation.
			#[must_use]
			pub const fn none() -> Self {
				Self {
					$($field: false),+
				}
			}

			/// Enable every transformation.
			#[must_use]
			pub const fn all() -> Self {
				Self {
					$($field: true),+
				}
			}

			/// Enable one transformation.
			pub const fn enable(&mut self, optimization: Optimization) {
				match optimization {
					$(Optimization::$variant => self.$field = true),+
				}
			}

			/// Return whether one transformation is enabled.
			#[must_use]
			pub const fn is_enabled(&self, optimization: Optimization) -> bool {
				match optimization {
					$(Optimization::$variant => self.$field),+
				}
			}

			/// Summarize the selected pass stages in one catalog traversal.
			#[must_use]
			pub const fn stages(&self) -> OptimizationStages {
				let mut stages = OptimizationStages {
					any_optimization: false,
					control_folder: false,
					isle_rewrites: false,
					match_output_reductions: false,
					target_lowering: false,
				};
				let mut index = 0;

				while index < Optimization::ALL.len() {
					let optimization = Optimization::ALL[index];

					if self.is_enabled(optimization) {
						stages.any_optimization = true;
						stages.control_folder |=
							optimization.uses_stage(OptimizationStage::ControlFolder);
						stages.isle_rewrites |=
							optimization.uses_stage(OptimizationStage::Isle);
						stages.match_output_reductions |=
							optimization.uses_stage(OptimizationStage::MatchOutputReducer);
						stages.target_lowering |=
							optimization.uses_stage(OptimizationStage::TargetLowering);
					}

					index += 1;
				}

				stages
			}
		}
	};
}

define_optimizations!(
	(
		[],
		eliminate_common_nodes,
		CommonNodeElimination,
		"eliminate-common-nodes",
		Two,
		CommonNodeElimination,
		"Eliminate congruent pure nodes within each region."
	),
	(
		[],
		move_invariant_ports,
		InvariantPortMotion,
		"move-invariant-ports",
		Two,
		ControlMotion,
		"Move values unchanged by conditional or loop regions into the surrounding region."
	),
	(
		[ControlFolder],
		fold_constant_match,
		ConstantMatchFolding,
		"fold-constant-match",
		One,
		ControlFolding,
		"Inline the selected branch of a statically known conditional region."
	),
	(
		[ControlFolder],
		fold_exiting_repeat,
		ExitingRepeatFolding,
		"fold-exiting-repeat",
		One,
		ControlFolding,
		"Inline a loop region that exits on its first continuation check."
	),
	(
		[Isle],
		complement_boolean_not,
		ComplementBooleanNot,
		"complement-boolean-not",
		Two,
		MatchTruthTables,
		"Cancel two Boolean complements."
	),
	(
		[Isle],
		complement_integer_comparison,
		ComplementIntegerComparison,
		"complement-integer-comparison",
		Two,
		IntegerComparisons,
		"Build the direct complement of an integer comparison."
	),
	(
		[Isle],
		complement_number_comparison,
		ComplementNumberComparison,
		"complement-floating-point-comparison",
		Two,
		FloatingPointComparisons,
		"Build the direct complement of a floating-point comparison."
	),
	(
		[Isle],
		reduce_i32_boolean_table_constant,
		ReduceI32BooleanTableConstant,
		"reduce-i32-boolean-table-constant",
		Two,
		MatchTruthTables,
		"Replace a constant i32 Boolean table with its result."
	),
	(
		[Isle],
		reduce_i32_boolean_table_identity,
		ReduceI32BooleanTableIdentity,
		"reduce-i32-boolean-table-identity",
		Two,
		MatchTruthTables,
		"Replace the i32 table [0, 1] with its predicate."
	),
	(
		[Isle],
		reduce_i32_boolean_table_complement,
		ReduceI32BooleanTableComplement,
		"reduce-i32-boolean-table-complement",
		Two,
		MatchTruthTables,
		"Replace the i32 table [1, 0] with a complemented predicate."
	),
	(
		[Isle],
		reduce_i64_boolean_table_constant,
		ReduceI64BooleanTableConstant,
		"reduce-i64-boolean-table-constant",
		Two,
		MatchTruthTables,
		"Replace a constant i64 Boolean table with its result."
	),
	(
		[Isle],
		reduce_i64_boolean_table_identity,
		ReduceI64BooleanTableIdentity,
		"reduce-i64-boolean-table-identity",
		Two,
		MatchTruthTables,
		"Replace the i64 table [0, 1] with its widened predicate."
	),
	(
		[Isle],
		reduce_i64_boolean_table_complement,
		ReduceI64BooleanTableComplement,
		"reduce-i64-boolean-table-complement",
		Two,
		MatchTruthTables,
		"Replace the i64 table [1, 0] with a widened complemented predicate."
	),
	(
		[Isle, MatchOutputReducer],
		reduce_i32_truth_table_with_xor,
		ReduceI32TruthTableWithXor,
		"reduce-i32-truth-table-with-xor",
		Three,
		MatchTruthTables,
		"Reduce an i32 two-row truth table to its predicate XOR a constant."
	),
	(
		[Isle, MatchOutputReducer],
		reduce_i32_truth_table_with_increment,
		ReduceI32TruthTableWithIncrement,
		"reduce-i32-truth-table-with-increment",
		Three,
		MatchTruthTables,
		"Reduce an i32 two-row truth table to predicate plus a constant."
	),
	(
		[Isle, MatchOutputReducer],
		reduce_i32_truth_table_with_decrement,
		ReduceI32TruthTableWithDecrement,
		"reduce-i32-truth-table-with-decrement",
		Three,
		MatchTruthTables,
		"Reduce an i32 two-row truth table to a constant minus its predicate."
	),
	(
		[Isle, MatchOutputReducer],
		reduce_i32_truth_table_with_mask,
		ReduceI32TruthTableWithMask,
		"reduce-i32-truth-table-with-mask",
		Three,
		MatchTruthTables,
		"Reduce any i32 two-row truth table to a mask expression."
	),
	(
		[Isle, MatchOutputReducer],
		reduce_i64_truth_table_with_xor,
		ReduceI64TruthTableWithXor,
		"reduce-i64-truth-table-with-xor",
		Three,
		MatchTruthTables,
		"Reduce an i64 two-row truth table to its widened predicate XOR a constant."
	),
	(
		[Isle, MatchOutputReducer],
		reduce_i64_truth_table_with_increment,
		ReduceI64TruthTableWithIncrement,
		"reduce-i64-truth-table-with-increment",
		Three,
		MatchTruthTables,
		"Reduce an i64 two-row truth table to its widened predicate plus a constant."
	),
	(
		[Isle, MatchOutputReducer],
		reduce_i64_truth_table_with_decrement,
		ReduceI64TruthTableWithDecrement,
		"reduce-i64-truth-table-with-decrement",
		Three,
		MatchTruthTables,
		"Reduce an i64 two-row truth table to a constant minus its widened predicate."
	),
	(
		[Isle, MatchOutputReducer],
		reduce_i64_truth_table_with_mask,
		ReduceI64TruthTableWithMask,
		"reduce-i64-truth-table-with-mask",
		Three,
		MatchTruthTables,
		"Reduce any i64 two-row truth table to a mask expression."
	),
	(
		[Isle],
		forward_aggregate_extraction,
		ForwardAggregateExtraction,
		"forward-aggregate-extraction",
		One,
		AggregateForwarding,
		"Forward extraction from a newly constructed aggregate."
	),
	(
		[Isle],
		forward_i32_load_from_store,
		ForwardI32LoadFromStore,
		"forward-i32-load-from-store",
		Two,
		MemoryForwarding,
		"Forward an i32 load from the adjacent store to the same offset."
	),
	(
		[Isle],
		forward_mutable_get,
		ForwardMutableGet,
		"forward-mutable-get",
		Two,
		MutableForwarding,
		"Forward a mutable get from the adjacent defining operation."
	),
	(
		[Isle],
		fold_null_reference_check,
		FoldNullReferenceCheck,
		"fold-null-reference-check",
		One,
		ReferenceFolding,
		"Fold a null check of the null reference."
	),
	(
		[Isle],
		forward_table_get_from_set,
		ForwardTableGetFromSet,
		"forward-table-get-from-set",
		Two,
		TableForwarding,
		"Forward a table get from the adjacent set to the same index."
	),
	(
		[Isle],
		reduce_double_float_negation,
		ReduceDoubleFloatNegation,
		"reduce-double-float-negation",
		One,
		FloatingPointIdentities,
		"Cancel two floating-point negations."
	),
	(
		[Isle],
		reduce_float_absolute,
		ReduceFloatAbsolute,
		"reduce-float-absolute",
		One,
		FloatingPointIdentities,
		"Remove redundant operations beneath floating-point absolute value."
	),
	(
		[Isle],
		reduce_boolean_sign_extension,
		ReduceBooleanSignExtension,
		"reduce-boolean-sign-extension",
		One,
		IntegerConversions,
		"Remove sign extension of an exact Boolean value."
	),
	(
		[Isle],
		reduce_narrow_widen,
		ReduceNarrowWiden,
		"reduce-narrow-widen",
		One,
		IntegerConversions,
		"Cancel integer narrowing after widening."
	),
	(
		[Isle],
		reduce_idempotent_sign_extension,
		ReduceIdempotentSignExtension,
		"reduce-idempotent-sign-extension",
		One,
		IntegerConversions,
		"Remove a redundant nested sign extension."
	),
	(
		[Isle],
		reduce_integer_transmute_round_trip,
		ReduceIntegerTransmuteRoundTrip,
		"reduce-integer-transmute-round-trip",
		One,
		IntegerConversions,
		"Cancel an integer bit-transmute round trip."
	),
	(
		[Isle],
		reduce_number_transmute_round_trip,
		ReduceNumberTransmuteRoundTrip,
		"reduce-floating-point-transmute-round-trip",
		One,
		FloatingPointConversions,
		"Cancel a floating-point bit-transmute round trip."
	),
	(
		[Isle],
		reduce_i32_add_zero,
		ReduceI32AddZero,
		"reduce-i32-add-zero",
		One,
		IntegerIdentities,
		"Remove addition of zero from an i32 value."
	),
	(
		[Isle],
		reduce_i32_subtract_identity,
		ReduceI32SubtractIdentity,
		"reduce-i32-subtract-identity",
		One,
		IntegerIdentities,
		"Reduce identity and self-subtraction forms for i32 values."
	),
	(
		[Isle],
		reduce_i32_multiply_identity,
		ReduceI32MultiplyIdentity,
		"reduce-i32-multiply-identity",
		One,
		IntegerIdentities,
		"Reduce multiplication by zero or one for i32 values."
	),
	(
		[Isle],
		reduce_i32_and_identity,
		ReduceI32AndIdentity,
		"reduce-i32-and-identity",
		One,
		IntegerIdentities,
		"Reduce i32 bitwise AND identity forms."
	),
	(
		[Isle],
		reduce_i32_or_identity,
		ReduceI32OrIdentity,
		"reduce-i32-or-identity",
		One,
		IntegerIdentities,
		"Reduce i32 bitwise OR identity forms."
	),
	(
		[Isle],
		reduce_i32_exclusive_or_identity,
		ReduceI32ExclusiveOrIdentity,
		"reduce-i32-exclusive-or-identity",
		One,
		IntegerIdentities,
		"Reduce i32 bitwise XOR identity forms."
	),
	(
		[Isle],
		reduce_i32_shift_identity,
		ReduceI32ShiftIdentity,
		"reduce-i32-shift-identity",
		One,
		IntegerIdentities,
		"Remove i32 shifts by zero."
	),
	(
		[Isle],
		reduce_i32_rotate_identity,
		ReduceI32RotateIdentity,
		"reduce-i32-rotate-identity",
		One,
		IntegerIdentities,
		"Remove i32 rotations by zero."
	),
	(
		[Isle],
		reduce_i32_divide_one,
		ReduceI32DivideOne,
		"reduce-i32-divide-one",
		One,
		IntegerIdentities,
		"Remove i32 division by one."
	),
	(
		[Isle],
		reduce_i32_remainder_one,
		ReduceI32RemainderOne,
		"reduce-i32-remainder-one",
		One,
		IntegerIdentities,
		"Replace i32 remainder by one with zero."
	),
	(
		[Isle],
		reduce_i32_double_negation,
		ReduceI32DoubleNegation,
		"reduce-i32-double-negation",
		One,
		IntegerIdentities,
		"Cancel two i32 arithmetic negations."
	),
	(
		[Isle],
		fold_i32_count_ones,
		FoldI32CountOnes,
		"fold-i32-count-ones",
		One,
		IntegerConstantFolding,
		"Fold i32 population count."
	),
	(
		[Isle],
		fold_i32_leading_zeros,
		FoldI32LeadingZeros,
		"fold-i32-leading-zeros",
		One,
		IntegerConstantFolding,
		"Fold i32 leading-zero count."
	),
	(
		[Isle],
		fold_i32_trailing_zeros,
		FoldI32TrailingZeros,
		"fold-i32-trailing-zeros",
		One,
		IntegerConstantFolding,
		"Fold i32 trailing-zero count."
	),
	(
		[Isle],
		fold_i32_add,
		FoldI32Add,
		"fold-i32-add",
		One,
		IntegerConstantFolding,
		"Fold i32 addition."
	),
	(
		[Isle],
		fold_i32_subtract,
		FoldI32Subtract,
		"fold-i32-subtract",
		One,
		IntegerConstantFolding,
		"Fold i32 subtraction."
	),
	(
		[Isle],
		fold_i32_multiply,
		FoldI32Multiply,
		"fold-i32-multiply",
		One,
		IntegerConstantFolding,
		"Fold i32 multiplication."
	),
	(
		[Isle],
		fold_i32_signed_divide,
		FoldI32SignedDivide,
		"fold-i32-signed-divide",
		One,
		IntegerConstantFolding,
		"Fold nontrapping signed i32 division."
	),
	(
		[Isle],
		fold_i32_unsigned_divide,
		FoldI32UnsignedDivide,
		"fold-i32-unsigned-divide",
		One,
		IntegerConstantFolding,
		"Fold nontrapping unsigned i32 division."
	),
	(
		[Isle],
		fold_i32_signed_remainder,
		FoldI32SignedRemainder,
		"fold-i32-signed-remainder",
		One,
		IntegerConstantFolding,
		"Fold nontrapping signed i32 remainder."
	),
	(
		[Isle],
		fold_i32_unsigned_remainder,
		FoldI32UnsignedRemainder,
		"fold-i32-unsigned-remainder",
		One,
		IntegerConstantFolding,
		"Fold nontrapping unsigned i32 remainder."
	),
	(
		[Isle],
		fold_i32_and,
		FoldI32And,
		"fold-i32-and",
		One,
		IntegerConstantFolding,
		"Fold i32 bitwise AND."
	),
	(
		[Isle],
		fold_i32_or,
		FoldI32Or,
		"fold-i32-or",
		One,
		IntegerConstantFolding,
		"Fold i32 bitwise OR."
	),
	(
		[Isle],
		fold_i32_exclusive_or,
		FoldI32ExclusiveOr,
		"fold-i32-exclusive-or",
		One,
		IntegerConstantFolding,
		"Fold i32 bitwise XOR."
	),
	(
		[Isle],
		fold_i32_shift_left,
		FoldI32ShiftLeft,
		"fold-i32-shift-left",
		One,
		IntegerConstantFolding,
		"Fold i32 left shift."
	),
	(
		[Isle],
		fold_i32_signed_shift_right,
		FoldI32SignedShiftRight,
		"fold-i32-signed-shift-right",
		One,
		IntegerConstantFolding,
		"Fold signed i32 right shift."
	),
	(
		[Isle],
		fold_i32_unsigned_shift_right,
		FoldI32UnsignedShiftRight,
		"fold-i32-unsigned-shift-right",
		One,
		IntegerConstantFolding,
		"Fold unsigned i32 right shift."
	),
	(
		[Isle],
		fold_i32_rotate_left,
		FoldI32RotateLeft,
		"fold-i32-rotate-left",
		One,
		IntegerConstantFolding,
		"Fold i32 left rotation."
	),
	(
		[Isle],
		fold_i32_rotate_right,
		FoldI32RotateRight,
		"fold-i32-rotate-right",
		One,
		IntegerConstantFolding,
		"Fold i32 right rotation."
	),
	(
		[Isle],
		reassociate_i32_constants,
		ReassociateI32Constants,
		"reassociate-i32-constants",
		Two,
		IntegerConstantReassociation,
		"Combine adjacent i32 addition and subtraction constants."
	),
	(
		[Isle],
		fold_reflexive_i32_comparison,
		FoldReflexiveI32Comparison,
		"fold-reflexive-i32-comparison",
		One,
		IntegerComparisons,
		"Fold an i32 comparison whose operands are identical."
	),
	(
		[Isle],
		fold_i32_equal,
		FoldI32Equal,
		"fold-i32-equal",
		One,
		IntegerConstantFolding,
		"Fold i32 equality."
	),
	(
		[Isle],
		fold_i32_not_equal,
		FoldI32NotEqual,
		"fold-i32-not-equal",
		One,
		IntegerConstantFolding,
		"Fold i32 inequality."
	),
	(
		[Isle],
		fold_i32_signed_less_than,
		FoldI32SignedLessThan,
		"fold-i32-signed-less-than",
		One,
		IntegerConstantFolding,
		"Fold signed i32 less-than comparison."
	),
	(
		[Isle],
		fold_i32_unsigned_less_than,
		FoldI32UnsignedLessThan,
		"fold-i32-unsigned-less-than",
		One,
		IntegerConstantFolding,
		"Fold unsigned i32 less-than comparison."
	),
	(
		[Isle],
		fold_i32_signed_less_than_equal,
		FoldI32SignedLessThanEqual,
		"fold-i32-signed-less-than-equal",
		One,
		IntegerConstantFolding,
		"Fold signed i32 less-than-or-equal comparison."
	),
	(
		[Isle],
		fold_i32_unsigned_less_than_equal,
		FoldI32UnsignedLessThanEqual,
		"fold-i32-unsigned-less-than-equal",
		One,
		IntegerConstantFolding,
		"Fold unsigned i32 less-than-or-equal comparison."
	),
	(
		[Isle],
		reduce_i64_add_zero,
		ReduceI64AddZero,
		"reduce-i64-add-zero",
		One,
		IntegerIdentities,
		"Remove addition of zero from an i64 value."
	),
	(
		[Isle],
		reduce_i64_subtract_identity,
		ReduceI64SubtractIdentity,
		"reduce-i64-subtract-identity",
		One,
		IntegerIdentities,
		"Reduce identity and self-subtraction forms for i64 values."
	),
	(
		[Isle],
		reduce_i64_multiply_identity,
		ReduceI64MultiplyIdentity,
		"reduce-i64-multiply-identity",
		One,
		IntegerIdentities,
		"Reduce multiplication by zero or one for i64 values."
	),
	(
		[Isle],
		reduce_i64_and_identity,
		ReduceI64AndIdentity,
		"reduce-i64-and-identity",
		One,
		IntegerIdentities,
		"Reduce i64 bitwise AND identity forms."
	),
	(
		[Isle],
		reduce_i64_or_identity,
		ReduceI64OrIdentity,
		"reduce-i64-or-identity",
		One,
		IntegerIdentities,
		"Reduce i64 bitwise OR identity forms."
	),
	(
		[Isle],
		reduce_i64_exclusive_or_identity,
		ReduceI64ExclusiveOrIdentity,
		"reduce-i64-exclusive-or-identity",
		One,
		IntegerIdentities,
		"Reduce i64 bitwise XOR identity forms."
	),
	(
		[Isle],
		reduce_i64_shift_identity,
		ReduceI64ShiftIdentity,
		"reduce-i64-shift-identity",
		One,
		IntegerIdentities,
		"Remove i64 shifts by zero."
	),
	(
		[Isle],
		reduce_i64_rotate_identity,
		ReduceI64RotateIdentity,
		"reduce-i64-rotate-identity",
		One,
		IntegerIdentities,
		"Remove i64 rotations by zero."
	),
	(
		[Isle],
		reduce_i64_divide_one,
		ReduceI64DivideOne,
		"reduce-i64-divide-one",
		One,
		IntegerIdentities,
		"Remove i64 division by one."
	),
	(
		[Isle],
		reduce_i64_remainder_one,
		ReduceI64RemainderOne,
		"reduce-i64-remainder-one",
		One,
		IntegerIdentities,
		"Replace i64 remainder by one with zero."
	),
	(
		[Isle],
		reduce_i64_double_negation,
		ReduceI64DoubleNegation,
		"reduce-i64-double-negation",
		One,
		IntegerIdentities,
		"Cancel two i64 arithmetic negations."
	),
	(
		[Isle],
		fold_i64_count_ones,
		FoldI64CountOnes,
		"fold-i64-count-ones",
		One,
		IntegerConstantFolding,
		"Fold i64 population count."
	),
	(
		[Isle],
		fold_i64_leading_zeros,
		FoldI64LeadingZeros,
		"fold-i64-leading-zeros",
		One,
		IntegerConstantFolding,
		"Fold i64 leading-zero count."
	),
	(
		[Isle],
		fold_i64_trailing_zeros,
		FoldI64TrailingZeros,
		"fold-i64-trailing-zeros",
		One,
		IntegerConstantFolding,
		"Fold i64 trailing-zero count."
	),
	(
		[Isle],
		fold_i64_add,
		FoldI64Add,
		"fold-i64-add",
		One,
		IntegerConstantFolding,
		"Fold i64 addition."
	),
	(
		[Isle],
		fold_i64_subtract,
		FoldI64Subtract,
		"fold-i64-subtract",
		One,
		IntegerConstantFolding,
		"Fold i64 subtraction."
	),
	(
		[Isle],
		fold_i64_multiply,
		FoldI64Multiply,
		"fold-i64-multiply",
		One,
		IntegerConstantFolding,
		"Fold i64 multiplication."
	),
	(
		[Isle],
		fold_i64_signed_divide,
		FoldI64SignedDivide,
		"fold-i64-signed-divide",
		One,
		IntegerConstantFolding,
		"Fold nontrapping signed i64 division."
	),
	(
		[Isle],
		fold_i64_unsigned_divide,
		FoldI64UnsignedDivide,
		"fold-i64-unsigned-divide",
		One,
		IntegerConstantFolding,
		"Fold nontrapping unsigned i64 division."
	),
	(
		[Isle],
		fold_i64_signed_remainder,
		FoldI64SignedRemainder,
		"fold-i64-signed-remainder",
		One,
		IntegerConstantFolding,
		"Fold nontrapping signed i64 remainder."
	),
	(
		[Isle],
		fold_i64_unsigned_remainder,
		FoldI64UnsignedRemainder,
		"fold-i64-unsigned-remainder",
		One,
		IntegerConstantFolding,
		"Fold nontrapping unsigned i64 remainder."
	),
	(
		[Isle],
		fold_i64_and,
		FoldI64And,
		"fold-i64-and",
		One,
		IntegerConstantFolding,
		"Fold i64 bitwise AND."
	),
	(
		[Isle],
		fold_i64_or,
		FoldI64Or,
		"fold-i64-or",
		One,
		IntegerConstantFolding,
		"Fold i64 bitwise OR."
	),
	(
		[Isle],
		fold_i64_exclusive_or,
		FoldI64ExclusiveOr,
		"fold-i64-exclusive-or",
		One,
		IntegerConstantFolding,
		"Fold i64 bitwise XOR."
	),
	(
		[Isle],
		fold_i64_shift_left,
		FoldI64ShiftLeft,
		"fold-i64-shift-left",
		One,
		IntegerConstantFolding,
		"Fold i64 left shift."
	),
	(
		[Isle],
		fold_i64_signed_shift_right,
		FoldI64SignedShiftRight,
		"fold-i64-signed-shift-right",
		One,
		IntegerConstantFolding,
		"Fold signed i64 right shift."
	),
	(
		[Isle],
		fold_i64_unsigned_shift_right,
		FoldI64UnsignedShiftRight,
		"fold-i64-unsigned-shift-right",
		One,
		IntegerConstantFolding,
		"Fold unsigned i64 right shift."
	),
	(
		[Isle],
		fold_i64_rotate_left,
		FoldI64RotateLeft,
		"fold-i64-rotate-left",
		One,
		IntegerConstantFolding,
		"Fold i64 left rotation."
	),
	(
		[Isle],
		fold_i64_rotate_right,
		FoldI64RotateRight,
		"fold-i64-rotate-right",
		One,
		IntegerConstantFolding,
		"Fold i64 right rotation."
	),
	(
		[Isle],
		reassociate_i64_constants,
		ReassociateI64Constants,
		"reassociate-i64-constants",
		Two,
		IntegerConstantReassociation,
		"Combine adjacent i64 addition and subtraction constants."
	),
	(
		[Isle],
		fold_reflexive_i64_comparison,
		FoldReflexiveI64Comparison,
		"fold-reflexive-i64-comparison",
		One,
		IntegerComparisons,
		"Fold an i64 comparison whose operands are identical."
	),
	(
		[Isle],
		fold_i64_equal,
		FoldI64Equal,
		"fold-i64-equal",
		One,
		IntegerConstantFolding,
		"Fold i64 equality."
	),
	(
		[Isle],
		fold_i64_not_equal,
		FoldI64NotEqual,
		"fold-i64-not-equal",
		One,
		IntegerConstantFolding,
		"Fold i64 inequality."
	),
	(
		[Isle],
		fold_i64_signed_less_than,
		FoldI64SignedLessThan,
		"fold-i64-signed-less-than",
		One,
		IntegerConstantFolding,
		"Fold signed i64 less-than comparison."
	),
	(
		[Isle],
		fold_i64_unsigned_less_than,
		FoldI64UnsignedLessThan,
		"fold-i64-unsigned-less-than",
		One,
		IntegerConstantFolding,
		"Fold unsigned i64 less-than comparison."
	),
	(
		[Isle],
		fold_i64_signed_less_than_equal,
		FoldI64SignedLessThanEqual,
		"fold-i64-signed-less-than-equal",
		One,
		IntegerConstantFolding,
		"Fold signed i64 less-than-or-equal comparison."
	),
	(
		[Isle],
		fold_i64_unsigned_less_than_equal,
		FoldI64UnsignedLessThanEqual,
		"fold-i64-unsigned-less-than-equal",
		One,
		IntegerConstantFolding,
		"Fold unsigned i64 less-than-or-equal comparison."
	),
	(
		[Isle],
		fold_reflexive_number_ordering,
		FoldReflexiveNumberOrdering,
		"fold-reflexive-floating-point-ordering",
		Two,
		FloatingPointComparisons,
		"Fold ordered floating-point comparisons with identical operands."
	),
	(
		[Isle],
		reduce_luau_double_negation,
		ReduceLuauDoubleNegation,
		"reduce-luau-double-negation",
		Two,
		LuauFloatingPointReduction,
		"Cancel two Luau numeric negations."
	),
	(
		[Isle],
		fold_luau_arithmetic_constants,
		FoldLuauArithmeticConstants,
		"fold-luau-arithmetic-constants",
		Two,
		LuauFloatingPointReduction,
		"Fold finite Luau arithmetic constants."
	),
	(
		[Isle],
		fuse_luau_bit32_constants,
		FuseLuauBit32Constants,
		"fuse-luau-bit32-constants",
		Two,
		LuauIntegerReduction,
		"Fuse adjacent Luau bit32 operations with constants."
	),
	(
		[Isle],
		fold_luau_bit32_constants,
		FoldLuauBit32Constants,
		"fold-luau-bit32-constants",
		Two,
		LuauIntegerReduction,
		"Fold Luau bit32 operations with constant operands."
	),
	(
		[Isle],
		fold_luau_bit32_unary,
		FoldLuauBit32Unary,
		"fold-luau-bit32-unary",
		Two,
		LuauIntegerReduction,
		"Fold Luau bit32 unary operations with constant operands."
	),
	(
		[Isle],
		reduce_luau_bit32_exclusive_or,
		ReduceLuauBit32ExclusiveOr,
		"reduce-luau-bit32-exclusive-or",
		Two,
		LuauIntegerReduction,
		"Reduce reflexive Luau bit32 XOR."
	),
	(
		[Isle],
		reduce_luau_bit32_and,
		ReduceLuauBit32And,
		"reduce-luau-bit32-and",
		Two,
		LuauIntegerReduction,
		"Reduce Luau bit32 AND identity forms."
	),
	(
		[Isle],
		reduce_luau_bit32_or,
		ReduceLuauBit32Or,
		"reduce-luau-bit32-or",
		Two,
		LuauIntegerReduction,
		"Reduce Luau bit32 OR identity forms."
	),
	(
		[Isle],
		reduce_canonical_luau_bit32_and,
		ReduceCanonicalLuauBit32And,
		"reduce-canonical-luau-bit32-and",
		Two,
		LuauIntegerReduction,
		"Remove a redundant all-ones mask from canonical bit32 values."
	),
	(
		[Isle],
		reduce_canonical_luau_bit32_zero,
		ReduceCanonicalLuauBit32Zero,
		"reduce-canonical-luau-bit32-zero",
		Two,
		LuauIntegerReduction,
		"Remove a redundant zero operation from canonical bit32 values."
	),
	(
		[Isle],
		fold_luau_reflexive_comparison,
		FoldLuauReflexiveComparison,
		"fold-luau-reflexive-comparison",
		Two,
		LuauFloatingPointReduction,
		"Fold a strict Luau comparison whose operands are identical."
	),
	(
		[Isle],
		fold_luau_comparison_constants,
		FoldLuauComparisonConstants,
		"fold-luau-comparison-constants",
		Two,
		LuauFloatingPointReduction,
		"Fold a Luau comparison with constant operands."
	),
	(
		[Isle],
		reduce_luau_absolute,
		ReduceLuauAbsolute,
		"reduce-luau-absolute",
		Two,
		LuauFloatingPointReduction,
		"Remove redundant Luau absolute-value operations."
	),
	(
		[Isle],
		reduce_luau_integral_unary,
		ReduceLuauIntegralUnary,
		"reduce-luau-integral-unary",
		Two,
		LuauFloatingPointReduction,
		"Remove redundant nested integral Luau unary operations."
	),
	(
		[Isle],
		reduce_luau_extremum,
		ReduceLuauExtremum,
		"reduce-luau-extremum",
		Two,
		LuauFloatingPointReduction,
		"Reduce a Luau extremum whose operands are identical."
	),
	(
		[Isle],
		fold_luau_unary_constants,
		FoldLuauUnaryConstants,
		"fold-luau-unary-constants",
		Two,
		LuauFloatingPointReduction,
		"Fold finite Luau unary operations with constant operands."
	),
	(
		[Isle],
		fold_luau_binary_constants,
		FoldLuauBinaryConstants,
		"fold-luau-binary-constants",
		Two,
		LuauFloatingPointReduction,
		"Fold Luau binary operations with constant operands."
	),
	(
		[Isle],
		reduce_luau_split_repack,
		ReduceLuauSplitRepack,
		"reduce-luau-split-repack",
		Two,
		LuauI64RepresentationReduction,
		"Cancel splitting after packing an i64 value."
	),
	(
		[Isle],
		fold_luau_i64_split,
		FoldLuauI64Split,
		"fold-luau-i64-split",
		Two,
		LuauI64RepresentationReduction,
		"Split a constant packed i64 value at compile time."
	),
	(
		[Isle],
		push_luau_sign_flip_through_split,
		PushLuauSignFlipThroughSplit,
		"push-luau-sign-flip-through-split",
		Two,
		LuauI64RepresentationReduction,
		"Push a packed sign-bit flip into the high split word."
	),
	(
		[Isle],
		reduce_luau_double_sign_flip,
		ReduceLuauDoubleSignFlip,
		"reduce-luau-double-sign-flip",
		Two,
		LuauI64RepresentationReduction,
		"Cancel two packed sign-bit flips."
	),
	(
		[Isle],
		reduce_luau_repack,
		ReduceLuauRepack,
		"reduce-luau-repack",
		Two,
		LuauI64RepresentationReduction,
		"Cancel packing after splitting an i64 value."
	),
	(
		[Isle],
		fold_luau_i64_pack,
		FoldLuauI64Pack,
		"fold-luau-i64-pack",
		Two,
		LuauI64RepresentationReduction,
		"Pack constant i64 words at compile time."
	),
	(
		[TargetLowering],
		lower_i32_count_ones,
		LowerI32CountOnes,
		"lower-i32-count-ones",
		Two,
		IntegerTargetLowering,
		"Lower i32 population count."
	),
	(
		[TargetLowering],
		lower_i32_leading_zeros,
		LowerI32LeadingZeros,
		"lower-i32-leading-zeros",
		Two,
		IntegerTargetLowering,
		"Lower i32 leading-zero count."
	),
	(
		[TargetLowering],
		lower_i32_trailing_zeros,
		LowerI32TrailingZeros,
		"lower-i32-trailing-zeros",
		Two,
		IntegerTargetLowering,
		"Lower i32 trailing-zero count."
	),
	(
		[TargetLowering],
		lower_i64_count_ones,
		LowerI64CountOnes,
		"lower-i64-count-ones",
		Two,
		IntegerTargetLowering,
		"Lower i64 population count."
	),
	(
		[TargetLowering],
		lower_i64_leading_zeros,
		LowerI64LeadingZeros,
		"lower-i64-leading-zeros",
		Two,
		IntegerTargetLowering,
		"Lower i64 leading-zero count."
	),
	(
		[TargetLowering],
		lower_i64_trailing_zeros,
		LowerI64TrailingZeros,
		"lower-i64-trailing-zeros",
		Two,
		IntegerTargetLowering,
		"Lower i64 trailing-zero count."
	),
	(
		[TargetLowering],
		lower_i32_add,
		LowerI32Add,
		"lower-i32-add",
		Two,
		IntegerTargetLowering,
		"Lower i32 addition."
	),
	(
		[TargetLowering],
		lower_i32_subtract,
		LowerI32Subtract,
		"lower-i32-subtract",
		Two,
		IntegerTargetLowering,
		"Lower i32 subtraction."
	),
	(
		[TargetLowering],
		lower_i32_multiply,
		LowerI32Multiply,
		"lower-i32-multiply",
		Two,
		IntegerTargetLowering,
		"Lower i32 multiplication."
	),
	(
		[TargetLowering],
		lower_i32_signed_divide,
		LowerI32SignedDivide,
		"lower-i32-signed-divide",
		Two,
		IntegerTargetLowering,
		"Lower i32 signed division."
	),
	(
		[TargetLowering],
		lower_i32_unsigned_divide,
		LowerI32UnsignedDivide,
		"lower-i32-unsigned-divide",
		Two,
		IntegerTargetLowering,
		"Lower i32 unsigned division."
	),
	(
		[TargetLowering],
		lower_i32_signed_remainder,
		LowerI32SignedRemainder,
		"lower-i32-signed-remainder",
		Two,
		IntegerTargetLowering,
		"Lower i32 signed remainder."
	),
	(
		[TargetLowering],
		lower_i32_unsigned_remainder,
		LowerI32UnsignedRemainder,
		"lower-i32-unsigned-remainder",
		Two,
		IntegerTargetLowering,
		"Lower i32 unsigned remainder."
	),
	(
		[TargetLowering],
		lower_i32_and,
		LowerI32And,
		"lower-i32-and",
		Two,
		IntegerTargetLowering,
		"Lower i32 bitwise AND."
	),
	(
		[TargetLowering],
		lower_i32_or,
		LowerI32Or,
		"lower-i32-or",
		Two,
		IntegerTargetLowering,
		"Lower i32 bitwise OR."
	),
	(
		[TargetLowering],
		lower_i32_exclusive_or,
		LowerI32ExclusiveOr,
		"lower-i32-exclusive-or",
		Two,
		IntegerTargetLowering,
		"Lower i32 bitwise XOR."
	),
	(
		[TargetLowering],
		lower_i32_shift_left,
		LowerI32ShiftLeft,
		"lower-i32-shift-left",
		Two,
		IntegerTargetLowering,
		"Lower i32 left shift."
	),
	(
		[TargetLowering],
		lower_i32_signed_shift_right,
		LowerI32SignedShiftRight,
		"lower-i32-signed-shift-right",
		Two,
		IntegerTargetLowering,
		"Lower i32 signed right shift."
	),
	(
		[TargetLowering],
		lower_i32_unsigned_shift_right,
		LowerI32UnsignedShiftRight,
		"lower-i32-unsigned-shift-right",
		Two,
		IntegerTargetLowering,
		"Lower i32 unsigned right shift."
	),
	(
		[TargetLowering],
		lower_i32_rotate_left,
		LowerI32RotateLeft,
		"lower-i32-rotate-left",
		Two,
		IntegerTargetLowering,
		"Lower i32 left rotation."
	),
	(
		[TargetLowering],
		lower_i32_rotate_right,
		LowerI32RotateRight,
		"lower-i32-rotate-right",
		Two,
		IntegerTargetLowering,
		"Lower i32 right rotation."
	),
	(
		[TargetLowering],
		lower_i64_add,
		LowerI64Add,
		"lower-i64-add",
		Two,
		IntegerTargetLowering,
		"Lower i64 addition."
	),
	(
		[TargetLowering],
		lower_i64_subtract,
		LowerI64Subtract,
		"lower-i64-subtract",
		Two,
		IntegerTargetLowering,
		"Lower i64 subtraction."
	),
	(
		[TargetLowering],
		lower_i64_multiply,
		LowerI64Multiply,
		"lower-i64-multiply",
		Two,
		IntegerTargetLowering,
		"Lower i64 multiplication."
	),
	(
		[TargetLowering],
		lower_i64_signed_divide,
		LowerI64SignedDivide,
		"lower-i64-signed-divide",
		Two,
		IntegerTargetLowering,
		"Lower i64 signed division."
	),
	(
		[TargetLowering],
		lower_i64_unsigned_divide,
		LowerI64UnsignedDivide,
		"lower-i64-unsigned-divide",
		Two,
		IntegerTargetLowering,
		"Lower i64 unsigned division."
	),
	(
		[TargetLowering],
		lower_i64_signed_remainder,
		LowerI64SignedRemainder,
		"lower-i64-signed-remainder",
		Two,
		IntegerTargetLowering,
		"Lower i64 signed remainder."
	),
	(
		[TargetLowering],
		lower_i64_unsigned_remainder,
		LowerI64UnsignedRemainder,
		"lower-i64-unsigned-remainder",
		Two,
		IntegerTargetLowering,
		"Lower i64 unsigned remainder."
	),
	(
		[TargetLowering],
		lower_i64_and,
		LowerI64And,
		"lower-i64-and",
		Two,
		IntegerTargetLowering,
		"Lower i64 bitwise AND."
	),
	(
		[TargetLowering],
		lower_i64_or,
		LowerI64Or,
		"lower-i64-or",
		Two,
		IntegerTargetLowering,
		"Lower i64 bitwise OR."
	),
	(
		[TargetLowering],
		lower_i64_exclusive_or,
		LowerI64ExclusiveOr,
		"lower-i64-exclusive-or",
		Two,
		IntegerTargetLowering,
		"Lower i64 bitwise XOR."
	),
	(
		[TargetLowering],
		lower_i64_shift_left,
		LowerI64ShiftLeft,
		"lower-i64-shift-left",
		Two,
		IntegerTargetLowering,
		"Lower i64 left shift."
	),
	(
		[TargetLowering],
		lower_i64_signed_shift_right,
		LowerI64SignedShiftRight,
		"lower-i64-signed-shift-right",
		Two,
		IntegerTargetLowering,
		"Lower i64 signed right shift."
	),
	(
		[TargetLowering],
		lower_i64_unsigned_shift_right,
		LowerI64UnsignedShiftRight,
		"lower-i64-unsigned-shift-right",
		Two,
		IntegerTargetLowering,
		"Lower i64 unsigned right shift."
	),
	(
		[TargetLowering],
		lower_i64_rotate_left,
		LowerI64RotateLeft,
		"lower-i64-rotate-left",
		Two,
		IntegerTargetLowering,
		"Lower i64 left rotation."
	),
	(
		[TargetLowering],
		lower_i64_rotate_right,
		LowerI64RotateRight,
		"lower-i64-rotate-right",
		Two,
		IntegerTargetLowering,
		"Lower i64 right rotation."
	),
	(
		[TargetLowering],
		lower_i32_equal,
		LowerI32Equal,
		"lower-i32-equal",
		Two,
		IntegerTargetLowering,
		"Lower i32 equality comparison."
	),
	(
		[TargetLowering],
		lower_i32_not_equal,
		LowerI32NotEqual,
		"lower-i32-not-equal",
		Two,
		IntegerTargetLowering,
		"Lower i32 inequality comparison."
	),
	(
		[TargetLowering],
		lower_i32_signed_less_than,
		LowerI32SignedLessThan,
		"lower-i32-signed-less-than",
		Two,
		IntegerTargetLowering,
		"Lower i32 signed less-than comparison."
	),
	(
		[TargetLowering],
		lower_i32_unsigned_less_than,
		LowerI32UnsignedLessThan,
		"lower-i32-unsigned-less-than",
		Two,
		IntegerTargetLowering,
		"Lower i32 unsigned less-than comparison."
	),
	(
		[TargetLowering],
		lower_i32_signed_less_than_equal,
		LowerI32SignedLessThanEqual,
		"lower-i32-signed-less-than-equal",
		Two,
		IntegerTargetLowering,
		"Lower i32 signed less-than-or-equal comparison."
	),
	(
		[TargetLowering],
		lower_i32_unsigned_less_than_equal,
		LowerI32UnsignedLessThanEqual,
		"lower-i32-unsigned-less-than-equal",
		Two,
		IntegerTargetLowering,
		"Lower i32 unsigned less-than-or-equal comparison."
	),
	(
		[TargetLowering],
		lower_i64_equal,
		LowerI64Equal,
		"lower-i64-equal",
		Two,
		IntegerTargetLowering,
		"Lower i64 equality comparison."
	),
	(
		[TargetLowering],
		lower_i64_not_equal,
		LowerI64NotEqual,
		"lower-i64-not-equal",
		Two,
		IntegerTargetLowering,
		"Lower i64 inequality comparison."
	),
	(
		[TargetLowering],
		lower_i64_signed_less_than,
		LowerI64SignedLessThan,
		"lower-i64-signed-less-than",
		Two,
		IntegerTargetLowering,
		"Lower i64 signed less-than comparison."
	),
	(
		[TargetLowering],
		lower_i64_unsigned_less_than,
		LowerI64UnsignedLessThan,
		"lower-i64-unsigned-less-than",
		Two,
		IntegerTargetLowering,
		"Lower i64 unsigned less-than comparison."
	),
	(
		[TargetLowering],
		lower_i64_signed_less_than_equal,
		LowerI64SignedLessThanEqual,
		"lower-i64-signed-less-than-equal",
		Two,
		IntegerTargetLowering,
		"Lower i64 signed less-than-or-equal comparison."
	),
	(
		[TargetLowering],
		lower_i64_unsigned_less_than_equal,
		LowerI64UnsignedLessThanEqual,
		"lower-i64-unsigned-less-than-equal",
		Two,
		IntegerTargetLowering,
		"Lower i64 unsigned less-than-or-equal comparison."
	),
	(
		[TargetLowering],
		lower_i64_narrow_to_i32,
		LowerI64NarrowToI32,
		"lower-i64-narrow-to-i32",
		Two,
		IntegerTargetLowering,
		"Lower i64-to-i32 narrowing."
	),
	(
		[TargetLowering],
		lower_i32_widen_to_i64,
		LowerI32WidenToI64,
		"lower-i32-widen-to-i64",
		Two,
		IntegerTargetLowering,
		"Lower i32-to-i64 widening."
	),
	(
		[TargetLowering],
		lower_i32_sign_extend,
		LowerI32SignExtend,
		"lower-i32-sign-extend",
		Two,
		IntegerTargetLowering,
		"Lower sign extension to i32."
	),
	(
		[TargetLowering],
		lower_i64_sign_extend,
		LowerI64SignExtend,
		"lower-i64-sign-extend",
		Two,
		IntegerTargetLowering,
		"Lower sign extension to i64."
	),
	(
		[TargetLowering],
		lower_signed_i32_to_f32,
		LowerSignedI32ToF32,
		"lower-signed-i32-to-f32",
		Two,
		IntegerTargetLowering,
		"Lower signed i32-to-f32 conversion."
	),
	(
		[TargetLowering],
		lower_unsigned_i32_to_f32,
		LowerUnsignedI32ToF32,
		"lower-unsigned-i32-to-f32",
		Two,
		IntegerTargetLowering,
		"Lower unsigned i32-to-f32 conversion."
	),
	(
		[TargetLowering],
		lower_signed_i32_to_f64,
		LowerSignedI32ToF64,
		"lower-signed-i32-to-f64",
		Two,
		IntegerTargetLowering,
		"Lower signed i32-to-f64 conversion."
	),
	(
		[TargetLowering],
		lower_unsigned_i32_to_f64,
		LowerUnsignedI32ToF64,
		"lower-unsigned-i32-to-f64",
		Two,
		IntegerTargetLowering,
		"Lower unsigned i32-to-f64 conversion."
	),
	(
		[TargetLowering],
		lower_signed_i64_to_f32,
		LowerSignedI64ToF32,
		"lower-signed-i64-to-f32",
		Two,
		IntegerTargetLowering,
		"Lower signed i64-to-f32 conversion."
	),
	(
		[TargetLowering],
		lower_unsigned_i64_to_f32,
		LowerUnsignedI64ToF32,
		"lower-unsigned-i64-to-f32",
		Two,
		IntegerTargetLowering,
		"Lower unsigned i64-to-f32 conversion."
	),
	(
		[TargetLowering],
		lower_signed_i64_to_f64,
		LowerSignedI64ToF64,
		"lower-signed-i64-to-f64",
		Two,
		IntegerTargetLowering,
		"Lower signed i64-to-f64 conversion."
	),
	(
		[TargetLowering],
		lower_unsigned_i64_to_f64,
		LowerUnsignedI64ToF64,
		"lower-unsigned-i64-to-f64",
		Two,
		IntegerTargetLowering,
		"Lower unsigned i64-to-f64 conversion."
	),
	(
		[TargetLowering],
		lower_integer_transmute_to_number,
		LowerIntegerTransmuteToNumber,
		"lower-integer-transmute-to-floating-point",
		Two,
		IntegerTargetLowering,
		"Lower integer-to-floating-point bit transmute."
	),
	(
		[TargetLowering],
		lower_f32_absolute,
		LowerF32Absolute,
		"lower-f32-absolute",
		Two,
		FloatingPointTargetLowering,
		"Lower f32 absolute value."
	),
	(
		[TargetLowering],
		lower_f32_negate,
		LowerF32Negate,
		"lower-f32-negate",
		Two,
		FloatingPointTargetLowering,
		"Lower f32 negation."
	),
	(
		[TargetLowering],
		lower_f32_square_root,
		LowerF32SquareRoot,
		"lower-f32-square-root",
		Two,
		FloatingPointTargetLowering,
		"Lower f32 square root."
	),
	(
		[TargetLowering],
		lower_f32_round_up,
		LowerF32RoundUp,
		"lower-f32-round-up",
		Two,
		FloatingPointTargetLowering,
		"Lower f32 rounding toward positive infinity."
	),
	(
		[TargetLowering],
		lower_f32_round_down,
		LowerF32RoundDown,
		"lower-f32-round-down",
		Two,
		FloatingPointTargetLowering,
		"Lower f32 rounding toward negative infinity."
	),
	(
		[TargetLowering],
		lower_f32_truncate,
		LowerF32Truncate,
		"lower-f32-truncate",
		Two,
		FloatingPointTargetLowering,
		"Lower f32 truncation toward zero."
	),
	(
		[TargetLowering],
		lower_f32_nearest,
		LowerF32Nearest,
		"lower-f32-nearest",
		Two,
		FloatingPointTargetLowering,
		"Lower f32 rounding to the nearest integer."
	),
	(
		[TargetLowering],
		lower_f64_absolute,
		LowerF64Absolute,
		"lower-f64-absolute",
		Two,
		FloatingPointTargetLowering,
		"Lower f64 absolute value."
	),
	(
		[TargetLowering],
		lower_f64_negate,
		LowerF64Negate,
		"lower-f64-negate",
		Two,
		FloatingPointTargetLowering,
		"Lower f64 negation."
	),
	(
		[TargetLowering],
		lower_f64_square_root,
		LowerF64SquareRoot,
		"lower-f64-square-root",
		Two,
		FloatingPointTargetLowering,
		"Lower f64 square root."
	),
	(
		[TargetLowering],
		lower_f64_round_up,
		LowerF64RoundUp,
		"lower-f64-round-up",
		Two,
		FloatingPointTargetLowering,
		"Lower f64 rounding toward positive infinity."
	),
	(
		[TargetLowering],
		lower_f64_round_down,
		LowerF64RoundDown,
		"lower-f64-round-down",
		Two,
		FloatingPointTargetLowering,
		"Lower f64 rounding toward negative infinity."
	),
	(
		[TargetLowering],
		lower_f64_truncate,
		LowerF64Truncate,
		"lower-f64-truncate",
		Two,
		FloatingPointTargetLowering,
		"Lower f64 truncation toward zero."
	),
	(
		[TargetLowering],
		lower_f64_nearest,
		LowerF64Nearest,
		"lower-f64-nearest",
		Two,
		FloatingPointTargetLowering,
		"Lower f64 rounding to the nearest integer."
	),
	(
		[TargetLowering],
		lower_f32_add,
		LowerF32Add,
		"lower-f32-add",
		Two,
		FloatingPointTargetLowering,
		"Lower f32 addition."
	),
	(
		[TargetLowering],
		lower_f32_subtract,
		LowerF32Subtract,
		"lower-f32-subtract",
		Two,
		FloatingPointTargetLowering,
		"Lower f32 subtraction."
	),
	(
		[TargetLowering],
		lower_f32_multiply,
		LowerF32Multiply,
		"lower-f32-multiply",
		Two,
		FloatingPointTargetLowering,
		"Lower f32 multiplication."
	),
	(
		[TargetLowering],
		lower_f32_divide,
		LowerF32Divide,
		"lower-f32-divide",
		Two,
		FloatingPointTargetLowering,
		"Lower f32 division."
	),
	(
		[TargetLowering],
		lower_f32_minimum,
		LowerF32Minimum,
		"lower-f32-minimum",
		Two,
		FloatingPointTargetLowering,
		"Lower f32 minimum."
	),
	(
		[TargetLowering],
		lower_f32_maximum,
		LowerF32Maximum,
		"lower-f32-maximum",
		Two,
		FloatingPointTargetLowering,
		"Lower f32 maximum."
	),
	(
		[TargetLowering],
		lower_f32_copy_sign,
		LowerF32CopySign,
		"lower-f32-copy-sign",
		Two,
		FloatingPointTargetLowering,
		"Lower f32 sign copying."
	),
	(
		[TargetLowering],
		lower_f64_add,
		LowerF64Add,
		"lower-f64-add",
		Two,
		FloatingPointTargetLowering,
		"Lower f64 addition."
	),
	(
		[TargetLowering],
		lower_f64_subtract,
		LowerF64Subtract,
		"lower-f64-subtract",
		Two,
		FloatingPointTargetLowering,
		"Lower f64 subtraction."
	),
	(
		[TargetLowering],
		lower_f64_multiply,
		LowerF64Multiply,
		"lower-f64-multiply",
		Two,
		FloatingPointTargetLowering,
		"Lower f64 multiplication."
	),
	(
		[TargetLowering],
		lower_f64_divide,
		LowerF64Divide,
		"lower-f64-divide",
		Two,
		FloatingPointTargetLowering,
		"Lower f64 division."
	),
	(
		[TargetLowering],
		lower_f64_minimum,
		LowerF64Minimum,
		"lower-f64-minimum",
		Two,
		FloatingPointTargetLowering,
		"Lower f64 minimum."
	),
	(
		[TargetLowering],
		lower_f64_maximum,
		LowerF64Maximum,
		"lower-f64-maximum",
		Two,
		FloatingPointTargetLowering,
		"Lower f64 maximum."
	),
	(
		[TargetLowering],
		lower_f64_copy_sign,
		LowerF64CopySign,
		"lower-f64-copy-sign",
		Two,
		FloatingPointTargetLowering,
		"Lower f64 sign copying."
	),
	(
		[TargetLowering],
		lower_f32_equal,
		LowerF32Equal,
		"lower-f32-equal",
		Two,
		FloatingPointTargetLowering,
		"Lower f32 equality comparison."
	),
	(
		[TargetLowering],
		lower_f32_not_equal,
		LowerF32NotEqual,
		"lower-f32-not-equal",
		Two,
		FloatingPointTargetLowering,
		"Lower f32 inequality comparison."
	),
	(
		[TargetLowering],
		lower_f32_less_than,
		LowerF32LessThan,
		"lower-f32-less-than",
		Two,
		FloatingPointTargetLowering,
		"Lower f32 less-than comparison."
	),
	(
		[TargetLowering],
		lower_f32_less_than_equal,
		LowerF32LessThanEqual,
		"lower-f32-less-than-equal",
		Two,
		FloatingPointTargetLowering,
		"Lower f32 less-than-or-equal comparison."
	),
	(
		[TargetLowering],
		lower_f64_equal,
		LowerF64Equal,
		"lower-f64-equal",
		Two,
		FloatingPointTargetLowering,
		"Lower f64 equality comparison."
	),
	(
		[TargetLowering],
		lower_f64_not_equal,
		LowerF64NotEqual,
		"lower-f64-not-equal",
		Two,
		FloatingPointTargetLowering,
		"Lower f64 inequality comparison."
	),
	(
		[TargetLowering],
		lower_f64_less_than,
		LowerF64LessThan,
		"lower-f64-less-than",
		Two,
		FloatingPointTargetLowering,
		"Lower f64 less-than comparison."
	),
	(
		[TargetLowering],
		lower_f64_less_than_equal,
		LowerF64LessThanEqual,
		"lower-f64-less-than-equal",
		Two,
		FloatingPointTargetLowering,
		"Lower f64 less-than-or-equal comparison."
	),
	(
		[TargetLowering],
		lower_f64_narrow_to_f32,
		LowerF64NarrowToF32,
		"lower-f64-narrow-to-f32",
		Two,
		FloatingPointTargetLowering,
		"Lower f64-to-f32 narrowing."
	),
	(
		[TargetLowering],
		lower_f32_widen_to_f64,
		LowerF32WidenToF64,
		"lower-f32-widen-to-f64",
		Two,
		FloatingPointTargetLowering,
		"Lower f32-to-f64 widening."
	),
	(
		[TargetLowering],
		lower_number_to_signed_i32_trapping,
		LowerNumberToSignedI32Trapping,
		"lower-floating-point-to-signed-i32-trapping",
		Two,
		FloatingPointTargetLowering,
		"Lower trapping floating-point-to-signed-i32 conversion."
	),
	(
		[TargetLowering],
		lower_number_to_signed_i32_saturating,
		LowerNumberToSignedI32Saturating,
		"lower-floating-point-to-signed-i32-saturating",
		Two,
		FloatingPointTargetLowering,
		"Lower saturating floating-point-to-signed-i32 conversion."
	),
	(
		[TargetLowering],
		lower_number_to_unsigned_i32_trapping,
		LowerNumberToUnsignedI32Trapping,
		"lower-floating-point-to-unsigned-i32-trapping",
		Two,
		FloatingPointTargetLowering,
		"Lower trapping floating-point-to-unsigned-i32 conversion."
	),
	(
		[TargetLowering],
		lower_number_to_unsigned_i32_saturating,
		LowerNumberToUnsignedI32Saturating,
		"lower-floating-point-to-unsigned-i32-saturating",
		Two,
		FloatingPointTargetLowering,
		"Lower saturating floating-point-to-unsigned-i32 conversion."
	),
	(
		[TargetLowering],
		lower_number_to_signed_i64_trapping,
		LowerNumberToSignedI64Trapping,
		"lower-floating-point-to-signed-i64-trapping",
		Two,
		FloatingPointTargetLowering,
		"Lower trapping floating-point-to-signed-i64 conversion."
	),
	(
		[TargetLowering],
		lower_number_to_signed_i64_saturating,
		LowerNumberToSignedI64Saturating,
		"lower-floating-point-to-signed-i64-saturating",
		Two,
		FloatingPointTargetLowering,
		"Lower saturating floating-point-to-signed-i64 conversion."
	),
	(
		[TargetLowering],
		lower_number_to_unsigned_i64_trapping,
		LowerNumberToUnsignedI64Trapping,
		"lower-floating-point-to-unsigned-i64-trapping",
		Two,
		FloatingPointTargetLowering,
		"Lower trapping floating-point-to-unsigned-i64 conversion."
	),
	(
		[TargetLowering],
		lower_number_to_unsigned_i64_saturating,
		LowerNumberToUnsignedI64Saturating,
		"lower-floating-point-to-unsigned-i64-saturating",
		Two,
		FloatingPointTargetLowering,
		"Lower saturating floating-point-to-unsigned-i64 conversion."
	),
	(
		[TargetLowering],
		lower_number_transmute_to_integer,
		LowerNumberTransmuteToInteger,
		"lower-floating-point-transmute-to-integer",
		Two,
		FloatingPointTargetLowering,
		"Lower floating-point-to-integer bit transmute."
	),
	(
		[TargetLowering],
		lower_i32_signed_narrow_load,
		LowerI32SignedNarrowLoad,
		"lower-i32-signed-narrow-load",
		Two,
		MemoryTargetLowering,
		"Lower a signed narrow memory load to i32."
	),
	(
		[TargetLowering],
		lower_i32_unsigned_narrow_load,
		LowerI32UnsignedNarrowLoad,
		"lower-i32-unsigned-narrow-load",
		Two,
		MemoryTargetLowering,
		"Lower an unsigned narrow memory load to i32."
	),
	(
		[TargetLowering],
		lower_i32_and_f32_load,
		LowerI32AndF32Load,
		"lower-i32-and-f32-load",
		Two,
		MemoryTargetLowering,
		"Lower a full-width i32 or f32 memory load."
	),
	(
		[TargetLowering],
		lower_i64_signed_narrow_load,
		LowerI64SignedNarrowLoad,
		"lower-i64-signed-narrow-load",
		Two,
		MemoryTargetLowering,
		"Lower a signed narrow memory load to i64."
	),
	(
		[TargetLowering],
		lower_i64_unsigned_narrow_load,
		LowerI64UnsignedNarrowLoad,
		"lower-i64-unsigned-narrow-load",
		Two,
		MemoryTargetLowering,
		"Lower an unsigned narrow memory load to i64."
	),
	(
		[TargetLowering],
		lower_i64_load,
		LowerI64Load,
		"lower-i64-load",
		Two,
		MemoryTargetLowering,
		"Lower a full-width i64 memory load."
	),
	(
		[TargetLowering],
		lower_f64_load,
		LowerF64Load,
		"lower-f64-load",
		Two,
		MemoryTargetLowering,
		"Lower a full-width f64 memory load."
	),
	(
		[TargetLowering],
		lower_i32_narrow_store,
		LowerI32NarrowStore,
		"lower-i32-narrow-store",
		Two,
		MemoryTargetLowering,
		"Lower a narrow i32 memory store."
	),
	(
		[TargetLowering],
		lower_i32_and_f32_store,
		LowerI32AndF32Store,
		"lower-i32-and-f32-store",
		Two,
		MemoryTargetLowering,
		"Lower a full-width i32 or f32 memory store."
	),
	(
		[TargetLowering],
		lower_i64_narrow_store,
		LowerI64NarrowStore,
		"lower-i64-narrow-store",
		Two,
		MemoryTargetLowering,
		"Lower a narrow i64 memory store."
	),
	(
		[TargetLowering],
		lower_i64_store,
		LowerI64Store,
		"lower-i64-store",
		Two,
		MemoryTargetLowering,
		"Lower a full-width i64 memory store."
	),
	(
		[TargetLowering],
		lower_f64_store,
		LowerF64Store,
		"lower-f64-store",
		Two,
		MemoryTargetLowering,
		"Lower a full-width f64 memory store."
	),
	(
		[TargetLowering],
		lower_table_get,
		LowerTableGet,
		"lower-table-get",
		Two,
		TableTargetLowering,
		"Lower a table element read."
	),
	(
		[TargetLowering],
		lower_table_set,
		LowerTableSet,
		"lower-table-set",
		Two,
		TableTargetLowering,
		"Lower a table element write."
	),
	(
		[TargetLowering],
		lower_table_size,
		LowerTableSize,
		"lower-table-size",
		Two,
		TableTargetLowering,
		"Lower a table size query."
	),
);
