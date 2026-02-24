mod context;
mod internal;

use ir_graph::{DataFlowGraph, Link, Node, simple::Identity};

use self::internal::{
	constructor_SimplifyGlobal, constructor_SimplifyI32, constructor_SimplifyTable,
};

fn replace_with_identity(graph: &mut DataFlowGraph, destination: u32, sources: &[Link]) {
	let sources = sources.iter().copied().collect();

	*graph.get_mut(destination) = Node::Identity(Identity { sources });
}

fn replace_with_direct(graph: &mut DataFlowGraph, destination: u32, source: u32) {
	let source = core::mem::take(graph.get_mut(source));

	*graph.get_mut(destination) = source;
}

fn replace_node(graph: &mut DataFlowGraph, destination: u32, sources: &[Link]) {
	if let &[source] = sources
		&& source.1 == 0
		&& source.0 > destination
	{
		replace_with_direct(graph, destination, source.0);
	} else {
		replace_with_identity(graph, destination, sources);
	}
}

pub fn simplify_i32(graph: &mut DataFlowGraph, id: u32) -> bool {
	constructor_SimplifyI32(graph, Link(id, 0)).is_some_and(|source| {
		replace_node(graph, id, &[source]);

		true
	})
}

pub fn simplify_global(graph: &mut DataFlowGraph, id: u32) -> bool {
	constructor_SimplifyGlobal(graph, Link(id, 0)).is_some_and(|sources| {
		replace_node(graph, id, &sources.as_fixed());

		true
	})
}

pub fn simplify_table(graph: &mut DataFlowGraph, id: u32) -> bool {
	constructor_SimplifyTable(graph, Link(id, 0)).is_some_and(|sources| {
		replace_node(graph, id, &sources.as_fixed());

		true
	})
}
