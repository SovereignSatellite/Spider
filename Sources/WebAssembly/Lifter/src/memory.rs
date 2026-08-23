use alloc::sync::Arc;

use ir_graph::{
	Link, Node,
	operation::{Aggregate, MemoryNew, MutableNew},
};

pub const CONTENT_FIELD: u32 = 0;
pub const SIZE_FIELD: u32 = 1;
pub const MAXIMUM_FIELD: u32 = 2;

pub fn create(
	nodes: &mut Vec<Node>,
	initializer: Vec<(Arc<[u8]>, u32)>,
	size: u32,
	maximum: u32,
) -> Link {
	let size = Node::add_i32_into(nodes, size.cast_signed());
	let maximum = Node::add_i32_into(nodes, maximum.cast_signed());

	let content = MemoryNew::add_into(nodes, initializer, size);
	let content = MutableNew::add_into(nodes, content);
	let size = MutableNew::add_into(nodes, size);

	Aggregate::add_into(nodes, vec![content, size, maximum])
}
