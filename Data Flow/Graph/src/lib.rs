#![no_std]
#![expect(clippy::missing_panics_doc)]

extern crate alloc;

mod dot;
mod link;
mod node;

use alloc::vec::Vec;

pub use list;

pub use self::{
	dot::Dot,
	link::Link,
	node::{Node, control, simple},
};

/// A directed graph of nodes containing operations.
pub struct DataFlowGraph {
	nodes: Vec<Node>,
}

impl DataFlowGraph {
	#[must_use]
	pub const fn new() -> Self {
		Self { nodes: Vec::new() }
	}

	#[must_use]
	pub const fn len(&self) -> usize {
		self.nodes.len()
	}

	#[must_use]
	pub const fn is_empty(&self) -> bool {
		self.nodes.is_empty()
	}

	#[must_use]
	pub fn get(&self, id: u32) -> &Node {
		&self.nodes[usize::try_from(id).unwrap()]
	}

	#[must_use]
	pub fn get_mut(&mut self, id: u32) -> &mut Node {
		&mut self.nodes[usize::try_from(id).unwrap()]
	}

	#[must_use]
	pub const fn inner_mut(&mut self) -> &mut Vec<Node> {
		&mut self.nodes
	}

	pub fn nodes(&self) -> core::slice::Iter<'_, Node> {
		self.nodes.iter()
	}

	pub fn nodes_mut(&mut self) -> core::slice::IterMut<'_, Node> {
		self.nodes.iter_mut()
	}

	pub fn add_node(&mut self, node: Node) -> u32 {
		let position = self.nodes.len();

		self.nodes.push(node);

		position.try_into().unwrap()
	}
}

impl Default for DataFlowGraph {
	fn default() -> Self {
		Self::new()
	}
}
