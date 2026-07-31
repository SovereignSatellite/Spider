/// Classify a transformation by its standard cumulative optimization level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptimizationLevel {
	/// Group inexpensive canonicalization and constant folding.
	One,
	/// Group whole-region and target-aware simplification.
	Two,
	/// Group aggressive expression synthesis.
	Three,
}

macro_rules! define_domains {
	($(($variant:ident, $flag:literal, $help:literal, $heading:literal)),+ $(,)?) => {
		/// Classify a transformation by its broad semantic domain.
		#[derive(Clone, Copy, Debug, PartialEq, Eq)]
		pub enum OptimizationDomain {
			$(#[doc = $help] $variant),+
		}

		impl OptimizationDomain {
			/// List every broad semantic domain in declaration order.
			pub const ALL: &'static [Self] = &[$(Self::$variant),+];

			/// Return the long CLI flag for this domain.
			#[must_use]
			pub const fn flag(self) -> &'static str {
				match self {
					$(Self::$variant => $flag),+
				}
			}

			/// Describe this domain in CLI help.
			#[must_use]
			pub const fn help(self) -> &'static str {
				match self {
					$(Self::$variant => $help),+
				}
			}

			/// Return the long-help heading for exact transformations in this domain.
			#[must_use]
			pub const fn exact_help_heading(self) -> &'static str {
				match self {
					$(Self::$variant => $heading),+
				}
			}
		}
	};
}

define_domains!(
	(
		Aggregate,
		"aggregate-optimizations",
		"Enable or disable all aggregate-value optimizations.",
		"Individual aggregate-value optimizations"
	),
	(
		ControlFlow,
		"control-flow-optimizations",
		"Enable or disable all control-flow optimizations.",
		"Individual control-flow optimizations"
	),
	(
		FloatingPoint,
		"floating-point-optimizations",
		"Enable or disable all floating-point optimizations.",
		"Individual floating-point optimizations"
	),
	(
		Graph,
		"graph-optimizations",
		"Enable or disable all graph-structure optimizations.",
		"Individual graph optimizations"
	),
	(
		Integer,
		"integer-optimizations",
		"Enable or disable all integer optimizations.",
		"Individual integer optimizations"
	),
	(
		Memory,
		"memory-optimizations",
		"Enable or disable all memory optimizations.",
		"Individual memory optimizations"
	),
	(
		Mutable,
		"mutable-optimizations",
		"Enable or disable all mutable-value optimizations.",
		"Individual mutable-value optimizations"
	),
	(
		Reference,
		"reference-optimizations",
		"Enable or disable all reference optimizations.",
		"Individual reference optimizations"
	),
	(
		Table,
		"table-optimizations",
		"Enable or disable all table optimizations.",
		"Individual table optimizations"
	),
);

macro_rules! define_intents {
	($(($variant:ident, $domain:ident, $flag:literal, $help:literal)),+ $(,)?) => {
		/// Classify a transformation by its intent family.
		#[derive(Clone, Copy, Debug, PartialEq, Eq)]
		pub enum OptimizationIntent {
			$(#[doc = $help] $variant),+
		}

		impl OptimizationIntent {
			/// List every intent family in declaration order.
			pub const ALL: &'static [Self] = &[$(Self::$variant),+];

			/// Return the broad domain that owns this intent.
			#[must_use]
			pub const fn domain(self) -> OptimizationDomain {
				match self {
					$(Self::$variant => OptimizationDomain::$domain),+
				}
			}

			/// Return the long CLI flag for this intent.
			#[must_use]
			pub const fn flag(self) -> &'static str {
				match self {
					$(Self::$variant => $flag),+
				}
			}

			/// Describe this intent in CLI help.
			#[must_use]
			pub const fn help(self) -> &'static str {
				match self {
					$(Self::$variant => $help),+
				}
			}
		}
	};
}

define_intents!(
	(
		AggregateForwarding,
		Aggregate,
		"aggregate-forwarding",
		"Enable or disable forwarding through aggregate projections."
	),
	(
		CommonNodeElimination,
		Graph,
		"common-node-elimination",
		"Enable or disable elimination of congruent pure nodes."
	),
	(
		ControlFolding,
		ControlFlow,
		"control-folding",
		"Enable or disable folding of statically selected control regions."
	),
	(
		ControlMotion,
		ControlFlow,
		"control-motion",
		"Enable or disable motion of values unchanged within control regions."
	),
	(
		FloatingPointConversions,
		FloatingPoint,
		"floating-point-conversions",
		"Enable or disable removal of redundant floating-point and bit conversions."
	),
	(
		FloatingPointComparisons,
		FloatingPoint,
		"floating-point-comparisons",
		"Enable or disable floating-point comparison reductions."
	),
	(
		FloatingPointIdentities,
		FloatingPoint,
		"floating-point-identities",
		"Enable or disable exact floating-point identity reductions."
	),
	(
		FloatingPointTargetLowering,
		FloatingPoint,
		"floating-point-target-lowering",
		"Enable or disable target lowering of floating-point operations."
	),
	(
		IntegerConstantFolding,
		Integer,
		"integer-folding",
		"Enable or disable integer constant folding."
	),
	(
		IntegerConstantReassociation,
		Integer,
		"integer-reassociation",
		"Enable or disable reassociation of adjacent integer constants."
	),
	(
		IntegerComparisons,
		Integer,
		"integer-comparisons",
		"Enable or disable integer comparison reductions."
	),
	(
		IntegerConversions,
		Integer,
		"integer-conversions",
		"Enable or disable removal of redundant integer and bit conversions."
	),
	(
		IntegerIdentities,
		Integer,
		"integer-identities",
		"Enable or disable integer identity reductions."
	),
	(
		IntegerTargetLowering,
		Integer,
		"integer-target-lowering",
		"Enable or disable target lowering of integer operations."
	),
	(
		LuauFloatingPointReduction,
		FloatingPoint,
		"luau-floating-point-reductions",
		"Enable or disable reductions over Luau floating-point operations."
	),
	(
		LuauIntegerReduction,
		Integer,
		"luau-integer-reductions",
		"Enable or disable reductions over Luau bit32 operations."
	),
	(
		LuauI64RepresentationReduction,
		Integer,
		"luau-i64-representation-reductions",
		"Enable or disable reductions over the Luau i64 representation."
	),
	(
		MatchTruthTables,
		ControlFlow,
		"match-truth-tables",
		"Enable or disable Boolean truth-table reductions."
	),
	(
		MemoryForwarding,
		Memory,
		"memory-forwarding",
		"Enable or disable forwarding through adjacent memory operations."
	),
	(
		MemoryTargetLowering,
		Memory,
		"memory-target-lowering",
		"Enable or disable target lowering of memory operations."
	),
	(
		MutableForwarding,
		Mutable,
		"mutable-forwarding",
		"Enable or disable forwarding through mutable-value operations."
	),
	(
		ReferenceFolding,
		Reference,
		"reference-folding",
		"Enable or disable folding of reference predicates."
	),
	(
		TableForwarding,
		Table,
		"table-forwarding",
		"Enable or disable forwarding through adjacent table operations."
	),
	(
		TableTargetLowering,
		Table,
		"table-target-lowering",
		"Enable or disable target lowering of table operations."
	),
);
