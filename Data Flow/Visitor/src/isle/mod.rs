mod context;
mod internal;

use data_flow_graph::{DataFlowGraph, Link, Node, base::Identity};

use self::internal::{
	constructor_SimplifyGlobal, constructor_SimplifyI32, constructor_SimplifyTable,
};

fn replace_node_content(graph: &mut DataFlowGraph, destination: u32, source: u32) {
	let source = if source < destination {
		Node::Identity(Identity {
			source: Link(source, 0),
		})
	} else {
		core::mem::take(graph.get_mut(source))
	};

	*graph.get_mut(destination) = source;
}

pub fn simplify_i32(graph: &mut DataFlowGraph, id: u32) -> bool {
	constructor_SimplifyI32(graph, Link(id, 0)).is_some_and(|source| {
		replace_node_content(graph, id, source.0);

		true
	})
}

pub fn simplify_global(graph: &mut DataFlowGraph, id: u32) -> bool {
	constructor_SimplifyGlobal(graph, Link(id, 0)).is_some_and(|source| {
		replace_node_content(graph, id, source.0);

		true
	})
}

pub fn simplify_table(graph: &mut DataFlowGraph, id: u32) -> bool {
	constructor_SimplifyTable(graph, Link(id, 0)).is_some_and(|source| {
		replace_node_content(graph, id, source.0);

		true
	})
}
