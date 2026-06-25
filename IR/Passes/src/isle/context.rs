//! ISLE context implementation for graph regions.

use ir_graph::{
	Link, Node,
	operation::{
		LoadType, Location, MemoryLoad, MemoryStore, MutableGet, MutableNew, MutableSet, StoreType,
		TableGet, TableSet,
		integer::{
			BinaryOperation as IntegerBinaryOperation, BinaryOperator as IntegerBinaryOperator,
			CompareOperation as IntegerCompareOperation, CompareOperator as IntegerCompareOperator,
			Type as IntegerType,
		},
	},
	tracer::identity_source,
};

use super::internal::Context;

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

	fn raw_compare_greater_than_signed_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		i32::from(arg0 > arg1)
	}

	fn raw_compare_greater_than_unsigned_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		i32::from(arg0.cast_unsigned() > arg1.cast_unsigned())
	}

	fn raw_compare_less_than_equal_signed_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		i32::from(arg0 <= arg1)
	}

	fn raw_compare_less_than_equal_unsigned_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		i32::from(arg0.cast_unsigned() <= arg1.cast_unsigned())
	}

	fn raw_compare_greater_than_equal_signed_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		i32::from(arg0 >= arg1)
	}

	fn raw_compare_greater_than_equal_unsigned_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		i32::from(arg0.cast_unsigned() >= arg1.cast_unsigned())
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

	fn raw_compare_greater_than_signed_i64(&mut self, arg0: i64, arg1: i64) -> i32 {
		i32::from(arg0 > arg1)
	}

	fn raw_compare_greater_than_unsigned_i64(&mut self, arg0: i64, arg1: i64) -> i32 {
		i32::from(arg0.cast_unsigned() > arg1.cast_unsigned())
	}

	fn raw_compare_less_than_equal_signed_i64(&mut self, arg0: i64, arg1: i64) -> i32 {
		i32::from(arg0 <= arg1)
	}

	fn raw_compare_less_than_equal_unsigned_i64(&mut self, arg0: i64, arg1: i64) -> i32 {
		i32::from(arg0.cast_unsigned() <= arg1.cast_unsigned())
	}

	fn raw_compare_greater_than_equal_signed_i64(&mut self, arg0: i64, arg1: i64) -> i32 {
		i32::from(arg0 >= arg1)
	}

	fn raw_compare_greater_than_equal_unsigned_i64(&mut self, arg0: i64, arg1: i64) -> i32 {
		i32::from(arg0.cast_unsigned() >= arg1.cast_unsigned())
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
}

fn shift_count_i64(count: i64) -> u32 {
	u32::try_from((count & 63).cast_unsigned()).unwrap()
}
