//! ISLE-based peephole optimizations.

use core::mem;

use ir_graph::{Link, Node, operation::Identity};

use self::internal::{
	constructor_SimplifyI32, constructor_SimplifyMemory, constructor_SimplifyMutable,
	constructor_SimplifyTable,
};

pub use self::context::RegionContext;

mod context;
mod internal;

fn replace_with_identity(nodes: &mut [Node], destination: u32, sources: &[Link]) {
	let sources = sources.iter().copied().collect();

	nodes[usize::try_from(destination).unwrap()] = Node::Identity(Identity { sources });
}

fn replace_with_direct(nodes: &mut [Node], destination: u32, source: u32) {
	let source = mem::take(&mut nodes[usize::try_from(source).unwrap()]);

	nodes[usize::try_from(destination).unwrap()] = source;
}

fn replace_node(nodes: &mut [Node], destination: u32, sources: &[Link]) {
	if let &[source] = sources
		&& source.1 == 0
		&& source.0 > destination
	{
		replace_with_direct(nodes, destination, source.0);
	} else {
		replace_with_identity(nodes, destination, sources);
	}
}

/// Simplifies an I32 operation at the given node ID.
pub fn simplify_i32(nodes: &mut Vec<Node>, id: u32) -> bool {
	constructor_SimplifyI32(&mut RegionContext(nodes), Link(id, 0)).is_some_and(|source| {
		replace_node(nodes, id, &[source]);

		true
	})
}

/// Simplifies a mutable-cell operation at the given node ID.
pub fn simplify_mutable(nodes: &mut Vec<Node>, id: u32) -> bool {
	constructor_SimplifyMutable(&mut RegionContext(nodes), Link(id, 0)).is_some_and(|sources| {
		replace_node(nodes, id, &sources.as_fixed());

		true
	})
}

/// Simplifies a table operation at the given node ID.
pub fn simplify_table(nodes: &mut Vec<Node>, id: u32) -> bool {
	constructor_SimplifyTable(&mut RegionContext(nodes), Link(id, 0)).is_some_and(|sources| {
		replace_node(nodes, id, &sources.as_fixed());

		true
	})
}

/// Simplifies a memory operation at the given node ID.
pub fn simplify_memory(nodes: &mut Vec<Node>, id: u32) -> bool {
	constructor_SimplifyMemory(&mut RegionContext(nodes), Link(id, 0)).is_some_and(|sources| {
		replace_node(nodes, id, &sources.as_fixed());

		true
	})
}
