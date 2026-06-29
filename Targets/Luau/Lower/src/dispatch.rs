//! Reverse-order region sweep that expands trivial nodes in place.

use core::mem;

use ir_graph::{
	Link, Node, Region,
	operation::{integer, number},
};

use crate::{
	convert, f32 as lower_f32, f64 as lower_f64, i32 as lower_i32, i64 as lower_i64, memory,
	replace, table, truncate,
};

/// Lowers every trivial node in the region, reporting whether anything changed.
pub fn apply(region: &mut Region) -> bool {
	let nodes = region.nodes_mut();

	let Ok(length) = u32::try_from(nodes.len()) else {
		unreachable!()
	};

	let mut changed = false;

	for id in (0..length).rev() {
		changed |= memory::lower(nodes, id) || table::lower(nodes, id) || lower_node(nodes, id);
	}

	changed
}

// The node is taken out so its (Copy) payload can be read while the lowering
// appends to `nodes`; a node that does not lower is put straight back.
#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
fn lower_node(nodes: &mut Vec<Node>, id: u32) -> bool {
	let index = usize::try_from(id).unwrap();
	let node = mem::take(&mut nodes[index]);

	let result = match &node {
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
		| Node::Null
		| Node::I32(_)
		| Node::I64(_)
		| Node::F32(_)
		| Node::F64(_)
		| Node::Identity(_)
		| Node::Fence(_)
		| Node::Apply(_)
		| Node::RefIsNull(_)
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
		| Node::MemorySize(_)
		| Node::MemoryGrow(_)
		| Node::MemoryFill(_)
		| Node::MemoryCopy(_)
		| Node::MemoryDrop(_) => None,

		Node::IntegerUnaryOperation(operation) => Some(lower_integer_unary(nodes, *operation)),
		Node::IntegerBinaryOperation(operation) => lower_integer_binary(nodes, *operation),
		Node::IntegerCompareOperation(operation) => Some(lower_integer_compare(nodes, *operation)),
		Node::IntegerNarrow(operation) => Some(convert::narrow_i64(nodes, operation.source)),
		Node::IntegerWiden(operation) => Some(convert::widen_i32(nodes, operation.source)),
		Node::IntegerSignExtend(operation) => Some(convert::sign_extend(
			nodes,
			operation.source,
			operation.kind,
		)),
		Node::IntegerConvertToNumber(operation) => Some(convert::convert_to_number(
			nodes,
			operation.source,
			operation.is_signed,
			operation.to,
			operation.from,
		)),
		Node::IntegerTransmuteToNumber(operation) => Some(convert::transmute_to_number(
			operation.source,
			operation.from,
		)),
		Node::NumberUnaryOperation(operation) => Some(lower_number_unary(nodes, *operation)),
		Node::NumberBinaryOperation(operation) => Some(lower_number_binary(nodes, *operation)),
		Node::NumberCompareOperation(operation) => Some(lower_number_compare(nodes, *operation)),
		Node::NumberNarrow(operation) => Some(convert::narrow_f64(nodes, operation.source)),
		Node::NumberWiden(operation) => Some(convert::widen_f32(nodes, operation.source)),
		Node::NumberTruncateToInteger(operation) => truncate::to_integer(nodes, operation),
		Node::NumberTransmuteToInteger(operation) => Some(convert::transmute_to_integer(
			operation.source,
			operation.from,
		)),
	};

	let lowered = result.is_some();

	match result {
		Some(result) => replace::replace_node(nodes, id, &[result]),
		None => nodes[index] = node,
	}

	lowered
}

fn lower_integer_unary(nodes: &mut Vec<Node>, operation: integer::UnaryOperation) -> Link {
	let integer::UnaryOperation {
		source,
		kind,
		operator,
	} = operation;

	match kind {
		integer::Type::I32 => lower_i32::unary(nodes, source, operator),
		integer::Type::I64 => lower_i64::unary(nodes, source, operator),
	}
}

fn lower_integer_binary(
	nodes: &mut Vec<Node>,
	operation: integer::BinaryOperation,
) -> Option<Link> {
	let integer::BinaryOperation {
		lhs,
		rhs,
		kind,
		operator,
	} = operation;

	match kind {
		integer::Type::I32 => Some(lower_i32::binary(nodes, lhs, rhs, operator)),
		integer::Type::I64 => lower_i64::binary(nodes, lhs, rhs, operator),
	}
}

fn lower_integer_compare(nodes: &mut Vec<Node>, operation: integer::CompareOperation) -> Link {
	let integer::CompareOperation {
		lhs,
		rhs,
		kind,
		operator,
	} = operation;

	match kind {
		integer::Type::I32 => lower_i32::compare(nodes, lhs, rhs, operator),
		integer::Type::I64 => lower_i64::compare(nodes, lhs, rhs, operator),
	}
}

fn lower_number_unary(nodes: &mut Vec<Node>, operation: number::UnaryOperation) -> Link {
	let number::UnaryOperation {
		source,
		kind,
		operator,
	} = operation;

	match kind {
		number::Type::F32 => lower_f32::unary(nodes, source, operator),
		number::Type::F64 => lower_f64::unary(nodes, source, operator),
	}
}

fn lower_number_binary(nodes: &mut Vec<Node>, operation: number::BinaryOperation) -> Link {
	let number::BinaryOperation {
		lhs,
		rhs,
		kind,
		operator,
	} = operation;

	match kind {
		number::Type::F32 => lower_f32::binary(nodes, lhs, rhs, operator),
		number::Type::F64 => lower_f64::binary(nodes, lhs, rhs, operator),
	}
}

fn lower_number_compare(nodes: &mut Vec<Node>, operation: number::CompareOperation) -> Link {
	let number::CompareOperation {
		lhs,
		rhs,
		kind,
		operator,
	} = operation;

	match kind {
		number::Type::F32 => lower_f32::compare(nodes, lhs, rhs, operator),
		number::Type::F64 => lower_f64::compare(nodes, lhs, rhs, operator),
	}
}
