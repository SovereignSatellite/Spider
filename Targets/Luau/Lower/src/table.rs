//! Lowerings for table element reads and writes.

use ir_graph::{
	Link, Node,
	operation::{Location, TableGet, TableSet},
	region::Match,
};
use luau_foreign::{BooleanToInteger, LuauLessThanEqual, TableLength, TableLoad, TableStore};

use crate::replace;

/// Lowers a table get or set in place, reporting whether one matched.
pub fn lower(nodes: &mut Vec<Node>, id: u32) -> bool {
	let index = usize::try_from(id).unwrap();

	if let &Node::TableGet(TableGet { source }) = &nodes[index] {
		get(nodes, id, source);

		return true;
	}

	if let &Node::TableSet(TableSet {
		destination,
		source,
	}) = &nodes[index]
	{
		set(nodes, id, destination, source);

		return true;
	}

	false
}

fn get(nodes: &mut Vec<Node>, id: u32, source: Location) {
	let condition = out_of_bounds(nodes, source.reference, source.offset);
	let matcher = Match::add_if_into(
		nodes,
		vec![source.reference, source.offset],
		condition,
		|nodes, arguments| {
			vec![TableLoad::add_into(
				nodes,
				Link(arguments, 0),
				Link(arguments, 1),
			)]
		},
		|nodes, _arguments| vec![Node::add_trap_into(nodes)],
	);

	replace::replace_read(nodes, id, Link(matcher, 0), source.reference, &[Link(matcher, 0)]);
}

fn set(nodes: &mut Vec<Node>, id: u32, destination: Location, source: Link) {
	let condition = out_of_bounds(nodes, destination.reference, destination.offset);
	let matcher = Match::add_if_into(
		nodes,
		vec![destination.reference, destination.offset, source],
		condition,
		|nodes, arguments| {
			vec![TableStore::add_into(
				nodes,
				Link(arguments, 0),
				Link(arguments, 1),
				Link(arguments, 2),
			)]
		},
		|nodes, _arguments| vec![Node::add_trap_into(nodes)],
	);

	replace::replace_node(nodes, id, &[Link(matcher, 0)]);
}

fn out_of_bounds(nodes: &mut Vec<Node>, reference: Link, offset: Link) -> Link {
	let length = TableLength::add_into(nodes, reference);
	let beyond = LuauLessThanEqual::add_into(nodes, length, offset);

	BooleanToInteger::add_into(nodes, beyond)
}
