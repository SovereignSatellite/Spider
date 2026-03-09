//! Data flow graph intermediate representation.

#![no_std]
#![expect(
	clippy::multiple_inherent_impl,
	reason = "macro-generated visitor impl blocks are separate from constructor impl blocks"
)]

extern crate alloc;

mod link;
mod node;

use alloc::vec::Vec;

pub use list;

pub use self::{
	link::Link,
	node::{Node, control, simple},
};

/// A directed graph of nodes containing operations.
pub struct DataFlowGraph {
	nodes: Vec<Node>,
}

impl DataFlowGraph {
	/// Creates a new empty graph.
	#[must_use]
	pub const fn new() -> Self {
		Self { nodes: Vec::new() }
	}

	/// Returns the number of nodes in the graph.
	#[must_use]
	pub const fn len(&self) -> usize {
		self.nodes.len()
	}

	/// Returns `true` if the graph contains no nodes.
	#[must_use]
	pub const fn is_empty(&self) -> bool {
		self.nodes.is_empty()
	}

	/// Returns a reference to the node with the given identifier.
	///
	/// # Panics
	///
	/// Panics if `id` is out of bounds.
	#[must_use]
	pub fn get(&self, id: u32) -> &Node {
		&self.nodes[usize::try_from(id).unwrap()]
	}

	/// Returns a mutable reference to the node with the given identifier.
	///
	/// # Panics
	///
	/// Panics if `id` is out of bounds.
	#[must_use]
	pub fn get_mut(&mut self, id: u32) -> &mut Node {
		&mut self.nodes[usize::try_from(id).unwrap()]
	}

	/// Returns a mutable reference to the underlying node storage.
	#[must_use]
	pub const fn inner_mut(&mut self) -> &mut Vec<Node> {
		&mut self.nodes
	}

	/// Returns an iterator over all nodes.
	pub fn nodes(&self) -> core::slice::Iter<'_, Node> {
		self.nodes.iter()
	}

	/// Returns a mutable iterator over all nodes.
	pub fn nodes_mut(&mut self) -> core::slice::IterMut<'_, Node> {
		self.nodes.iter_mut()
	}

	/// Adds a node to the graph and returns its identifier.
	///
	/// # Panics
	///
	/// Panics if the number of nodes exceeds `u32::MAX`.
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
