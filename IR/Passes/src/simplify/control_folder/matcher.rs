use alloc::sync::Arc;
use core::mem;

use ir_graph::Node;

use super::{constant_at, inline::inline_once};

#[must_use = "propagate whether this pass changed the graph"]
pub fn fold(nodes: &mut Vec<Node>, id: usize) -> bool {
	let Node::Match(arc) = &nodes[id] else {
		return false;
	};

	let matcher = arc.lock();

	let Some(selector) = constant_at(nodes, matcher.condition) else {
		return false;
	};

	let index = usize::try_from(selector.cast_unsigned()).unwrap();
	let chosen = Arc::clone(&matcher.branches[index]);
	let inputs = matcher.arguments.clone();

	drop(matcher);

	let body = mem::take(&mut chosen.lock().nodes);

	inline_once(nodes, id, body, &inputs);

	true
}
