use ir_graph::operation::{ExtendType, LoadType, StoreType, integer, number};

use super::Optimization;

impl Optimization {
	/// Return the exact folding leaf for an integer unary operation.
	#[must_use]
	pub const fn for_integer_unary_folding(
		kind: integer::Type,
		operator: integer::UnaryOperator,
	) -> Self {
		match (kind, operator) {
			(integer::Type::I32, integer::UnaryOperator::CountOnes) => Self::FoldI32CountOnes,
			(integer::Type::I32, integer::UnaryOperator::LeadingZeros) => Self::FoldI32LeadingZeros,
			(integer::Type::I32, integer::UnaryOperator::TrailingZeros) => {
				Self::FoldI32TrailingZeros
			}
			(integer::Type::I64, integer::UnaryOperator::CountOnes) => Self::FoldI64CountOnes,
			(integer::Type::I64, integer::UnaryOperator::LeadingZeros) => Self::FoldI64LeadingZeros,
			(integer::Type::I64, integer::UnaryOperator::TrailingZeros) => {
				Self::FoldI64TrailingZeros
			}
		}
	}

	/// Return the exact folding leaf for an integer binary operation.
	#[expect(
		clippy::too_many_lines,
		reason = "the exhaustive operation mapping stays local and follows declaration order"
	)]
	#[must_use]
	pub const fn for_integer_binary_folding(
		kind: integer::Type,
		operator: integer::BinaryOperator,
	) -> Self {
		match (kind, operator) {
			(integer::Type::I32, integer::BinaryOperator::Add) => Self::FoldI32Add,
			(integer::Type::I32, integer::BinaryOperator::Subtract) => Self::FoldI32Subtract,
			(integer::Type::I32, integer::BinaryOperator::Multiply) => Self::FoldI32Multiply,
			(integer::Type::I32, integer::BinaryOperator::Divide { is_signed: true }) => {
				Self::FoldI32SignedDivide
			}
			(integer::Type::I32, integer::BinaryOperator::Divide { is_signed: false }) => {
				Self::FoldI32UnsignedDivide
			}
			(integer::Type::I32, integer::BinaryOperator::Remainder { is_signed: true }) => {
				Self::FoldI32SignedRemainder
			}
			(integer::Type::I32, integer::BinaryOperator::Remainder { is_signed: false }) => {
				Self::FoldI32UnsignedRemainder
			}
			(integer::Type::I32, integer::BinaryOperator::And) => Self::FoldI32And,
			(integer::Type::I32, integer::BinaryOperator::Or) => Self::FoldI32Or,
			(integer::Type::I32, integer::BinaryOperator::ExclusiveOr) => Self::FoldI32ExclusiveOr,
			(integer::Type::I32, integer::BinaryOperator::ShiftLeft) => Self::FoldI32ShiftLeft,
			(integer::Type::I32, integer::BinaryOperator::ShiftRight { is_signed: true }) => {
				Self::FoldI32SignedShiftRight
			}
			(integer::Type::I32, integer::BinaryOperator::ShiftRight { is_signed: false }) => {
				Self::FoldI32UnsignedShiftRight
			}
			(integer::Type::I32, integer::BinaryOperator::RotateLeft) => Self::FoldI32RotateLeft,
			(integer::Type::I32, integer::BinaryOperator::RotateRight) => Self::FoldI32RotateRight,
			(integer::Type::I64, integer::BinaryOperator::Add) => Self::FoldI64Add,
			(integer::Type::I64, integer::BinaryOperator::Subtract) => Self::FoldI64Subtract,
			(integer::Type::I64, integer::BinaryOperator::Multiply) => Self::FoldI64Multiply,
			(integer::Type::I64, integer::BinaryOperator::Divide { is_signed: true }) => {
				Self::FoldI64SignedDivide
			}
			(integer::Type::I64, integer::BinaryOperator::Divide { is_signed: false }) => {
				Self::FoldI64UnsignedDivide
			}
			(integer::Type::I64, integer::BinaryOperator::Remainder { is_signed: true }) => {
				Self::FoldI64SignedRemainder
			}
			(integer::Type::I64, integer::BinaryOperator::Remainder { is_signed: false }) => {
				Self::FoldI64UnsignedRemainder
			}
			(integer::Type::I64, integer::BinaryOperator::And) => Self::FoldI64And,
			(integer::Type::I64, integer::BinaryOperator::Or) => Self::FoldI64Or,
			(integer::Type::I64, integer::BinaryOperator::ExclusiveOr) => Self::FoldI64ExclusiveOr,
			(integer::Type::I64, integer::BinaryOperator::ShiftLeft) => Self::FoldI64ShiftLeft,
			(integer::Type::I64, integer::BinaryOperator::ShiftRight { is_signed: true }) => {
				Self::FoldI64SignedShiftRight
			}
			(integer::Type::I64, integer::BinaryOperator::ShiftRight { is_signed: false }) => {
				Self::FoldI64UnsignedShiftRight
			}
			(integer::Type::I64, integer::BinaryOperator::RotateLeft) => Self::FoldI64RotateLeft,
			(integer::Type::I64, integer::BinaryOperator::RotateRight) => Self::FoldI64RotateRight,
		}
	}

	/// Return the exact folding leaf for an integer comparison.
	#[must_use]
	pub const fn for_integer_comparison_folding(
		kind: integer::Type,
		operator: integer::CompareOperator,
	) -> Self {
		match (kind, operator) {
			(integer::Type::I32, integer::CompareOperator::Equal) => Self::FoldI32Equal,
			(integer::Type::I32, integer::CompareOperator::NotEqual) => Self::FoldI32NotEqual,
			(integer::Type::I32, integer::CompareOperator::LessThan { is_signed: true }) => {
				Self::FoldI32SignedLessThan
			}
			(integer::Type::I32, integer::CompareOperator::LessThan { is_signed: false }) => {
				Self::FoldI32UnsignedLessThan
			}
			(integer::Type::I32, integer::CompareOperator::LessThanEqual { is_signed: true }) => {
				Self::FoldI32SignedLessThanEqual
			}
			(integer::Type::I32, integer::CompareOperator::LessThanEqual { is_signed: false }) => {
				Self::FoldI32UnsignedLessThanEqual
			}
			(integer::Type::I64, integer::CompareOperator::Equal) => Self::FoldI64Equal,
			(integer::Type::I64, integer::CompareOperator::NotEqual) => Self::FoldI64NotEqual,
			(integer::Type::I64, integer::CompareOperator::LessThan { is_signed: true }) => {
				Self::FoldI64SignedLessThan
			}
			(integer::Type::I64, integer::CompareOperator::LessThan { is_signed: false }) => {
				Self::FoldI64UnsignedLessThan
			}
			(integer::Type::I64, integer::CompareOperator::LessThanEqual { is_signed: true }) => {
				Self::FoldI64SignedLessThanEqual
			}
			(integer::Type::I64, integer::CompareOperator::LessThanEqual { is_signed: false }) => {
				Self::FoldI64UnsignedLessThanEqual
			}
		}
	}

	/// Return the exact target-lowering leaf for an integer unary operation.
	#[must_use]
	pub const fn for_integer_unary_lowering(
		kind: integer::Type,
		operator: integer::UnaryOperator,
	) -> Self {
		match (kind, operator) {
			(integer::Type::I32, integer::UnaryOperator::CountOnes) => Self::LowerI32CountOnes,
			(integer::Type::I32, integer::UnaryOperator::LeadingZeros) => {
				Self::LowerI32LeadingZeros
			}
			(integer::Type::I32, integer::UnaryOperator::TrailingZeros) => {
				Self::LowerI32TrailingZeros
			}
			(integer::Type::I64, integer::UnaryOperator::CountOnes) => Self::LowerI64CountOnes,
			(integer::Type::I64, integer::UnaryOperator::LeadingZeros) => {
				Self::LowerI64LeadingZeros
			}
			(integer::Type::I64, integer::UnaryOperator::TrailingZeros) => {
				Self::LowerI64TrailingZeros
			}
		}
	}

	/// Return the exact target-lowering leaf for an integer binary operation.
	#[expect(
		clippy::too_many_lines,
		reason = "the exhaustive operation mapping stays local and follows declaration order"
	)]
	#[must_use]
	pub const fn for_integer_binary_lowering(
		kind: integer::Type,
		operator: integer::BinaryOperator,
	) -> Self {
		match (kind, operator) {
			(integer::Type::I32, integer::BinaryOperator::Add) => Self::LowerI32Add,
			(integer::Type::I32, integer::BinaryOperator::Subtract) => Self::LowerI32Subtract,
			(integer::Type::I32, integer::BinaryOperator::Multiply) => Self::LowerI32Multiply,
			(integer::Type::I32, integer::BinaryOperator::Divide { is_signed: true }) => {
				Self::LowerI32SignedDivide
			}
			(integer::Type::I32, integer::BinaryOperator::Divide { is_signed: false }) => {
				Self::LowerI32UnsignedDivide
			}
			(integer::Type::I32, integer::BinaryOperator::Remainder { is_signed: true }) => {
				Self::LowerI32SignedRemainder
			}
			(integer::Type::I32, integer::BinaryOperator::Remainder { is_signed: false }) => {
				Self::LowerI32UnsignedRemainder
			}
			(integer::Type::I32, integer::BinaryOperator::And) => Self::LowerI32And,
			(integer::Type::I32, integer::BinaryOperator::Or) => Self::LowerI32Or,
			(integer::Type::I32, integer::BinaryOperator::ExclusiveOr) => Self::LowerI32ExclusiveOr,
			(integer::Type::I32, integer::BinaryOperator::ShiftLeft) => Self::LowerI32ShiftLeft,
			(integer::Type::I32, integer::BinaryOperator::ShiftRight { is_signed: true }) => {
				Self::LowerI32SignedShiftRight
			}
			(integer::Type::I32, integer::BinaryOperator::ShiftRight { is_signed: false }) => {
				Self::LowerI32UnsignedShiftRight
			}
			(integer::Type::I32, integer::BinaryOperator::RotateLeft) => Self::LowerI32RotateLeft,
			(integer::Type::I32, integer::BinaryOperator::RotateRight) => Self::LowerI32RotateRight,
			(integer::Type::I64, integer::BinaryOperator::Add) => Self::LowerI64Add,
			(integer::Type::I64, integer::BinaryOperator::Subtract) => Self::LowerI64Subtract,
			(integer::Type::I64, integer::BinaryOperator::Multiply) => Self::LowerI64Multiply,
			(integer::Type::I64, integer::BinaryOperator::Divide { is_signed: true }) => {
				Self::LowerI64SignedDivide
			}
			(integer::Type::I64, integer::BinaryOperator::Divide { is_signed: false }) => {
				Self::LowerI64UnsignedDivide
			}
			(integer::Type::I64, integer::BinaryOperator::Remainder { is_signed: true }) => {
				Self::LowerI64SignedRemainder
			}
			(integer::Type::I64, integer::BinaryOperator::Remainder { is_signed: false }) => {
				Self::LowerI64UnsignedRemainder
			}
			(integer::Type::I64, integer::BinaryOperator::And) => Self::LowerI64And,
			(integer::Type::I64, integer::BinaryOperator::Or) => Self::LowerI64Or,
			(integer::Type::I64, integer::BinaryOperator::ExclusiveOr) => Self::LowerI64ExclusiveOr,
			(integer::Type::I64, integer::BinaryOperator::ShiftLeft) => Self::LowerI64ShiftLeft,
			(integer::Type::I64, integer::BinaryOperator::ShiftRight { is_signed: true }) => {
				Self::LowerI64SignedShiftRight
			}
			(integer::Type::I64, integer::BinaryOperator::ShiftRight { is_signed: false }) => {
				Self::LowerI64UnsignedShiftRight
			}
			(integer::Type::I64, integer::BinaryOperator::RotateLeft) => Self::LowerI64RotateLeft,
			(integer::Type::I64, integer::BinaryOperator::RotateRight) => Self::LowerI64RotateRight,
		}
	}

	/// Return the exact target-lowering leaf for an integer comparison.
	#[must_use]
	pub const fn for_integer_comparison_lowering(
		kind: integer::Type,
		operator: integer::CompareOperator,
	) -> Self {
		match (kind, operator) {
			(integer::Type::I32, integer::CompareOperator::Equal) => Self::LowerI32Equal,
			(integer::Type::I32, integer::CompareOperator::NotEqual) => Self::LowerI32NotEqual,
			(integer::Type::I32, integer::CompareOperator::LessThan { is_signed: true }) => {
				Self::LowerI32SignedLessThan
			}
			(integer::Type::I32, integer::CompareOperator::LessThan { is_signed: false }) => {
				Self::LowerI32UnsignedLessThan
			}
			(integer::Type::I32, integer::CompareOperator::LessThanEqual { is_signed: true }) => {
				Self::LowerI32SignedLessThanEqual
			}
			(integer::Type::I32, integer::CompareOperator::LessThanEqual { is_signed: false }) => {
				Self::LowerI32UnsignedLessThanEqual
			}
			(integer::Type::I64, integer::CompareOperator::Equal) => Self::LowerI64Equal,
			(integer::Type::I64, integer::CompareOperator::NotEqual) => Self::LowerI64NotEqual,
			(integer::Type::I64, integer::CompareOperator::LessThan { is_signed: true }) => {
				Self::LowerI64SignedLessThan
			}
			(integer::Type::I64, integer::CompareOperator::LessThan { is_signed: false }) => {
				Self::LowerI64UnsignedLessThan
			}
			(integer::Type::I64, integer::CompareOperator::LessThanEqual { is_signed: true }) => {
				Self::LowerI64SignedLessThanEqual
			}
			(integer::Type::I64, integer::CompareOperator::LessThanEqual { is_signed: false }) => {
				Self::LowerI64UnsignedLessThanEqual
			}
		}
	}

	/// Return the exact target-lowering leaf for an integer sign extension.
	#[must_use]
	pub const fn for_integer_sign_extension_lowering(kind: ExtendType) -> Self {
		match kind {
			ExtendType::I32_S8 | ExtendType::I32_S16 => Self::LowerI32SignExtend,
			ExtendType::I64_S8 | ExtendType::I64_S16 | ExtendType::I64_S32 => {
				Self::LowerI64SignExtend
			}
		}
	}

	/// Return the exact target-lowering leaf for an integer-to-number conversion.
	#[must_use]
	pub const fn for_integer_to_number_lowering(
		from: integer::Type,
		to: number::Type,
		is_signed: bool,
	) -> Self {
		match (from, to, is_signed) {
			(integer::Type::I32, number::Type::F32, true) => Self::LowerSignedI32ToF32,
			(integer::Type::I32, number::Type::F32, false) => Self::LowerUnsignedI32ToF32,
			(integer::Type::I32, number::Type::F64, true) => Self::LowerSignedI32ToF64,
			(integer::Type::I32, number::Type::F64, false) => Self::LowerUnsignedI32ToF64,
			(integer::Type::I64, number::Type::F32, true) => Self::LowerSignedI64ToF32,
			(integer::Type::I64, number::Type::F32, false) => Self::LowerUnsignedI64ToF32,
			(integer::Type::I64, number::Type::F64, true) => Self::LowerSignedI64ToF64,
			(integer::Type::I64, number::Type::F64, false) => Self::LowerUnsignedI64ToF64,
		}
	}

	/// Return the exact target-lowering leaf for an integer-to-number bit transmute.
	#[must_use]
	pub const fn for_integer_transmute_to_number_lowering(from: integer::Type) -> Self {
		match from {
			integer::Type::I32 | integer::Type::I64 => Self::LowerIntegerTransmuteToNumber,
		}
	}

	/// Return the exact target-lowering leaf for a floating-point unary operation.
	#[must_use]
	pub const fn for_number_unary_lowering(
		kind: number::Type,
		operator: number::UnaryOperator,
	) -> Self {
		match (kind, operator) {
			(number::Type::F32, number::UnaryOperator::Absolute) => Self::LowerF32Absolute,
			(number::Type::F32, number::UnaryOperator::Negate) => Self::LowerF32Negate,
			(number::Type::F32, number::UnaryOperator::SquareRoot) => Self::LowerF32SquareRoot,
			(number::Type::F32, number::UnaryOperator::RoundUp) => Self::LowerF32RoundUp,
			(number::Type::F32, number::UnaryOperator::RoundDown) => Self::LowerF32RoundDown,
			(number::Type::F32, number::UnaryOperator::Truncate) => Self::LowerF32Truncate,
			(number::Type::F32, number::UnaryOperator::Nearest) => Self::LowerF32Nearest,
			(number::Type::F64, number::UnaryOperator::Absolute) => Self::LowerF64Absolute,
			(number::Type::F64, number::UnaryOperator::Negate) => Self::LowerF64Negate,
			(number::Type::F64, number::UnaryOperator::SquareRoot) => Self::LowerF64SquareRoot,
			(number::Type::F64, number::UnaryOperator::RoundUp) => Self::LowerF64RoundUp,
			(number::Type::F64, number::UnaryOperator::RoundDown) => Self::LowerF64RoundDown,
			(number::Type::F64, number::UnaryOperator::Truncate) => Self::LowerF64Truncate,
			(number::Type::F64, number::UnaryOperator::Nearest) => Self::LowerF64Nearest,
		}
	}

	/// Return the exact target-lowering leaf for a floating-point binary operation.
	#[must_use]
	pub const fn for_number_binary_lowering(
		kind: number::Type,
		operator: number::BinaryOperator,
	) -> Self {
		match (kind, operator) {
			(number::Type::F32, number::BinaryOperator::Add) => Self::LowerF32Add,
			(number::Type::F32, number::BinaryOperator::Subtract) => Self::LowerF32Subtract,
			(number::Type::F32, number::BinaryOperator::Multiply) => Self::LowerF32Multiply,
			(number::Type::F32, number::BinaryOperator::Divide) => Self::LowerF32Divide,
			(number::Type::F32, number::BinaryOperator::Minimum) => Self::LowerF32Minimum,
			(number::Type::F32, number::BinaryOperator::Maximum) => Self::LowerF32Maximum,
			(number::Type::F32, number::BinaryOperator::CopySign) => Self::LowerF32CopySign,
			(number::Type::F64, number::BinaryOperator::Add) => Self::LowerF64Add,
			(number::Type::F64, number::BinaryOperator::Subtract) => Self::LowerF64Subtract,
			(number::Type::F64, number::BinaryOperator::Multiply) => Self::LowerF64Multiply,
			(number::Type::F64, number::BinaryOperator::Divide) => Self::LowerF64Divide,
			(number::Type::F64, number::BinaryOperator::Minimum) => Self::LowerF64Minimum,
			(number::Type::F64, number::BinaryOperator::Maximum) => Self::LowerF64Maximum,
			(number::Type::F64, number::BinaryOperator::CopySign) => Self::LowerF64CopySign,
		}
	}

	/// Return the exact target-lowering leaf for a floating-point comparison.
	#[must_use]
	pub const fn for_number_comparison_lowering(
		kind: number::Type,
		operator: number::CompareOperator,
	) -> Self {
		match (kind, operator) {
			(number::Type::F32, number::CompareOperator::Equal) => Self::LowerF32Equal,
			(number::Type::F32, number::CompareOperator::NotEqual) => Self::LowerF32NotEqual,
			(number::Type::F32, number::CompareOperator::LessThan) => Self::LowerF32LessThan,
			(number::Type::F32, number::CompareOperator::LessThanEqual) => {
				Self::LowerF32LessThanEqual
			}
			(number::Type::F64, number::CompareOperator::Equal) => Self::LowerF64Equal,
			(number::Type::F64, number::CompareOperator::NotEqual) => Self::LowerF64NotEqual,
			(number::Type::F64, number::CompareOperator::LessThan) => Self::LowerF64LessThan,
			(number::Type::F64, number::CompareOperator::LessThanEqual) => {
				Self::LowerF64LessThanEqual
			}
		}
	}

	/// Return the exact target-lowering leaf for a number-to-integer conversion.
	#[must_use]
	pub const fn for_number_to_integer_lowering(
		from: number::Type,
		to: integer::Type,
		is_signed: bool,
		is_saturating: bool,
	) -> Self {
		match (from, to, is_signed, is_saturating) {
			(number::Type::F32 | number::Type::F64, integer::Type::I32, true, false) => {
				Self::LowerNumberToSignedI32Trapping
			}
			(number::Type::F32 | number::Type::F64, integer::Type::I32, true, true) => {
				Self::LowerNumberToSignedI32Saturating
			}
			(number::Type::F32 | number::Type::F64, integer::Type::I32, false, false) => {
				Self::LowerNumberToUnsignedI32Trapping
			}
			(number::Type::F32 | number::Type::F64, integer::Type::I32, false, true) => {
				Self::LowerNumberToUnsignedI32Saturating
			}
			(number::Type::F32 | number::Type::F64, integer::Type::I64, true, false) => {
				Self::LowerNumberToSignedI64Trapping
			}
			(number::Type::F32 | number::Type::F64, integer::Type::I64, true, true) => {
				Self::LowerNumberToSignedI64Saturating
			}
			(number::Type::F32 | number::Type::F64, integer::Type::I64, false, false) => {
				Self::LowerNumberToUnsignedI64Trapping
			}
			(number::Type::F32 | number::Type::F64, integer::Type::I64, false, true) => {
				Self::LowerNumberToUnsignedI64Saturating
			}
		}
	}

	/// Return the exact target-lowering leaf for a number-to-integer bit transmute.
	#[must_use]
	pub const fn for_number_transmute_to_integer_lowering(from: number::Type) -> Self {
		match from {
			number::Type::F32 | number::Type::F64 => Self::LowerNumberTransmuteToInteger,
		}
	}

	/// Return the exact target-lowering leaf for a memory load.
	#[must_use]
	pub const fn for_memory_load_lowering(kind: LoadType) -> Self {
		match kind {
			LoadType::I32_S8 | LoadType::I32_S16 => Self::LowerI32SignedNarrowLoad,
			LoadType::I32_U8 | LoadType::I32_U16 => Self::LowerI32UnsignedNarrowLoad,
			LoadType::I32 | LoadType::F32 => Self::LowerI32AndF32Load,
			LoadType::I64_S8 | LoadType::I64_S16 | LoadType::I64_S32 => {
				Self::LowerI64SignedNarrowLoad
			}
			LoadType::I64_U8 | LoadType::I64_U16 | LoadType::I64_U32 => {
				Self::LowerI64UnsignedNarrowLoad
			}
			LoadType::I64 => Self::LowerI64Load,
			LoadType::F64 => Self::LowerF64Load,
		}
	}

	/// Return the exact target-lowering leaf for a memory store.
	#[must_use]
	pub const fn for_memory_store_lowering(kind: StoreType) -> Self {
		match kind {
			StoreType::I32_I8 | StoreType::I32_I16 => Self::LowerI32NarrowStore,
			StoreType::I32 | StoreType::F32 => Self::LowerI32AndF32Store,
			StoreType::I64_I8 | StoreType::I64_I16 | StoreType::I64_I32 => {
				Self::LowerI64NarrowStore
			}
			StoreType::I64 => Self::LowerI64Store,
			StoreType::F64 => Self::LowerF64Store,
		}
	}
}
