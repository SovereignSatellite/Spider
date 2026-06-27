use core::mem;

use ir_graph::Node;

use super::{constant_at, inline::inline_once};

pub fn fold(nodes: &mut Vec<Node>, id: usize) -> bool {
	let Node::Repeat(arc) = &nodes[id] else {
		return false;
	};

	let mut repeat = arc.lock();

	if constant_at(&repeat.nodes, repeat.results().condition) != Some(0_i32) {
		return false;
	}

	let inputs = repeat.arguments.clone();
	let body = mem::take(&mut repeat.nodes);

	drop(repeat);

	inline_once(nodes, id, body, &inputs);

	true
}
