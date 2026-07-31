//! ISLE context implementation for graph regions.

use ir_graph::{
	Link, Node,
	operation::{
		Aggregate, ExtendType, Extract, IntegerNarrow, IntegerSignExtend, IntegerTransmuteToNumber,
		IntegerWiden, LoadType, Location, MemoryLoad, MemoryStore, MutableGet, MutableNew,
		MutableSet, NumberTransmuteToInteger, RefIsNull, StoreType, TableGet, TableSet,
		integer::{
			BinaryOperation as IntegerBinaryOperation, BinaryOperator as IntegerBinaryOperator,
			CompareOperation as IntegerCompareOperation, CompareOperator as IntegerCompareOperator,
			Type as IntegerType, UnaryOperation as IntegerUnaryOperation,
			UnaryOperator as IntegerUnaryOperator,
		},
		number::{
			CompareOperation as NumberCompareOperation, CompareOperator as NumberCompareOperator,
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

use crate::catalog::{Optimization, Optimizations};

use super::{
	internal::Context,
	luau::{
		self, Bit32BinaryOperator, Bit32UnaryOperator, LuauArithmeticOperator, LuauBinaryOperator,
		LuauCompareOperator, LuauUnaryOperator,
	},
};

/// Supply a graph region to ISLE.
pub struct RegionContext<'context> {
	/// Store the region nodes being rewritten.
	pub nodes: &'context mut Vec<Node>,
	/// Select the enabled rewrites.
	pub optimizations: &'context Optimizations,
}

impl RegionContext<'_> {
	fn at(&self, link: Link) -> &Node {
		&self.nodes[usize::try_from(link.0).unwrap()]
	}

	fn trace(&self, link: Link) -> Link {
		identity_source(self.nodes, link)
	}
}

#[expect(
	clippy::renamed_function_params,
	reason = "semantic names replace generated positional Context parameter names"
)]
impl Context for RegionContext<'_> {
	fn optimization_enabled(&mut self, optimization: &Optimization) -> bool {
		self.optimizations.is_enabled(*optimization)
	}

	fn i32_unary_folding_enabled(&mut self, operator: &IntegerUnaryOperator) -> bool {
		self.optimizations
			.is_enabled(Optimization::for_integer_unary_folding(
				IntegerType::I32,
				*operator,
			))
	}

	fn i64_unary_folding_enabled(&mut self, operator: &IntegerUnaryOperator) -> bool {
		self.optimizations
			.is_enabled(Optimization::for_integer_unary_folding(
				IntegerType::I64,
				*operator,
			))
	}

	fn i32_binary_folding_enabled(&mut self, operator: &IntegerBinaryOperator) -> bool {
		self.optimizations
			.is_enabled(Optimization::for_integer_binary_folding(
				IntegerType::I32,
				*operator,
			))
	}

	fn i64_binary_folding_enabled(&mut self, operator: &IntegerBinaryOperator) -> bool {
		self.optimizations
			.is_enabled(Optimization::for_integer_binary_folding(
				IntegerType::I64,
				*operator,
			))
	}

	fn i32_comparison_folding_enabled(&mut self, operator: &IntegerCompareOperator) -> bool {
		self.optimizations
			.is_enabled(Optimization::for_integer_comparison_folding(
				IntegerType::I32,
				*operator,
			))
	}

	fn i64_comparison_folding_enabled(&mut self, operator: &IntegerCompareOperator) -> bool {
		self.optimizations
			.is_enabled(Optimization::for_integer_comparison_folding(
				IntegerType::I64,
				*operator,
			))
	}

	fn get_exact_boolean(&mut self, arg0: Link) -> Option<Link> {
		let boolean = self.trace(arg0);

		matches!(
			self.at(boolean),
			Node::IntegerCompareOperation(_) | Node::NumberCompareOperation(_) | Node::RefIsNull(_)
		)
		.then_some(boolean)
	}

	fn get_boolean_not(&mut self, arg0: Link) -> Option<Link> {
		let boolean_not = self.trace(arg0);
		let &Node::IntegerCompareOperation(operation) = self.at(boolean_not) else {
			return None;
		};
		let right = self.trace(operation.rhs);

		if operation.kind != IntegerType::I32
			|| operation.operator != IntegerCompareOperator::Equal
			|| !matches!(self.at(right), Node::I32(0_i32))
		{
			return None;
		}

		self.get_exact_boolean(operation.lhs)
	}

	fn get_i32(&mut self, arg0: Link) -> Option<i32> {
		if let &Node::I32(value) = self.at(arg0) {
			Some(value)
		} else {
			None
		}
	}

	fn add_i32(&mut self, arg0: i32) -> Link {
		Node::add_i32_into(self.nodes, arg0)
	}

	fn get_i64(&mut self, arg0: Link) -> Option<i64> {
		if let &Node::I64(value) = self.at(arg0) {
			Some(value)
		} else {
			None
		}
	}

	fn add_i64(&mut self, arg0: i64) -> Link {
		Node::add_i64_into(self.nodes, arg0)
	}

	fn widen_i32_value(&mut self, arg0: i32) -> i64 {
		i64::from(arg0)
	}

	fn get_integer_unary_operation(
		&mut self,
		arg0: Link,
	) -> Option<(Link, IntegerType, IntegerUnaryOperator)> {
		if let &Node::IntegerUnaryOperation(IntegerUnaryOperation {
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

	fn evaluate_i32_unary(&mut self, operator: &IntegerUnaryOperator, source: i32) -> i32 {
		match *operator {
			IntegerUnaryOperator::CountOnes => source.count_ones().cast_signed(),
			IntegerUnaryOperator::LeadingZeros => source.leading_zeros().cast_signed(),
			IntegerUnaryOperator::TrailingZeros => source.trailing_zeros().cast_signed(),
		}
	}

	fn evaluate_i64_unary(&mut self, operator: &IntegerUnaryOperator, source: i64) -> i64 {
		match *operator {
			IntegerUnaryOperator::CountOnes => i64::from(source.count_ones()),
			IntegerUnaryOperator::LeadingZeros => i64::from(source.leading_zeros()),
			IntegerUnaryOperator::TrailingZeros => i64::from(source.trailing_zeros()),
		}
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
		IntegerBinaryOperation::add_into(self.nodes, arg0, arg1, *arg2, *arg3)
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

	fn add_integer_compare_operation(
		&mut self,
		arg0: Link,
		arg1: Link,
		arg2: &IntegerType,
		arg3: &IntegerCompareOperator,
	) -> Link {
		IntegerCompareOperation::add_into(self.nodes, arg0, arg1, *arg2, *arg3)
	}

	fn evaluate_i32_binary(
		&mut self,
		operator: &IntegerBinaryOperator,
		left: i32,
		right: i32,
	) -> Option<i32> {
		match *operator {
			IntegerBinaryOperator::Add => Some(left.wrapping_add(right)),
			IntegerBinaryOperator::Subtract => Some(left.wrapping_sub(right)),
			IntegerBinaryOperator::Multiply => Some(left.wrapping_mul(right)),
			IntegerBinaryOperator::Divide { is_signed: true } => left.checked_div(right),
			IntegerBinaryOperator::Divide { is_signed: false } => left
				.cast_unsigned()
				.checked_div(right.cast_unsigned())
				.map(u32::cast_signed),
			IntegerBinaryOperator::Remainder { is_signed: true } => left.checked_rem(right),
			IntegerBinaryOperator::Remainder { is_signed: false } => left
				.cast_unsigned()
				.checked_rem(right.cast_unsigned())
				.map(u32::cast_signed),
			IntegerBinaryOperator::And => Some(left & right),
			IntegerBinaryOperator::Or => Some(left | right),
			IntegerBinaryOperator::ExclusiveOr => Some(left ^ right),
			IntegerBinaryOperator::ShiftLeft => Some(left.wrapping_shl(right.cast_unsigned())),
			IntegerBinaryOperator::ShiftRight { is_signed: true } => {
				Some(left.wrapping_shr(right.cast_unsigned()))
			}
			IntegerBinaryOperator::ShiftRight { is_signed: false } => Some(
				left.cast_unsigned()
					.wrapping_shr(right.cast_unsigned())
					.cast_signed(),
			),
			IntegerBinaryOperator::RotateLeft => Some(left.rotate_left(right.cast_unsigned())),
			IntegerBinaryOperator::RotateRight => Some(left.rotate_right(right.cast_unsigned())),
		}
	}

	fn evaluate_i64_binary(
		&mut self,
		operator: &IntegerBinaryOperator,
		left: i64,
		right: i64,
	) -> Option<i64> {
		match *operator {
			IntegerBinaryOperator::Add => Some(left.wrapping_add(right)),
			IntegerBinaryOperator::Subtract => Some(left.wrapping_sub(right)),
			IntegerBinaryOperator::Multiply => Some(left.wrapping_mul(right)),
			IntegerBinaryOperator::Divide { is_signed: true } => left.checked_div(right),
			IntegerBinaryOperator::Divide { is_signed: false } => left
				.cast_unsigned()
				.checked_div(right.cast_unsigned())
				.map(u64::cast_signed),
			IntegerBinaryOperator::Remainder { is_signed: true } => left.checked_rem(right),
			IntegerBinaryOperator::Remainder { is_signed: false } => left
				.cast_unsigned()
				.checked_rem(right.cast_unsigned())
				.map(u64::cast_signed),
			IntegerBinaryOperator::And => Some(left & right),
			IntegerBinaryOperator::Or => Some(left | right),
			IntegerBinaryOperator::ExclusiveOr => Some(left ^ right),
			IntegerBinaryOperator::ShiftLeft => Some(left.wrapping_shl(shift_count_i64(right))),
			IntegerBinaryOperator::ShiftRight { is_signed: true } => {
				Some(left.wrapping_shr(shift_count_i64(right)))
			}
			IntegerBinaryOperator::ShiftRight { is_signed: false } => Some(
				left.cast_unsigned()
					.wrapping_shr(shift_count_i64(right))
					.cast_signed(),
			),
			IntegerBinaryOperator::RotateLeft => Some(left.rotate_left(shift_count_i64(right))),
			IntegerBinaryOperator::RotateRight => Some(left.rotate_right(shift_count_i64(right))),
		}
	}

	fn evaluate_i32_comparison(
		&mut self,
		operator: &IntegerCompareOperator,
		left: i32,
		right: i32,
	) -> i32 {
		i32::from(match *operator {
			IntegerCompareOperator::Equal => left == right,
			IntegerCompareOperator::NotEqual => left != right,
			IntegerCompareOperator::LessThan { is_signed: true } => left < right,
			IntegerCompareOperator::LessThan { is_signed: false } => {
				left.cast_unsigned() < right.cast_unsigned()
			}
			IntegerCompareOperator::LessThanEqual { is_signed: true } => left <= right,
			IntegerCompareOperator::LessThanEqual { is_signed: false } => {
				left.cast_unsigned() <= right.cast_unsigned()
			}
		})
	}

	fn evaluate_i64_comparison(
		&mut self,
		operator: &IntegerCompareOperator,
		left: i64,
		right: i64,
	) -> i32 {
		i32::from(match *operator {
			IntegerCompareOperator::Equal => left == right,
			IntegerCompareOperator::NotEqual => left != right,
			IntegerCompareOperator::LessThan { is_signed: true } => left < right,
			IntegerCompareOperator::LessThan { is_signed: false } => {
				left.cast_unsigned() < right.cast_unsigned()
			}
			IntegerCompareOperator::LessThanEqual { is_signed: true } => left <= right,
			IntegerCompareOperator::LessThanEqual { is_signed: false } => {
				left.cast_unsigned() <= right.cast_unsigned()
			}
		})
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
		IntegerWiden::add_into(self.nodes, arg0)
	}

	fn get_integer_sign_extend(&mut self, arg0: Link) -> Option<(Link, ExtendType)> {
		if let &Node::IntegerSignExtend(IntegerSignExtend { source, kind }) = self.at(arg0) {
			Some((self.trace(source), kind))
		} else {
			None
		}
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
		NumberUnaryOperation::add_into(self.nodes, arg0, *arg1, *arg2)
	}

	fn get_number_compare_operation(
		&mut self,
		arg0: Link,
	) -> Option<(Link, Link, NumberType, NumberCompareOperator)> {
		if let &Node::NumberCompareOperation(NumberCompareOperation {
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

	fn add_number_compare_operation(
		&mut self,
		arg0: Link,
		arg1: Link,
		arg2: &NumberType,
		arg3: &NumberCompareOperator,
	) -> Link {
		NumberCompareOperation::add_into(self.nodes, arg0, arg1, *arg2, *arg3)
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
			Bit32BinaryOperator::And => Bit32And::add_into(self.nodes, arg0, arg1),
			Bit32BinaryOperator::Or => Bit32Or::add_into(self.nodes, arg0, arg1),
			Bit32BinaryOperator::ExclusiveOr => Bit32Xor::add_into(self.nodes, arg0, arg1),
			Bit32BinaryOperator::ShiftLeft => Bit32LShift::add_into(self.nodes, arg0, arg1),
			Bit32BinaryOperator::ShiftRightUnsigned => {
				Bit32RShift::add_into(self.nodes, arg0, arg1)
			}
			Bit32BinaryOperator::ShiftRightSigned => Bit32ArShift::add_into(self.nodes, arg0, arg1),
			Bit32BinaryOperator::RotateLeft => Bit32LRotate::add_into(self.nodes, arg0, arg1),
			Bit32BinaryOperator::RotateRight => Bit32RRotate::add_into(self.nodes, arg0, arg1),
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

	fn fuse_bit32_binary(
		&mut self,
		operator: &Bit32BinaryOperator,
		first: i32,
		second: i32,
	) -> Option<i32> {
		match *operator {
			Bit32BinaryOperator::And => Some(first & second),
			Bit32BinaryOperator::Or => Some(first | second),
			Bit32BinaryOperator::ExclusiveOr => Some(first ^ second),
			Bit32BinaryOperator::ShiftLeft
			| Bit32BinaryOperator::ShiftRightUnsigned
			| Bit32BinaryOperator::ShiftRightSigned
			| Bit32BinaryOperator::RotateLeft
			| Bit32BinaryOperator::RotateRight => {
				let first = first.cast_unsigned();
				let second = second.cast_unsigned();

				(first < 32 && second < 32).then(|| (first + second).cast_signed())
			}
		}
	}

	fn evaluate_bit32_binary(
		&mut self,
		operator: &Bit32BinaryOperator,
		left: i32,
		right: i32,
	) -> i32 {
		match *operator {
			Bit32BinaryOperator::And => left & right,
			Bit32BinaryOperator::Or => left | right,
			Bit32BinaryOperator::ExclusiveOr => left ^ right,
			Bit32BinaryOperator::ShiftLeft => {
				let count = right.cast_unsigned();

				if count < 32 {
					left.cast_unsigned().wrapping_shl(count).cast_signed()
				} else {
					0
				}
			}
			Bit32BinaryOperator::ShiftRightUnsigned => {
				let count = right.cast_unsigned();

				if count < 32 {
					left.cast_unsigned().wrapping_shr(count).cast_signed()
				} else {
					0
				}
			}
			Bit32BinaryOperator::ShiftRightSigned => left >> right.cast_unsigned().min(31),
			Bit32BinaryOperator::RotateLeft => left.rotate_left(right.cast_unsigned()),
			Bit32BinaryOperator::RotateRight => left.rotate_right(right.cast_unsigned()),
		}
	}

	fn evaluate_bit32_unary(&mut self, operator: &Bit32UnaryOperator, source: i32) -> i32 {
		match *operator {
			Bit32UnaryOperator::CountLeadingZeros => {
				source.cast_unsigned().leading_zeros().cast_signed()
			}
			Bit32UnaryOperator::CountTrailingZeros => {
				source.cast_unsigned().trailing_zeros().cast_signed()
			}
		}
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
			Some(Node::add_f64_into(self.nodes, arg0))
		}
	}

	fn evaluate_luau_arithmetic(
		&mut self,
		operator: &LuauArithmeticOperator,
		left: f64,
		right: f64,
	) -> f64 {
		match *operator {
			LuauArithmeticOperator::Add => left + right,
			LuauArithmeticOperator::Subtract => left - right,
			LuauArithmeticOperator::Multiply => left * right,
			LuauArithmeticOperator::Divide => left / right,
			LuauArithmeticOperator::FloorDivide => (left / right).floor(),
			LuauArithmeticOperator::Modulo => {
				let remainder = left % right;
				let follows_wrong_sign = (remainder < 0.0_f64) != (right < 0.0_f64);

				if remainder != 0.0_f64 && follows_wrong_sign {
					remainder + right
				} else {
					remainder
				}
			}
		}
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
		reason = "Luau equality is exact rather than approximate"
	)]
	fn evaluate_luau_comparison(
		&mut self,
		operator: &LuauCompareOperator,
		left: f64,
		right: f64,
	) -> i32 {
		i32::from(match *operator {
			LuauCompareOperator::Equal => left == right,
			LuauCompareOperator::NotEqual => left != right,
			LuauCompareOperator::LessThan => left < right,
			LuauCompareOperator::LessThanEqual => left <= right,
		})
	}

	fn evaluate_luau_unary(&mut self, operator: &LuauUnaryOperator, source: f64) -> Option<f64> {
		match *operator {
			LuauUnaryOperator::Negate => Some(-source),
			LuauUnaryOperator::Absolute => Some(source.abs()),
			LuauUnaryOperator::SquareRoot => Some(source.sqrt()),
			LuauUnaryOperator::RoundDown => Some(source.floor()),
			LuauUnaryOperator::RoundUp => Some(source.ceil()),
			LuauUnaryOperator::RoundToZero => Some(source.trunc()),
			LuauUnaryOperator::FlipMostSignificant => None,
		}
	}

	fn evaluate_luau_binary(
		&mut self,
		operator: &LuauBinaryOperator,
		left: f64,
		right: f64,
	) -> f64 {
		match *operator {
			LuauBinaryOperator::Minimum => {
				if right < left {
					right
				} else {
					left
				}
			}
			LuauBinaryOperator::Maximum => {
				if right > left {
					right
				} else {
					left
				}
			}
			LuauBinaryOperator::FloatModulo => left % right,
		}
	}

	fn get_integral_luau_unary(&mut self, source: Link) -> Option<Link> {
		let (inner, operator) = self.get_luau_unary_operation(source)?;

		matches!(
			operator,
			LuauUnaryOperator::RoundDown
				| LuauUnaryOperator::RoundUp
				| LuauUnaryOperator::RoundToZero
		)
		.then_some(inner)
	}

	fn get_luau_extremum(&mut self, source: Link) -> Option<(Link, Link)> {
		let (left, right, operator) = self.get_luau_binary_operation(source)?;

		matches!(
			operator,
			LuauBinaryOperator::Minimum | LuauBinaryOperator::Maximum
		)
		.then_some((left, right))
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
		FromBitsI64::add_into(self.nodes, arg0).0
	}

	fn flip_split_high(&mut self, arg0: Link) -> Link {
		let high = Link(arg0.0, FromBitsI64::HIGH_PORT);

		Bit32Xor::add_fast_into(self.nodes, high, 0x8000_0000)
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
