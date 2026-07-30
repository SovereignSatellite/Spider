//! ISLE context implementation for graph regions.

use ir_graph::{
	Link, Node,
	operation::{
		Aggregate, Extract, IntegerNarrow, IntegerSignExtend, IntegerTransmuteToNumber,
		IntegerWiden, LoadType, Location, MemoryLoad, MemoryStore, MutableGet, MutableNew,
		MutableSet, NumberTransmuteToInteger, RefIsNull, StoreType, TableGet, TableSet,
		integer::{
			BinaryOperation as IntegerBinaryOperation, BinaryOperator as IntegerBinaryOperator,
			CompareOperation as IntegerCompareOperation, CompareOperator as IntegerCompareOperator,
			Type as IntegerType,
		},
		number::{
			Type as NumberType, UnaryOperation as NumberUnaryOperation,
			UnaryOperator as NumberUnaryOperator,
		},
	},
	tracer::identity_source,
};
use luau_foreign::{
	Bit32And, Bit32ArShift, Bit32LRotate, Bit32LShift, Bit32Or, Bit32RRotate, Bit32RShift,
	Bit32Xor, FromBitsI64,
};

use super::{
	internal::Context,
	luau::{
		self, Bit32BinaryOperator, Bit32UnaryOperator, LuauArithmeticOperator, LuauBinaryOperator,
		LuauCompareOperator, LuauUnaryOperator,
	},
};

/// A newtype wrapper for implementing the ISLE `Context` trait on a region.
pub struct RegionContext<'nodes>(pub &'nodes mut Vec<Node>);

impl RegionContext<'_> {
	fn at(&self, link: Link) -> &Node {
		&self.0[usize::try_from(link.0).unwrap()]
	}

	fn trace(&self, link: Link) -> Link {
		identity_source(self.0, link)
	}
}

impl Context for RegionContext<'_> {
	fn get_i32(&mut self, arg0: Link) -> Option<i32> {
		if let &Node::I32(value) = self.at(arg0) {
			Some(value)
		} else {
			None
		}
	}

	fn add_i32(&mut self, arg0: i32) -> Link {
		Node::add_i32_into(self.0, arg0)
	}

	fn get_i64(&mut self, arg0: Link) -> Option<i64> {
		if let &Node::I64(value) = self.at(arg0) {
			Some(value)
		} else {
			None
		}
	}

	fn add_i64(&mut self, arg0: i64) -> Link {
		Node::add_i64_into(self.0, arg0)
	}

	fn get_integer_binary_operation(
		&mut self,
		arg0: Link,
	) -> Option<(Link, Link, IntegerType, IntegerBinaryOperator)> {
		if let &Node::IntegerBinaryOperation(IntegerBinaryOperation {
			lhs,
			rhs,
			kind,
			operator,
		}) = self.at(arg0)
		{
			Some((self.trace(lhs), self.trace(rhs), kind, operator))
		} else {
			None
		}
	}

	fn add_integer_binary_operation(
		&mut self,
		arg0: Link,
		arg1: Link,
		arg2: &IntegerType,
		arg3: &IntegerBinaryOperator,
	) -> Link {
		IntegerBinaryOperation::add_into(self.0, arg0, arg1, *arg2, *arg3)
	}

	fn get_integer_compare_operation(
		&mut self,
		arg0: Link,
	) -> Option<(Link, Link, IntegerType, IntegerCompareOperator)> {
		if let &Node::IntegerCompareOperation(IntegerCompareOperation {
			lhs,
			rhs,
			kind,
			operator,
		}) = self.at(arg0)
		{
			Some((self.trace(lhs), self.trace(rhs), kind, operator))
		} else {
			None
		}
	}

	fn reduce_boolean_comparison(
		&mut self,
		arg0: Link,
		arg1: &IntegerCompareOperator,
		arg2: i32,
	) -> Option<Link> {
		let comparison = arg0;

		match (*arg1, arg2) {
			(IntegerCompareOperator::NotEqual, 0_i32) | (IntegerCompareOperator::Equal, 1_i32) => {
				return Some(comparison);
			}
			(IntegerCompareOperator::Equal, 0_i32) | (IntegerCompareOperator::NotEqual, 1_i32) => {}
			_ => return None,
		}

		let &Node::IntegerCompareOperation(operation) = self.at(comparison) else {
			unreachable!()
		};
		let (left_operand, right_operand, operator) = match operation.operator {
			IntegerCompareOperator::Equal => (
				operation.lhs,
				operation.rhs,
				IntegerCompareOperator::NotEqual,
			),
			IntegerCompareOperator::NotEqual => {
				(operation.lhs, operation.rhs, IntegerCompareOperator::Equal)
			}
			IntegerCompareOperator::LessThan { is_signed } => (
				operation.rhs,
				operation.lhs,
				IntegerCompareOperator::LessThanEqual { is_signed },
			),
			IntegerCompareOperator::LessThanEqual { is_signed } => (
				operation.rhs,
				operation.lhs,
				IntegerCompareOperator::LessThan { is_signed },
			),
		};

		Some(IntegerCompareOperation::add_into(
			self.0,
			left_operand,
			right_operand,
			operation.kind,
			operator,
		))
	}

	fn raw_add_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		arg0.wrapping_add(arg1)
	}

	fn raw_sub_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		arg0.wrapping_sub(arg1)
	}

	fn raw_multiply_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		arg0.wrapping_mul(arg1)
	}

	fn raw_and_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		arg0 & arg1
	}

	fn raw_or_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		arg0 | arg1
	}

	fn raw_exclusive_or_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		arg0 ^ arg1
	}

	fn raw_shift_left_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		arg0.wrapping_shl(arg1.cast_unsigned())
	}

	fn raw_shift_right_signed_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		arg0.wrapping_shr(arg1.cast_unsigned())
	}

	fn raw_shift_right_unsigned_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		arg0.cast_unsigned()
			.wrapping_shr(arg1.cast_unsigned())
			.cast_signed()
	}

	fn raw_rotate_left_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		arg0.rotate_left(arg1.cast_unsigned())
	}

	fn raw_rotate_right_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		arg0.rotate_right(arg1.cast_unsigned())
	}

	fn raw_divide_signed_i32(&mut self, arg0: i32, arg1: i32) -> Option<i32> {
		arg0.checked_div(arg1)
	}

	fn raw_divide_unsigned_i32(&mut self, arg0: i32, arg1: i32) -> Option<i32> {
		arg0.cast_unsigned()
			.checked_div(arg1.cast_unsigned())
			.map(u32::cast_signed)
	}

	fn raw_remainder_signed_i32(&mut self, arg0: i32, arg1: i32) -> Option<i32> {
		arg0.checked_rem(arg1)
	}

	fn raw_remainder_unsigned_i32(&mut self, arg0: i32, arg1: i32) -> Option<i32> {
		arg0.cast_unsigned()
			.checked_rem(arg1.cast_unsigned())
			.map(u32::cast_signed)
	}

	fn raw_compare_equal_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		i32::from(arg0 == arg1)
	}

	fn raw_compare_not_equal_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		i32::from(arg0 != arg1)
	}

	fn raw_compare_less_than_signed_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		i32::from(arg0 < arg1)
	}

	fn raw_compare_less_than_unsigned_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		i32::from(arg0.cast_unsigned() < arg1.cast_unsigned())
	}

	fn raw_compare_less_than_equal_signed_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		i32::from(arg0 <= arg1)
	}

	fn raw_compare_less_than_equal_unsigned_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		i32::from(arg0.cast_unsigned() <= arg1.cast_unsigned())
	}

	fn raw_add_i64(&mut self, arg0: i64, arg1: i64) -> i64 {
		arg0.wrapping_add(arg1)
	}

	fn raw_sub_i64(&mut self, arg0: i64, arg1: i64) -> i64 {
		arg0.wrapping_sub(arg1)
	}

	fn raw_multiply_i64(&mut self, arg0: i64, arg1: i64) -> i64 {
		arg0.wrapping_mul(arg1)
	}

	fn raw_and_i64(&mut self, arg0: i64, arg1: i64) -> i64 {
		arg0 & arg1
	}

	fn raw_or_i64(&mut self, arg0: i64, arg1: i64) -> i64 {
		arg0 | arg1
	}

	fn raw_exclusive_or_i64(&mut self, arg0: i64, arg1: i64) -> i64 {
		arg0 ^ arg1
	}

	fn raw_shift_left_i64(&mut self, arg0: i64, arg1: i64) -> i64 {
		arg0.wrapping_shl(shift_count_i64(arg1))
	}

	fn raw_shift_right_signed_i64(&mut self, arg0: i64, arg1: i64) -> i64 {
		arg0.wrapping_shr(shift_count_i64(arg1))
	}

	fn raw_shift_right_unsigned_i64(&mut self, arg0: i64, arg1: i64) -> i64 {
		arg0.cast_unsigned()
			.wrapping_shr(shift_count_i64(arg1))
			.cast_signed()
	}

	fn raw_rotate_left_i64(&mut self, arg0: i64, arg1: i64) -> i64 {
		arg0.rotate_left(shift_count_i64(arg1))
	}

	fn raw_rotate_right_i64(&mut self, arg0: i64, arg1: i64) -> i64 {
		arg0.rotate_right(shift_count_i64(arg1))
	}

	fn raw_divide_signed_i64(&mut self, arg0: i64, arg1: i64) -> Option<i64> {
		arg0.checked_div(arg1)
	}

	fn raw_divide_unsigned_i64(&mut self, arg0: i64, arg1: i64) -> Option<i64> {
		arg0.cast_unsigned()
			.checked_div(arg1.cast_unsigned())
			.map(u64::cast_signed)
	}

	fn raw_remainder_signed_i64(&mut self, arg0: i64, arg1: i64) -> Option<i64> {
		arg0.checked_rem(arg1)
	}

	fn raw_remainder_unsigned_i64(&mut self, arg0: i64, arg1: i64) -> Option<i64> {
		arg0.cast_unsigned()
			.checked_rem(arg1.cast_unsigned())
			.map(u64::cast_signed)
	}

	fn raw_compare_equal_i64(&mut self, arg0: i64, arg1: i64) -> i32 {
		i32::from(arg0 == arg1)
	}

	fn raw_compare_not_equal_i64(&mut self, arg0: i64, arg1: i64) -> i32 {
		i32::from(arg0 != arg1)
	}

	fn raw_compare_less_than_signed_i64(&mut self, arg0: i64, arg1: i64) -> i32 {
		i32::from(arg0 < arg1)
	}

	fn raw_compare_less_than_unsigned_i64(&mut self, arg0: i64, arg1: i64) -> i32 {
		i32::from(arg0.cast_unsigned() < arg1.cast_unsigned())
	}

	fn raw_compare_less_than_equal_signed_i64(&mut self, arg0: i64, arg1: i64) -> i32 {
		i32::from(arg0 <= arg1)
	}

	fn raw_compare_less_than_equal_unsigned_i64(&mut self, arg0: i64, arg1: i64) -> i32 {
		i32::from(arg0.cast_unsigned() <= arg1.cast_unsigned())
	}

	fn get_f32(&mut self, arg0: Link) -> Option<f32> {
		if let &Node::F32(value) = self.at(arg0) {
			Some(value)
		} else {
			None
		}
	}

	fn add_f32(&mut self, arg0: f32) -> Link {
		Node::add_f32_into(self.0, arg0)
	}

	fn get_f64(&mut self, arg0: Link) -> Option<f64> {
		if let &Node::F64(value) = self.at(arg0) {
			Some(value)
		} else {
			None
		}
	}

	fn add_f64(&mut self, arg0: f64) -> Link {
		Node::add_f64_into(self.0, arg0)
	}

	fn get_mutable_new(&mut self, arg0: Link) -> Option<Link> {
		if let &Node::MutableNew(MutableNew { initializer }) = self.at(arg0) {
			Some(self.trace(initializer))
		} else {
			None
		}
	}

	fn get_mutable_get(&mut self, arg0: Link) -> Option<Link> {
		if let &Node::MutableGet(MutableGet { source }) = self.at(arg0) {
			Some(self.trace(source))
		} else {
			None
		}
	}

	fn get_mutable_set(&mut self, arg0: Link) -> Option<(Link, Link)> {
		if let &Node::MutableSet(MutableSet {
			destination,
			source,
		}) = self.at(arg0)
		{
			Some((self.trace(destination), self.trace(source)))
		} else {
			None
		}
	}

	fn get_table_get(&mut self, arg0: Link) -> Option<(Link, Link)> {
		if let &Node::TableGet(TableGet {
			source: Location { reference, offset },
		}) = self.at(arg0)
		{
			Some((self.trace(reference), self.trace(offset)))
		} else {
			None
		}
	}

	fn get_table_set(&mut self, arg0: Link) -> Option<(Link, Link, Link)> {
		if let &Node::TableSet(TableSet {
			destination: Location { reference, offset },
			source,
		}) = self.at(arg0)
		{
			Some((
				self.trace(reference),
				self.trace(offset),
				self.trace(source),
			))
		} else {
			None
		}
	}

	fn get_memory_load(&mut self, arg0: Link) -> Option<(Link, Link, LoadType)> {
		if let &Node::MemoryLoad(MemoryLoad {
			source: Location { reference, offset },
			kind,
		}) = self.at(arg0)
		{
			Some((self.trace(reference), self.trace(offset), kind))
		} else {
			None
		}
	}

	fn get_memory_store(&mut self, arg0: Link) -> Option<(Link, Link, Link, StoreType)> {
		if let &Node::MemoryStore(MemoryStore {
			destination: Location { reference, offset },
			source,
			kind,
		}) = self.at(arg0)
		{
			Some((
				self.trace(reference),
				self.trace(offset),
				self.trace(source),
				kind,
			))
		} else {
			None
		}
	}

	fn get_integer_narrow(&mut self, arg0: Link) -> Option<Link> {
		if let &Node::IntegerNarrow(IntegerNarrow { source }) = self.at(arg0) {
			Some(self.trace(source))
		} else {
			None
		}
	}

	fn get_integer_widen(&mut self, arg0: Link) -> Option<Link> {
		if let &Node::IntegerWiden(IntegerWiden { source }) = self.at(arg0) {
			Some(self.trace(source))
		} else {
			None
		}
	}

	fn add_integer_widen(&mut self, arg0: Link) -> Link {
		IntegerWiden::add_into(self.0, arg0)
	}

	fn get_sign_extend_idempotent(&mut self, arg0: Link) -> Option<Link> {
		let &Node::IntegerSignExtend(IntegerSignExtend {
			source,
			kind: outer,
		}) = self.at(arg0)
		else {
			return None;
		};
		let &Node::IntegerSignExtend(IntegerSignExtend { kind: inner, .. }) =
			self.at(self.trace(source))
		else {
			return None;
		};

		(outer == inner).then(|| self.trace(source))
	}

	fn get_transmute_int_round_trip(&mut self, arg0: Link) -> Option<Link> {
		let &Node::IntegerTransmuteToNumber(IntegerTransmuteToNumber {
			source,
			from: integer,
		}) = self.at(arg0)
		else {
			return None;
		};
		let &Node::NumberTransmuteToInteger(NumberTransmuteToInteger {
			source: inner,
			from: number,
		}) = self.at(self.trace(source))
		else {
			return None;
		};

		is_transmute_width_matched(integer, number).then(|| self.trace(inner))
	}

	fn get_transmute_number_round_trip(&mut self, arg0: Link) -> Option<Link> {
		let &Node::NumberTransmuteToInteger(NumberTransmuteToInteger {
			source,
			from: number,
		}) = self.at(arg0)
		else {
			return None;
		};
		let &Node::IntegerTransmuteToNumber(IntegerTransmuteToNumber {
			source: inner,
			from: integer,
		}) = self.at(self.trace(source))
		else {
			return None;
		};

		is_transmute_width_matched(integer, number).then(|| self.trace(inner))
	}

	fn get_extract_of_aggregate(&mut self, arg0: Link) -> Option<Link> {
		let &Node::Extract(Extract { source, index }) = self.at(arg0) else {
			return None;
		};
		let source = self.trace(source);
		let Node::Aggregate(Aggregate { fields }) = self.at(source) else {
			return None;
		};

		fields
			.get(usize::try_from(index).unwrap())
			.copied()
			.map(|field| self.trace(field))
	}

	fn get_ref_is_null(&mut self, arg0: Link) -> Option<Link> {
		if let &Node::RefIsNull(RefIsNull { source }) = self.at(arg0) {
			Some(self.trace(source))
		} else {
			None
		}
	}

	fn get_null(&mut self, arg0: Link) -> Option<()> {
		matches!(self.at(arg0), Node::Null).then_some(())
	}

	fn get_number_unary_operation(
		&mut self,
		arg0: Link,
	) -> Option<(Link, NumberType, NumberUnaryOperator)> {
		if let &Node::NumberUnaryOperation(NumberUnaryOperation {
			source,
			kind,
			operator,
		}) = self.at(arg0)
		{
			Some((self.trace(source), kind, operator))
		} else {
			None
		}
	}

	fn add_number_unary_operation(
		&mut self,
		arg0: Link,
		arg1: &NumberType,
		arg2: &NumberUnaryOperator,
	) -> Link {
		NumberUnaryOperation::add_into(self.0, arg0, *arg1, *arg2)
	}

	fn get_bit32_binary_operation(
		&mut self,
		arg0: Link,
	) -> Option<(Link, Link, Bit32BinaryOperator)> {
		let Node::Foreign(foreign) = self.at(arg0) else {
			return None;
		};
		let (lhs, rhs, operator) = luau::bit32_binary_operation(&**foreign)?;

		Some((self.trace(lhs), self.trace(rhs), operator))
	}

	fn add_bit32_binary_operation(
		&mut self,
		arg0: Link,
		arg1: Link,
		arg2: &Bit32BinaryOperator,
	) -> Link {
		match *arg2 {
			Bit32BinaryOperator::And => Bit32And::add_into(self.0, arg0, arg1),
			Bit32BinaryOperator::Or => Bit32Or::add_into(self.0, arg0, arg1),
			Bit32BinaryOperator::ExclusiveOr => Bit32Xor::add_into(self.0, arg0, arg1),
			Bit32BinaryOperator::ShiftLeft => Bit32LShift::add_into(self.0, arg0, arg1),
			Bit32BinaryOperator::ShiftRightUnsigned => Bit32RShift::add_into(self.0, arg0, arg1),
			Bit32BinaryOperator::ShiftRightSigned => Bit32ArShift::add_into(self.0, arg0, arg1),
			Bit32BinaryOperator::RotateLeft => Bit32LRotate::add_into(self.0, arg0, arg1),
			Bit32BinaryOperator::RotateRight => Bit32RRotate::add_into(self.0, arg0, arg1),
		}
	}

	fn get_bit32_unary_operation(&mut self, arg0: Link) -> Option<(Link, Bit32UnaryOperator)> {
		let Node::Foreign(foreign) = self.at(arg0) else {
			return None;
		};
		let (source, operator) = luau::bit32_unary_operation(&**foreign)?;

		Some((self.trace(source), operator))
	}

	fn get_bit32_canonical(&mut self, arg0: Link) -> Option<Link> {
		let Node::Foreign(foreign) = self.at(arg0) else {
			return None;
		};

		luau::is_bit32_canonical(&**foreign).then_some(arg0)
	}

	fn raw_luau_shift_left(&mut self, arg0: i32, arg1: i32) -> i32 {
		let value = arg0.cast_unsigned();
		let count = arg1.cast_unsigned();
		let result = if count < 32 { value << count } else { 0 };

		result.cast_signed()
	}

	fn raw_luau_shift_right_unsigned(&mut self, arg0: i32, arg1: i32) -> i32 {
		let value = arg0.cast_unsigned();
		let count = arg1.cast_unsigned();
		let result = if count < 32 { value >> count } else { 0 };

		result.cast_signed()
	}

	fn raw_luau_shift_right_signed(&mut self, arg0: i32, arg1: i32) -> i32 {
		let count = arg1.cast_unsigned();

		if count < 32 {
			arg0 >> count
		} else {
			arg0 >> 31
		}
	}

	fn raw_luau_shift_fusion(&mut self, arg0: i32, arg1: i32) -> Option<i32> {
		let first = arg0.cast_unsigned();
		let second = arg1.cast_unsigned();
		let both_in_range = first < 32 && second < 32;

		both_in_range.then(|| (first + second).cast_signed())
	}

	fn raw_bit32_count_leading_zeros(&mut self, arg0: i32) -> i32 {
		arg0.cast_unsigned().leading_zeros().cast_signed()
	}

	fn raw_bit32_count_trailing_zeros(&mut self, arg0: i32) -> i32 {
		arg0.cast_unsigned().trailing_zeros().cast_signed()
	}

	fn get_luau_unary_operation(&mut self, arg0: Link) -> Option<(Link, LuauUnaryOperator)> {
		let Node::Foreign(foreign) = self.at(arg0) else {
			return None;
		};
		let (source, operator) = luau::luau_unary_operation(&**foreign)?;

		Some((self.trace(source), operator))
	}

	fn get_luau_arithmetic_operation(
		&mut self,
		arg0: Link,
	) -> Option<(Link, Link, LuauArithmeticOperator)> {
		let Node::Foreign(foreign) = self.at(arg0) else {
			return None;
		};
		let (lhs, rhs, operator) = luau::luau_arithmetic_operation(&**foreign)?;

		Some((self.trace(lhs), self.trace(rhs), operator))
	}

	fn constant_luau_number(&mut self, arg0: Link) -> Option<f64> {
		if let &Node::I32(value) = self.at(arg0) {
			Some(f64::from(value.cast_unsigned()))
		} else if let &Node::F64(value) = self.at(arg0) {
			Some(value)
		} else {
			None
		}
	}

	#[expect(
		clippy::cast_possible_truncation,
		clippy::cast_sign_loss,
		reason = "the guard restricts the value to the exact, lossless u32 range"
	)]
	fn raw_luau_number_result(&mut self, arg0: f64) -> Option<Link> {
		let is_exact_integer = arg0.fract() == 0.0_f64;
		let is_in_word_range = (0.0_f64..=4_294_967_295.0_f64).contains(&arg0);
		let is_negative_zero = arg0 == 0.0_f64 && arg0.is_sign_negative();

		if !arg0.is_finite() {
			None
		} else if is_exact_integer && is_in_word_range && !is_negative_zero {
			Some(self.add_i32((arg0 as u32).cast_signed()))
		} else {
			Some(self.add_f64(arg0))
		}
	}

	fn raw_luau_add(&mut self, arg0: f64, arg1: f64) -> f64 {
		arg0 + arg1
	}

	fn raw_luau_subtract(&mut self, arg0: f64, arg1: f64) -> f64 {
		arg0 - arg1
	}

	fn raw_luau_multiply(&mut self, arg0: f64, arg1: f64) -> f64 {
		arg0 * arg1
	}

	fn raw_luau_divide(&mut self, arg0: f64, arg1: f64) -> f64 {
		arg0 / arg1
	}

	fn raw_luau_floor_divide(&mut self, arg0: f64, arg1: f64) -> f64 {
		(arg0 / arg1).floor()
	}

	// Luau `%` takes the sign of the divisor, not the dividend.
	fn raw_luau_modulo(&mut self, arg0: f64, arg1: f64) -> f64 {
		let remainder = arg0 % arg1;
		let follows_wrong_sign = (remainder < 0.0_f64) != (arg1 < 0.0_f64);

		if remainder != 0.0_f64 && follows_wrong_sign {
			remainder + arg1
		} else {
			remainder
		}
	}

	fn raw_luau_negate(&mut self, arg0: f64) -> f64 {
		-arg0
	}

	fn get_luau_binary_operation(
		&mut self,
		arg0: Link,
	) -> Option<(Link, Link, LuauBinaryOperator)> {
		let Node::Foreign(foreign) = self.at(arg0) else {
			return None;
		};
		let (lhs, rhs, operator) = luau::luau_binary_operation(&**foreign)?;

		Some((self.trace(lhs), self.trace(rhs), operator))
	}

	fn get_luau_compare_operation(
		&mut self,
		arg0: Link,
	) -> Option<(Link, Link, LuauCompareOperator)> {
		let Node::Foreign(foreign) = self.at(arg0) else {
			return None;
		};
		let (lhs, rhs, operator) = luau::luau_compare_operation(&**foreign)?;

		Some((self.trace(lhs), self.trace(rhs), operator))
	}

	fn get_boolean_to_integer(&mut self, arg0: Link) -> Option<Link> {
		let Node::Foreign(foreign) = self.at(arg0) else {
			return None;
		};
		let source = luau::boolean_to_integer(&**foreign)?;

		Some(self.trace(source))
	}

	#[expect(
		clippy::float_cmp,
		reason = "Luau equality is an exact bit comparison, not an epsilon test"
	)]
	fn raw_luau_equal(&mut self, arg0: f64, arg1: f64) -> i32 {
		i32::from(arg0 == arg1)
	}

	#[expect(
		clippy::float_cmp,
		reason = "Luau inequality is an exact bit comparison, not an epsilon test"
	)]
	fn raw_luau_not_equal(&mut self, arg0: f64, arg1: f64) -> i32 {
		i32::from(arg0 != arg1)
	}

	fn raw_luau_less_than(&mut self, arg0: f64, arg1: f64) -> i32 {
		i32::from(arg0 < arg1)
	}

	fn raw_luau_less_than_equal(&mut self, arg0: f64, arg1: f64) -> i32 {
		i32::from(arg0 <= arg1)
	}

	fn raw_math_absolute(&mut self, arg0: f64) -> f64 {
		arg0.abs()
	}

	fn raw_math_square_root(&mut self, arg0: f64) -> f64 {
		arg0.sqrt()
	}

	fn raw_math_floor(&mut self, arg0: f64) -> f64 {
		arg0.floor()
	}

	fn raw_math_ceil(&mut self, arg0: f64) -> f64 {
		arg0.ceil()
	}

	fn raw_math_modf(&mut self, arg0: f64) -> f64 {
		arg0.trunc()
	}

	fn raw_luau_minimum(&mut self, arg0: f64, arg1: f64) -> f64 {
		if arg1 < arg0 { arg1 } else { arg0 }
	}

	fn raw_luau_maximum(&mut self, arg0: f64, arg1: f64) -> f64 {
		if arg1 > arg0 { arg1 } else { arg0 }
	}

	fn raw_math_fmod(&mut self, arg0: f64, arg1: f64) -> f64 {
		arg0 % arg1
	}

	fn get_from_bits_i64(&mut self, arg0: Link) -> Option<Link> {
		let Node::Foreign(foreign) = self.at(arg0) else {
			return None;
		};
		let source = luau::from_bits_i64(&**foreign)?;

		Some(self.trace(source))
	}

	fn get_into_bits_i64(&mut self, arg0: Link) -> Option<(Link, Link)> {
		let Node::Foreign(foreign) = self.at(arg0) else {
			return None;
		};
		let (lhs, rhs) = luau::into_bits_i64(&**foreign)?;

		Some((self.trace(lhs), self.trace(rhs)))
	}

	fn get_repacked_i64(&mut self, arg0: Link) -> Option<Link> {
		let Node::Foreign(packer) = self.at(arg0) else {
			return None;
		};
		let (low, high) = luau::into_bits_i64(&**packer)?;
		let low = self.trace(low);
		let high = self.trace(high);
		let is_same_unpacker =
			low.0 == high.0 && low.1 == FromBitsI64::LOW_PORT && high.1 == FromBitsI64::HIGH_PORT;

		if !is_same_unpacker {
			return None;
		}

		let Node::Foreign(unpacker) = self.at(low) else {
			return None;
		};
		let source = luau::from_bits_i64(&**unpacker)?;

		Some(self.trace(source))
	}

	fn raw_i64_low_word(&mut self, arg0: i64) -> i32 {
		let [byte_0, byte_1, byte_2, byte_3, ..] = arg0.to_le_bytes();

		i32::from_le_bytes([byte_0, byte_1, byte_2, byte_3])
	}

	fn raw_i64_high_word(&mut self, arg0: i64) -> i32 {
		let [.., byte_4, byte_5, byte_6, byte_7] = arg0.to_le_bytes();

		i32::from_le_bytes([byte_4, byte_5, byte_6, byte_7])
	}

	fn raw_i64_from_words(&mut self, arg0: i32, arg1: i32) -> i64 {
		let [byte_0, byte_1, byte_2, byte_3] = arg0.to_le_bytes();
		let [byte_4, byte_5, byte_6, byte_7] = arg1.to_le_bytes();

		i64::from_le_bytes([
			byte_0, byte_1, byte_2, byte_3, byte_4, byte_5, byte_6, byte_7,
		])
	}

	fn split_low(&mut self, arg0: Link) -> Link {
		FromBitsI64::add_into(self.0, arg0).0
	}

	fn flip_split_high(&mut self, arg0: Link) -> Link {
		let high = Link(arg0.0, FromBitsI64::HIGH_PORT);

		Bit32Xor::add_fast_into(self.0, high, 0x8000_0000)
	}
}

const fn is_transmute_width_matched(integer: IntegerType, number: NumberType) -> bool {
	matches!(
		(integer, number),
		(IntegerType::I32, NumberType::F32) | (IntegerType::I64, NumberType::F64)
	)
}

fn shift_count_i64(count: i64) -> u32 {
	u32::try_from((count & 63).cast_unsigned()).unwrap()
}
