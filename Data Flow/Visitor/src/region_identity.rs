use data_flow_graph::{
	mvp::Identity,
	nested::{RegionOut, ThetaIn, ThetaOut},
	DataFlowGraph, Link, Node,
};

fn replace_with_producer(graph: &mut DataFlowGraph, from: &mut Link) {
	let Node::Identity(Identity { source }) = *graph.get(from.0) else {
		return;
	};

	*from = source;
}

pub fn remove(graph: &mut DataFlowGraph) {
	for id in 0..graph.len().try_into().unwrap() {
		let mut node = std::mem::take(graph.get_mut(id));

		if matches!(
			node,
			Node::RegionOut(_) | Node::ThetaIn(_) | Node::ThetaOut(_)
		) {
			node.for_each_mut_argument(|argument| replace_with_producer(graph, argument));
		}

		*graph.get_mut(id) = node;
	}
}

fn replace_with_identity(graph: &mut DataFlowGraph, from: &mut Link) {
	let identity = graph.add_identity(*from);

	*from = identity;
}

fn replace_if_producer(graph: &mut DataFlowGraph, from: &mut Link, producer: u32) {
	if from.0 != producer {
		return;
	}

	replace_with_identity(graph, from);
}

// NOTE: We insert at...
// `RegionOut` arguments if they reference the start, since we need to issue the correct move order.
// `ThetaIn` arguments always, since they are mutable and must produce new locals.
// `ThetaOut` arguments and condition if they reference the start, since we need to issue the correct move order.
fn insert_at(graph: &mut DataFlowGraph, node: &mut Node) {
	match node {
		Node::RegionOut(RegionOut { input, results, .. }) => {
			for result in results {
				replace_if_producer(graph, result, *input);
			}
		}
		Node::ThetaIn(ThetaIn { arguments, .. }) => {
			for argument in arguments {
				replace_with_identity(graph, argument);
			}
		}
		Node::ThetaOut(ThetaOut {
			condition, results, ..
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
	for id in 0..graph.len().try_into().unwrap() {
		let mut node = std::mem::take(graph.get_mut(id));

		insert_at(graph, &mut node);

		*graph.get_mut(id) = node;
	}
}
