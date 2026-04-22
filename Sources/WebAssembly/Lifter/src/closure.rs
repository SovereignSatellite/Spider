use core::iter;

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

pub fn apply<I>(nodes: &mut Vec<Node>, closure: Link, extra_arguments: I, result_count: u16) -> u32
where
	I: IntoIterator<Item = Link>,
{
	let (function, state) = split(nodes, closure);
	let arguments = iter::once(state).chain(extra_arguments).collect();

	Apply::add_into(nodes, function, arguments, result_count)
}
