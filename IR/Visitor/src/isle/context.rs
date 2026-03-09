use ir_graph::{
	DataFlowGraph, Link, Node,
	control::{GammaIn, RegionIn},
	simple::{
		GlobalGet, GlobalNew, GlobalSet, Identity, IntegerBinaryOperation, IntegerBinaryOperator,
		IntegerType, LoadType, Location, MemoryLoad, MemoryStore, StoreType, TableGet, TableSet,
	},
};

use super::internal::Context;

fn get_next_producer(node: &Node, port: u16) -> Option<Link> {
	let index = usize::from(port);
	let producer = match node {
		Node::RegionIn(RegionIn { input, .. }) => Link(*input, port),
		Node::GammaIn(GammaIn { arguments, .. }) => arguments.get(index).copied()?,
		Node::Identity(Identity { sources }) => sources.get(index).copied()?,

		Node::Apply(_)
		| Node::F32(_)
		| Node::F64(_)
		| Node::Fence(_)
		| Node::GammaOut(_)
		| Node::GlobalGet(_)
		| Node::GlobalNew(_)
		| Node::GlobalSet(_)
		| Node::Host(_)
		| Node::I32(_)
		| Node::I64(_)
		| Node::Import(_)
		| Node::IntegerBinaryOperation(_)
		| Node::IntegerCompareOperation(_)
		| Node::IntegerConvertToNumber(_)
		| Node::IntegerExtend(_)
		| Node::IntegerNarrow(_)
		| Node::IntegerTransmuteToNumber(_)
		| Node::IntegerUnaryOperation(_)
		| Node::IntegerWiden(_)
		| Node::LambdaIn(_)
		| Node::LambdaOut(_)
		| Node::MemoryCopy(_)
		| Node::MemoryDrop(_)
		| Node::MemoryFill(_)
		| Node::MemoryGrow(_)
		| Node::MemoryLoad(_)
		| Node::MemoryNew(_)
		| Node::MemorySize(_)
		| Node::MemoryStore(_)
		| Node::Null
		| Node::NumberBinaryOperation(_)
		| Node::NumberCompareOperation(_)
		| Node::NumberNarrow(_)
		| Node::NumberTransmuteToInteger(_)
		| Node::NumberTruncateToInteger(_)
		| Node::NumberUnaryOperation(_)
		| Node::NumberWiden(_)
		| Node::OmegaIn(_)
		| Node::OmegaOut(_)
		| Node::RefIsNull(_)
		| Node::RegionOut(_)
		| Node::TableCopy(_)
		| Node::TableDrop(_)
		| Node::TableFill(_)
		| Node::TableGet(_)
		| Node::TableGrow(_)
		| Node::TableNew(_)
		| Node::TableSet(_)
		| Node::TableSize(_)
		| Node::ThetaIn(_)
		| Node::ThetaOut(_)
		| Node::Trap => return None,
	};

	Some(producer)
}

fn find_first_producer(graph: &DataFlowGraph, mut source: Link) -> Link {
	while let Some(next) = {
		let Link(id, port) = source;

		get_next_producer(graph.get(id), port)
	} {
		source = next;
	}

	source
}

impl Context for DataFlowGraph {
	fn get_i32(&mut self, arg0: Link) -> Option<i32> {
		if let Node::I32(value) = *self.get(arg0.0) {
			Some(value)
		} else {
			None
		}
	}

	fn add_i32(&mut self, arg0: i32) -> Link {
		Node::add_i32_into(self, arg0)
	}

	fn get_i64(&mut self, arg0: Link) -> Option<i64> {
		if let Node::I64(value) = *self.get(arg0.0) {
			Some(value)
		} else {
			None
		}
	}

	fn add_i64(&mut self, arg0: i64) -> Link {
		Node::add_i64_into(self, arg0)
	}

	fn get_integer_binary_operation(
		&mut self,
		arg0: Link,
	) -> Option<(Link, Link, IntegerType, IntegerBinaryOperator)> {
		if let Node::IntegerBinaryOperation(IntegerBinaryOperation {
			lhs,
			rhs,
			kind,
			operator,
		}) = *self.get(arg0.0)
		{
			let lhs = find_first_producer(self, lhs);
			let rhs = find_first_producer(self, rhs);

			Some((lhs, rhs, kind, operator))
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
		IntegerBinaryOperation::add_into(self, arg0, arg1, *arg2, *arg3)
	}

	fn raw_add_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		arg0.wrapping_add(arg1)
	}

	fn raw_sub_i32(&mut self, arg0: i32, arg1: i32) -> i32 {
		arg0.wrapping_sub(arg1)
	}

	fn get_f32(&mut self, arg0: Link) -> Option<f32> {
		if let Node::F32(value) = *self.get(arg0.0) {
			Some(value)
		} else {
			None
		}
	}

	fn add_f32(&mut self, arg0: f32) -> Link {
		Node::add_f32_into(self, arg0)
	}

	fn get_f64(&mut self, arg0: Link) -> Option<f64> {
		if let Node::F64(value) = *self.get(arg0.0) {
			Some(value)
		} else {
			None
		}
	}

	fn add_f64(&mut self, arg0: f64) -> Link {
		Node::add_f64_into(self, arg0)
	}

	fn get_global_new(&mut self, arg0: Link) -> Option<Link> {
		if let Node::GlobalNew(GlobalNew { initializer }) = *self.get(arg0.0) {
			let initializer = find_first_producer(self, initializer);

			Some(initializer)
		} else {
			None
		}
	}

	fn get_global_get(&mut self, arg0: Link) -> Option<Link> {
		if let Node::GlobalGet(GlobalGet { source }) = *self.get(arg0.0) {
			let source = find_first_producer(self, source);

			Some(source)
		} else {
			None
		}
	}

	fn get_global_set(&mut self, arg0: Link) -> Option<(Link, Link)> {
		if let Node::GlobalSet(GlobalSet {
			destination,
			source,
		}) = *self.get(arg0.0)
		{
			let destination = find_first_producer(self, destination);
			let source = find_first_producer(self, source);

			Some((destination, source))
		} else {
			None
		}
	}

	fn get_table_get(&mut self, arg0: Link) -> Option<(Link, Link)> {
		if let Node::TableGet(TableGet {
			source: Location { reference, offset },
		}) = *self.get(arg0.0)
		{
			let reference = find_first_producer(self, reference);
			let offset = find_first_producer(self, offset);

			Some((reference, offset))
		} else {
			None
		}
	}

	fn get_table_set(&mut self, arg0: Link) -> Option<(Link, Link, Link)> {
		if let Node::TableSet(TableSet {
			destination: Location { reference, offset },
			source,
		}) = *self.get(arg0.0)
		{
			let reference = find_first_producer(self, reference);
			let offset = find_first_producer(self, offset);
			let source = find_first_producer(self, source);

			Some((reference, offset, source))
		} else {
			None
		}
	}

	fn get_memory_load(&mut self, arg0: Link) -> Option<(Link, Link, LoadType)> {
		if let Node::MemoryLoad(MemoryLoad {
			source: Location { reference, offset },
			kind,
		}) = *self.get(arg0.0)
		{
			let reference = find_first_producer(self, reference);
			let offset = find_first_producer(self, offset);

			Some((reference, offset, kind))
		} else {
			None
		}
	}

	fn get_memory_store(&mut self, arg0: Link) -> Option<(Link, Link, Link, StoreType)> {
		if let Node::MemoryStore(MemoryStore {
			destination: Location { reference, offset },
			source,
			kind,
		}) = *self.get(arg0.0)
		{
			let reference = find_first_producer(self, reference);
			let offset = find_first_producer(self, offset);
			let source = find_first_producer(self, source);

			Some((reference, offset, source, kind))
		} else {
			None
		}
	}
}
