//! The register-reuse hint: which operand's register an output port would like.

use ir_graph::{Link, Node, operation};

/// Returns the operand whose register the core node's port would like to reuse.
#[must_use]
#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
pub fn reuse_hint(node: &Node, port: u16) -> Option<Link> {
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
		| Node::Trap
		| Node::Null
		| Node::I32(_)
		| Node::I64(_)
		| Node::F32(_)
		| Node::F64(_)
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
		| Node::Aggregate(_)
		| Node::Extract(_)
		| Node::TableNew(_)
		| Node::MemoryNew(_)
		| Node::Foreign(_) => None,

		Node::Export(node) => (port == operation::Export::STATE_PORT).then_some(node.value),

		Node::Identity(node) => node.sources.get(usize::from(port)).copied(),
		Node::Fence(node) => node.sources.get(usize::from(port)).copied(),

		Node::MutableGet(node) => {
			(port == operation::MutableGet::STATE_PORT).then_some(node.source)
		}
		Node::MutableSet(node) => {
			(port == operation::MutableSet::STATE_PORT).then_some(node.destination)
		}

		Node::TableGet(node) => {
			(port == operation::TableGet::STATE_PORT).then_some(node.source.reference)
		}
		Node::TableSet(node) => {
			(port == operation::TableSet::STATE_PORT).then_some(node.destination.reference)
		}
		Node::TableSize(node) => (port == operation::TableSize::STATE_PORT).then_some(node.source),
		Node::TableGrow(node) => {
			(port == operation::TableGrow::STATE_PORT).then_some(node.destination)
		}
		Node::TableFill(node) => {
			(port == operation::TableFill::STATE_PORT).then_some(node.destination.reference)
		}
		Node::TableCopy(node) => match port {
			operation::TableCopy::DESTINATION_STATE_PORT => Some(node.destination.reference),
			operation::TableCopy::SOURCE_STATE_PORT => Some(node.source.reference),
			_ => None,
		},
		Node::TableDrop(node) => (port == operation::TableDrop::STATE_PORT).then_some(node.source),

		Node::MemoryLoad(node) => {
			(port == operation::MemoryLoad::STATE_PORT).then_some(node.source.reference)
		}
		Node::MemoryStore(node) => {
			(port == operation::MemoryStore::STATE_PORT).then_some(node.destination.reference)
		}
		Node::MemorySize(node) => {
			(port == operation::MemorySize::STATE_PORT).then_some(node.source)
		}
		Node::MemoryGrow(node) => {
			(port == operation::MemoryGrow::STATE_PORT).then_some(node.destination)
		}
		Node::MemoryFill(node) => {
			(port == operation::MemoryFill::STATE_PORT).then_some(node.destination.reference)
		}
		Node::MemoryCopy(node) => match port {
			operation::MemoryCopy::DESTINATION_STATE_PORT => Some(node.destination.reference),
			operation::MemoryCopy::SOURCE_STATE_PORT => Some(node.source.reference),
			_ => None,
		},
		Node::MemoryDrop(node) => {
			(port == operation::MemoryDrop::STATE_PORT).then_some(node.source)
		}
	}
}
