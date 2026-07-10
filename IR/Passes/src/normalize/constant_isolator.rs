//! Constant isolation.

use core::mem;

use ir_graph::{Link, Node};

#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
const fn clone_constant(node: &Node) -> Option<Node> {
	match node {
		Node::Function(_)
		| Node::Match(_)
		| Node::Repeat(_)
		| Node::FunctionArguments(_)
		| Node::FunctionResults(_)
		| Node::BranchArguments(_)
		| Node::BranchResults(_)
		| Node::RepeatArguments(_)
		| Node::RepeatResults(_)
		| Node::Import(_)
		| Node::Export(_)
		| Node::Foreign(_)
		| Node::Trap
		| Node::Identity(_)
		| Node::Fence(_)
		| Node::Apply(_)
		| Node::RefIsNull(_)
		| Node::IntegerUnaryOperation(_)
		| Node::IntegerBinaryOperation(_)
		| Node::IntegerCompareOperation(_)
		| Node::IntegerNarrow(_)
		| Node::IntegerWiden(_)
		| Node::IntegerSignExtend(_)
		| Node::IntegerConvertToNumber(_)
		| Node::IntegerTransmuteToNumber(_)
		| Node::NumberUnaryOperation(_)
		| Node::NumberBinaryOperation(_)
		| Node::NumberCompareOperation(_)
		| Node::NumberNarrow(_)
		| Node::NumberWiden(_)
		| Node::NumberTruncateToInteger(_)
		| Node::NumberTransmuteToInteger(_)
		| Node::MutableNew(_)
		| Node::MutableGet(_)
		| Node::MutableSet(_)
		| Node::Aggregate(_)
		| Node::Extract(_)
		| Node::TableNew(_)
		| Node::TableGet(_)
		| Node::TableSet(_)
		| Node::TableSize(_)
		| Node::TableGrow(_)
		| Node::TableFill(_)
		| Node::TableCopy(_)
		| Node::TableDrop(_)
		| Node::MemoryNew(_)
		| Node::MemoryLoad(_)
		| Node::MemoryStore(_)
		| Node::MemoryFill(_)
		| Node::MemoryCopy(_)
		| Node::MemoryDrop(_) => None,

		Node::Null => Some(Node::Null),
		Node::I32(value) => Some(Node::I32(*value)),
		Node::I64(value) => Some(Node::I64(*value)),
		Node::F32(value) => Some(Node::F32(*value)),
		Node::F64(value) => Some(Node::F64(*value)),
	}
}

/// Gives every consumer of a constant its own private copy.
pub struct ConstantIsolator {
	claimed: Vec<bool>,
}

impl ConstantIsolator {
	/// Creates a new constant isolator.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			claimed: Vec::new(),
		}
	}

	fn isolate(&mut self, nodes: &mut Vec<Node>, link: &mut Link) {
		let index = usize::try_from(link.0).unwrap();

		let Some(claimed) = self.claimed.get_mut(index) else {
			return;
		};

		let Some(copy) = clone_constant(&nodes[index]) else {
			return;
		};

		if *claimed {
			let Ok(id) = u32::try_from(nodes.len()) else {
				unreachable!()
			};

			nodes.push(copy);

			*link = Link(id, 0);
		} else {
			*claimed = true;
		}
	}

	/// Runs constant isolation on the region.
	pub fn run(&mut self, nodes: &mut Vec<Node>) {
		self.claimed.clear();
		self.claimed.resize(nodes.len(), false);

		for index in 0..nodes.len() {
			let mut node = mem::take(&mut nodes[index]);

			node.for_each_mut_outer(|link| self.isolate(nodes, link));

			nodes[index] = node;
		}
	}
}

impl Default for ConstantIsolator {
	fn default() -> Self {
		Self::new()
	}
}
