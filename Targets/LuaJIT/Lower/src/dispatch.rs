//! Reverse-order region sweep that expands trivial nodes in place.

use core::mem;

use ir_graph::{
	Link, Node, Region,
	operation::{integer, number},
};
use ir_passes::catalog::{Optimization, Optimizations};

use crate::{
	convert, f32 as lower_f32, f64 as lower_f64, i32 as lower_i32, i64 as lower_i64, memory,
	replace, table, truncate,
};

/// Lowers every trivial node in the region, reporting whether anything changed.
#[must_use = "propagate whether this pass changed the graph"]
pub fn apply(region: &mut Region, optimizations: &Optimizations) -> bool {
	let nodes = region.nodes_mut();

	let Ok(length) = u32::try_from(nodes.len()) else {
		unreachable!()
	};

	let mut changed = false;

	for identifier in (0..length).rev() {
		changed |= lower_node(nodes, optimizations, identifier);
	}

	changed
}

enum LoweringResult {
	ReplaceWith(Link),
	AlreadyReplaced,
}

#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
fn lower_node(nodes: &mut Vec<Node>, optimizations: &Optimizations, identifier: u32) -> bool {
	let index = usize::try_from(identifier).unwrap();
	let node = mem::take(&mut nodes[index]);

	let lowering = match &node {
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
		| Node::TableGrow(_)
		| Node::TableFill(_)
		| Node::TableCopy(_)
		| Node::TableDrop(_)
		| Node::MemoryNew(_)
		| Node::MemoryFill(_)
		| Node::MemoryCopy(_)
		| Node::MemoryDrop(_) => None,

		Node::IntegerUnaryOperation(operation) => optimizations
			.is_enabled(Optimization::for_integer_unary_lowering(
				operation.kind,
				operation.operator,
			))
			.then(|| LoweringResult::ReplaceWith(lower_integer_unary(nodes, *operation))),
		Node::IntegerBinaryOperation(operation) => optimizations
			.is_enabled(Optimization::for_integer_binary_lowering(
				operation.kind,
				operation.operator,
			))
			.then(|| LoweringResult::ReplaceWith(lower_integer_binary(nodes, *operation))),
		Node::IntegerCompareOperation(operation) => optimizations
			.is_enabled(Optimization::for_integer_comparison_lowering(
				operation.kind,
				operation.operator,
			))
			.then(|| LoweringResult::ReplaceWith(lower_integer_compare(nodes, *operation))),
		Node::IntegerNarrow(operation) => optimizations
			.lower_i64_narrow_to_i32
			.then(|| LoweringResult::ReplaceWith(convert::narrow_i64(nodes, operation.source))),
		Node::IntegerWiden(operation) => optimizations
			.lower_i32_widen_to_i64
			.then(|| LoweringResult::ReplaceWith(convert::widen_i32(nodes, operation.source))),
		Node::IntegerSignExtend(operation) => optimizations
			.is_enabled(Optimization::for_integer_sign_extension_lowering(
				operation.kind,
			))
			.then(|| {
				LoweringResult::ReplaceWith(convert::sign_extend(
					nodes,
					operation.source,
					operation.kind,
				))
			}),
		Node::IntegerConvertToNumber(operation) => optimizations
			.is_enabled(Optimization::for_integer_to_number_lowering(
				operation.from,
				operation.to,
				operation.is_signed,
			))
			.then(|| {
				LoweringResult::ReplaceWith(convert::convert_to_number(
					nodes,
					operation.source,
					operation.is_signed,
					operation.to,
					operation.from,
				))
			}),
		Node::IntegerTransmuteToNumber(operation) => optimizations
			.is_enabled(Optimization::for_integer_transmute_to_number_lowering(
				operation.from,
			))
			.then(|| {
				LoweringResult::ReplaceWith(convert::transmute_to_number(
					operation.source,
					operation.from,
				))
			}),
		Node::NumberUnaryOperation(operation) => optimizations
			.is_enabled(Optimization::for_number_unary_lowering(
				operation.kind,
				operation.operator,
			))
			.then(|| LoweringResult::ReplaceWith(lower_number_unary(nodes, *operation))),
		Node::NumberBinaryOperation(operation) => optimizations
			.is_enabled(Optimization::for_number_binary_lowering(
				operation.kind,
				operation.operator,
			))
			.then(|| LoweringResult::ReplaceWith(lower_number_binary(nodes, *operation))),
		Node::NumberCompareOperation(operation) => optimizations
			.is_enabled(Optimization::for_number_comparison_lowering(
				operation.kind,
				operation.operator,
			))
			.then(|| LoweringResult::ReplaceWith(lower_number_compare(nodes, *operation))),
		Node::NumberNarrow(operation) => optimizations
			.lower_f64_narrow_to_f32
			.then(|| LoweringResult::ReplaceWith(convert::narrow_f64(nodes, operation.source))),
		Node::NumberWiden(operation) => optimizations
			.lower_f32_widen_to_f64
			.then(|| LoweringResult::ReplaceWith(convert::widen_f32(nodes, operation.source))),
		Node::NumberTruncateToInteger(operation) => optimizations
			.is_enabled(Optimization::for_number_to_integer_lowering(
				operation.from,
				operation.to,
				operation.is_signed,
				operation.is_saturating,
			))
			.then(|| LoweringResult::ReplaceWith(truncate::to_integer(nodes, operation))),
		Node::NumberTransmuteToInteger(operation) => optimizations
			.is_enabled(Optimization::for_number_transmute_to_integer_lowering(
				operation.from,
			))
			.then(|| {
				LoweringResult::ReplaceWith(convert::transmute_to_integer(
					operation.source,
					operation.from,
				))
			}),
		Node::TableGet(operation) => optimizations.lower_table_get.then(|| {
			table::lower_get(nodes, identifier, operation.source);

			LoweringResult::AlreadyReplaced
		}),
		Node::TableSet(operation) => optimizations.lower_table_set.then(|| {
			table::lower_set(nodes, identifier, operation.destination, operation.source);

			LoweringResult::AlreadyReplaced
		}),
		Node::TableSize(operation) => optimizations.lower_table_size.then(|| {
			table::lower_size(nodes, identifier, operation.source);

			LoweringResult::AlreadyReplaced
		}),
		Node::MemoryLoad(operation) => optimizations
			.is_enabled(Optimization::for_memory_load_lowering(operation.kind))
			.then(|| {
				memory::lower_load(nodes, identifier, operation.source, operation.kind);

				LoweringResult::AlreadyReplaced
			}),
		Node::MemoryStore(operation) => optimizations
			.is_enabled(Optimization::for_memory_store_lowering(operation.kind))
			.then(|| {
				memory::lower_store(
					nodes,
					identifier,
					operation.destination,
					operation.source,
					operation.kind,
				);

				LoweringResult::AlreadyReplaced
			}),
	};

	match lowering {
		Some(LoweringResult::ReplaceWith(link)) => {
			replace::replace_node(nodes, identifier, &[link]);

			true
		}
		Some(LoweringResult::AlreadyReplaced) => true,
		None => {
			nodes[index] = node;

			false
		}
	}
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

fn lower_integer_binary(nodes: &mut Vec<Node>, operation: integer::BinaryOperation) -> Link {
	let integer::BinaryOperation {
		lhs,
		rhs,
		kind,
		operator,
	} = operation;

	match kind {
		integer::Type::I32 => lower_i32::binary(nodes, lhs, rhs, operator),
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
