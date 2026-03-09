//! Region identity insertion and removal.

use ir_graph::{
	DataFlowGraph, Link, Node,
	control::{RegionOut, ThetaIn, ThetaOut},
	list,
	simple::Identity,
};

fn replace_with_producer(graph: &DataFlowGraph, from: &mut Link) {
	let Node::Identity(Identity { sources }) = graph.get(from.0) else {
		return;
	};

	*from = sources[usize::from(from.1)];
}

// We remove all identities, as they are always redundant.
fn remove_at(graph: &DataFlowGraph, node: &mut Node) {
	node.for_each_mut_argument(|argument| replace_with_producer(graph, argument));
}

/// Removes all identity nodes from the graph.
///
/// # Panics
///
/// Panics if the graph length overflows a `u32`; if this happens, it is a bug.
pub fn remove(graph: &mut DataFlowGraph) {
	let len = graph.len();

	for id in 0..len.try_into().unwrap() {
		let mut node = core::mem::take(graph.get_mut(id));

		remove_at(graph, &mut node);

		*graph.get_mut(id) = node;
	}
}

fn replace_with_identity(graph: &mut DataFlowGraph, from: &mut Link) {
	let sources = list::resizable![*from];
	let identity = Identity::add_into(graph, sources);

	*from = Link(identity, 0);
}

// We insert at...
//   * `RegionOut` arguments, since we need to issue the correct move order.
//   * `ThetaIn` arguments always, since they are mutable and must produce new locals.
//   * `ThetaOut` arguments and condition, since we need to issue the correct move order.
fn insert_at(graph: &mut DataFlowGraph, node: &mut Node) {
	match node {
		Node::RegionOut(RegionOut { results, .. }) => {
			for result in results {
				replace_with_identity(graph, result);
			}
		}
		Node::ThetaIn(ThetaIn { arguments, .. }) => {
			for argument in arguments {
				replace_with_identity(graph, argument);
			}
		}
		Node::ThetaOut(ThetaOut {
			results, condition, ..
		}) => {
			replace_with_identity(graph, condition);

			for result in results {
				replace_with_identity(graph, result);
			}
		}

		Node::Apply(_)
		| Node::F32(_)
		| Node::F64(_)
		| Node::Fence(_)
		| Node::GammaIn(_)
		| Node::GammaOut(_)
		| Node::GlobalGet(_)
		| Node::GlobalNew(_)
		| Node::GlobalSet(_)
		| Node::Host(_)
		| Node::I32(_)
		| Node::I64(_)
		| Node::Identity(_)
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
		| Node::RegionIn(_)
		| Node::TableCopy(_)
		| Node::TableDrop(_)
		| Node::TableFill(_)
		| Node::TableGet(_)
		| Node::TableGrow(_)
		| Node::TableNew(_)
		| Node::TableSet(_)
		| Node::TableSize(_)
		| Node::Trap => {}
	}
}

/// Inserts identity nodes at control flow boundaries.
///
/// # Panics
///
/// Panics if the graph length overflows a `u32`; if this happens, it is a bug.
pub fn insert(graph: &mut DataFlowGraph) {
	let len = graph.len();

	for id in 0..len.try_into().unwrap() {
		let mut node = core::mem::take(graph.get_mut(id));

		insert_at(graph, &mut node);

		*graph.get_mut(id) = node;
	}
}
