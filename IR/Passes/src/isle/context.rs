//! ISLE context implementation for graph regions.

use ir_graph::{
	Link, Node,
	operation::{
		Identity, LoadType, Location, MemoryLoad, MemoryStore, MutableGet, MutableNew, MutableSet,
		StoreType, TableGet, TableSet,
		integer::{
			BinaryOperation as IntegerBinaryOperation, BinaryOperator as IntegerBinaryOperator,
			Type as IntegerType,
		},
	},
};

use super::internal::Context;

fn skip_identities(nodes: &[Node], mut link: Link) -> Link {
	while let Node::Identity(Identity { sources }) = &nodes[usize::try_from(link.0).unwrap()] {
		if let Some(&next) = sources.get(usize::from(link.1)) {
			link = next;
		} else {
			break;
		}
	}

	link
}

/// A newtype wrapper for implementing the ISLE `Context` trait on a region.
pub struct RegionContext<'nodes>(pub &'nodes mut Vec<Node>);

impl RegionContext<'_> {
	fn at(&self, link: Link) -> &Node {
		&self.0[usize::try_from(link.0).unwrap()]
	}

	fn trace(&self, link: Link) -> Link {
		skip_identities(self.0, link)
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

	fn raw_add_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		arg0.wrapping_add(arg1)
	}

	fn raw_sub_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		arg0.wrapping_sub(arg1)
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
