use alloc::sync::Arc;
use core::{iter::once, mem};

use parking_lot::Mutex;

use ir_graph::{
	Link, Node,
	operation::Identity,
	region::{Branch, Match},
	tracer::identity_source,
};

use super::{
	context::RegionContext,
	internal::{constructor_ReduceI32Table, constructor_ReduceI64Table},
};

#[derive(Clone, Copy)]
enum IntegerConstant {
	I32(i32),
	I64(i64),
}

struct BinaryMatchBranches {
	on_false: Arc<Mutex<Branch>>,
	on_true: Arc<Mutex<Branch>>,
}

fn integer_constant_at(branch: &Mutex<Branch>, output_port: u16) -> Option<IntegerConstant> {
	let guard = branch.lock();
	let source = identity_source(
		&guard.nodes,
		guard.results().sources[usize::from(output_port)],
	);
	let node = &guard.nodes[usize::try_from(source.0).unwrap()];
	let constant = if let &Node::I32(value) = node {
		Some(IntegerConstant::I32(value))
	} else if let &Node::I64(value) = node {
		Some(IntegerConstant::I64(value))
	} else {
		None
	};
	drop(guard);

	constant
}

fn reduce_output(
	nodes: &mut Vec<Node>,
	predicate: Link,
	branches: &BinaryMatchBranches,
	output_port: u16,
) -> Option<Link> {
	match (
		integer_constant_at(&branches.on_false, output_port)?,
		integer_constant_at(&branches.on_true, output_port)?,
	) {
		(IntegerConstant::I32(on_false), IntegerConstant::I32(on_true)) => Some(
			constructor_ReduceI32Table(&mut RegionContext(nodes), predicate, on_false, on_true),
		),
		(IntegerConstant::I64(on_false), IntegerConstant::I64(on_true)) => Some(
			constructor_ReduceI64Table(&mut RegionContext(nodes), predicate, on_false, on_true),
		),
		_ => None,
	}
}

fn reduce_match(
	nodes: &mut Vec<Node>,
	match_index: usize,
	match_region: &Arc<Mutex<Match>>,
) -> bool {
	let (predicate, branches, result_count) = {
		let guard = match_region.lock();
		let [on_false, on_true] = guard.branches.as_slice() else {
			return false;
		};

		(
			guard.condition,
			BinaryMatchBranches {
				on_false: Arc::clone(on_false),
				on_true: Arc::clone(on_true),
			},
			guard.result_count(),
		)
	};
	let mut remaining_output_ports = 0..result_count;
	let Some((first_reduced_port, first_replacement)) =
		remaining_output_ports.find_map(|output_port| {
			reduce_output(nodes, predicate, &branches, output_port)
				.map(|replacement| (output_port, replacement))
		})
	else {
		return false;
	};
	let moved_match_identifier = u32::try_from(nodes.len()).unwrap();
	let moved_match = mem::take(&mut nodes[match_index]);

	nodes.push(moved_match);

	let replacement_sources = (0..first_reduced_port)
		.map(|output_port| Link(moved_match_identifier, output_port))
		.chain(once(first_replacement))
		.chain(remaining_output_ports.map(|output_port| {
			reduce_output(nodes, predicate, &branches, output_port)
				.unwrap_or(Link(moved_match_identifier, output_port))
		}))
		.collect();

	nodes[match_index] = Node::Identity(Identity {
		sources: replacement_sources,
	});

	true
}

/// Reduce binary Match outputs selected from integer constants.
#[must_use = "propagate whether this pass changed the graph"]
pub fn reduce_match_outputs(nodes: &mut Vec<Node>) -> bool {
	let original_node_count = nodes.len();
	let mut any_match_moved = false;

	for node_index in 0..original_node_count {
		let Node::Match(match_region) = &nodes[node_index] else {
			continue;
		};
		let match_region = Arc::clone(match_region);

		any_match_moved |= reduce_match(nodes, node_index, &match_region);
	}

	any_match_moved
}
