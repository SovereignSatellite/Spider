//! Region identity insertion and removal.

use ir_graph::{Link, Node, Region, list, simple::Identity};

fn route_to_source(nodes: &[Node], from: &mut Link) {
	let id = usize::try_from(from.0).unwrap();
	let port = usize::from(from.1);

	if let Node::Identity(Identity { sources }) = &nodes[id] {
		*from = sources[port];
	}
}

fn route_all_to_source(nodes: &[Node], node: &mut Node) {
	node.for_each_mut_outer(|from| route_to_source(nodes, from));
}

/// Removes all identity nodes from the region.
pub fn remove(nodes: &mut [Node]) {
	for id in 0..nodes.len() {
		let mut node = core::mem::take(&mut nodes[id]);

		route_all_to_source(nodes, &mut node);

		nodes[id] = node;
	}
}

fn route_to_identity(nodes: &mut Vec<Node>, from: &mut Link) {
	let sources = list::resizable![*from];
	let id = Identity::add_into(nodes, sources);

	*from = Link(id, 0);
}

fn route_all_to_identity(nodes: &mut Vec<Node>, results: &mut [Link]) {
	for result in results {
		route_to_identity(nodes, result);
	}
}

fn route_results_node(nodes: &mut Vec<Node>, position: usize) {
	let mut results_node = core::mem::take(&mut nodes[position]);

	results_node.for_each_mut_outer(|link| route_to_identity(nodes, link));

	nodes[position] = results_node;
}

fn route_region_results(region: &mut Region) {
	match region {
		Region::Module(_) | Region::Function(_) => {}

		Region::Branch(region) => {
			let position = region.results_index();

			route_results_node(&mut region.nodes, position);
		}
		Region::Repeat(region) => {
			let position = region.results_index();

			route_results_node(&mut region.nodes, position);
		}
	}
}

fn route_repeat_child(nodes: &mut Vec<Node>, node: &Node) {
	if let Node::Repeat(region) = node {
		let mut guard = region.lock();

		route_all_to_identity(nodes, &mut guard.arguments);
	}
}

fn route_children(nodes: &mut Vec<Node>) {
	for id in 0..nodes.len() {
		let node = core::mem::take(&mut nodes[id]);

		route_repeat_child(nodes, &node);

		nodes[id] = node;
	}
}

/// Inserts identity nodes at control flow boundaries in the region.
pub fn insert(region: &mut Region) {
	route_region_results(region);
	route_children(region.nodes_mut());
}
