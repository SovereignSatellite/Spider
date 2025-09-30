use data_flow_graph::{
	DataFlowGraph, Link, Node,
	base::{
		GlobalGet, GlobalNew, GlobalSet, IntegerBinaryOperation, IntegerBinaryOperator,
		IntegerType, Location, TableGet, TableSet,
	},
};

use super::internal::Context;

impl Context for DataFlowGraph {
	fn get_i32(&mut self, source: Link) -> Option<i32> {
		if let Node::I32(value) = *self.get(source.0) {
			Some(value)
		} else {
			None
		}
	}

	fn add_i32(&mut self, value: i32) -> Link {
		self.add_i32(value)
	}

	fn get_i64(&mut self, source: Link) -> Option<i64> {
		if let Node::I64(source) = *self.get(source.0) {
			Some(source)
		} else {
			None
		}
	}

	fn add_i64(&mut self, value: i64) -> Link {
		self.add_i64(value)
	}

	fn get_integer_binary_operation(
		&mut self,
		source: Link,
	) -> Option<(Link, Link, IntegerType, IntegerBinaryOperator)> {
		if let Node::IntegerBinaryOperation(IntegerBinaryOperation {
			lhs,
			rhs,
			r#type,
			operator,
		}) = *self.get(source.0)
		{
			Some((lhs, rhs, r#type, operator))
		} else {
			None
		}
	}

	fn add_integer_binary_operation(
		&mut self,
		lhs: Link,
		rhs: Link,
		r#type: &IntegerType,
		operator: &IntegerBinaryOperator,
	) -> Link {
		self.add_integer_binary_operation(lhs, rhs, *r#type, *operator)
	}

	fn raw_add_i32(&mut self, lhs: i32, rhs: i32) -> i32 {
		lhs.wrapping_add(rhs)
	}

	fn raw_sub_i32(&mut self, lhs: i32, rhs: i32) -> i32 {
		lhs.wrapping_sub(rhs)
	}

	fn get_f32(&mut self, source: Link) -> Option<f32> {
		if let Node::F32(source) = *self.get(source.0) {
			Some(source)
		} else {
			None
		}
	}

	fn add_f32(&mut self, value: f32) -> Link {
		self.add_f32(value)
	}

	fn get_f64(&mut self, source: Link) -> Option<f64> {
		if let Node::F64(source) = *self.get(source.0) {
			Some(source)
		} else {
			None
		}
	}

	fn add_f64(&mut self, value: f64) -> Link {
		self.add_f64(value)
	}

	fn get_table_get(&mut self, source: Link) -> Option<(Link, Link)> {
		if let Node::TableGet(TableGet {
			source: Location { reference, offset },
		}) = *self.get(source.0)
		{
			Some((reference, offset))
		} else {
			None
		}
	}

	fn get_table_set(&mut self, source: Link) -> Option<(Link, Link, Link)> {
		if let Node::TableSet(TableSet {
			destination: Location { reference, offset },
			source,
		}) = *self.get(source.0)
		{
			Some((reference, offset, source))
		} else {
			None
		}
	}

	fn get_global_new(&mut self, source: Link) -> Option<Link> {
		if let Node::GlobalNew(GlobalNew { initializer }) = *self.get(source.0) {
			Some(initializer)
		} else {
			None
		}
	}

	fn get_global_get(&mut self, source: Link) -> Option<Link> {
		if let Node::GlobalGet(GlobalGet { source }) = *self.get(source.0) {
			Some(source)
		} else {
			None
		}
	}

	fn get_global_set(&mut self, source: Link) -> Option<(Link, Link)> {
		if let Node::GlobalSet(GlobalSet {
			destination,
			source,
		}) = *self.get(source.0)
		{
			Some((destination, source))
		} else {
			None
		}
	}
}
