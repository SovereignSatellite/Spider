use data_flow_graph::{
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

fn remove_at(graph: &DataFlowGraph, node: &mut Node) {
	if !matches!(
		node,
		Node::RegionOut(_) | Node::ThetaIn(_) | Node::ThetaOut(_)
	) {
		return;
	}

	node.for_each_mut_argument(|argument| replace_with_producer(graph, argument));
}

pub fn remove(graph: &mut DataFlowGraph) {
	let len = graph.len();

	for id in 0..len.try_into().unwrap() {
		let mut node = std::mem::take(graph.get_mut(id));

		remove_at(graph, &mut node);

		*graph.get_mut(id) = node;
	}
}

fn replace_with_identity(graph: &mut DataFlowGraph, from: &mut Link) {
	let sources = list::resizable![*from];

	*from = Identity::add_into(graph, sources);
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

		_ => {}
	}
}

pub fn insert(graph: &mut DataFlowGraph) {
	let len = graph.len();

	for id in 0..len.try_into().unwrap() {
		let mut node = std::mem::take(graph.get_mut(id));

		insert_at(graph, &mut node);

		*graph.get_mut(id) = node;
	}
}
