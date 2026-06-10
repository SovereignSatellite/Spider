use ir_graph::{
	Link, Node,
	operation::{Aggregate, Apply, Extract},
};

const FUNCTION_PORT: u32 = 0;
const STATE_PORT: u32 = 1;

pub fn wrap(nodes: &mut Vec<Node>, function: Link, state: Link) -> Link {
	Aggregate::add_into(nodes, vec![function, state])
}

pub fn split(nodes: &mut Vec<Node>, closure: Link) -> (Link, Link) {
	let function = Extract::add_into(nodes, closure, FUNCTION_PORT);
	let state = Extract::add_into(nodes, closure, STATE_PORT);

	(function, state)
}

pub fn apply(nodes: &mut Vec<Node>, closure: Link, argument: Link, result_count: u16) -> u32 {
	let (function, state) = split(nodes, closure);

	Apply::add_into(nodes, function, vec![state, argument], result_count)
}
