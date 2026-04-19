//! Generic single-slot mutable storage operations.

use crate::{Link, Node};

/// A mutable-cell creation node.
#[derive(Clone, Copy)]
pub struct MutableNew {
	/// The link to the initial value.
	pub initializer: Link,
}

impl MutableNew {
	/// Adds a mutable-cell creation node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, initializer: Link) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::MutableNew(Self { initializer });

		nodes.push(node);

		Link(id, 0)
	}

	handle_sources!((initializer, link));
}

/// A mutable-cell read node.
#[derive(Clone, Copy)]
pub struct MutableGet {
	/// The link to the mutable cell.
	pub source: Link,
}

impl MutableGet {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a mutable-cell read node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> (Link, Link) {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::MutableGet(Self { source });

		nodes.push(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}

	handle_sources!((source, link));
}

/// A mutable-cell write node.
#[derive(Clone, Copy)]
pub struct MutableSet {
	/// The link to the mutable cell.
	pub destination: Link,
	/// The link to the value being stored.
	pub source: Link,
}

impl MutableSet {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 1;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a mutable-cell write node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, destination: Link, source: Link) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::MutableSet(Self {
			destination,
			source,
		});

		nodes.push(node);

		Link(id, Self::STATE_PORT)
	}

	handle_sources!((destination, link), (source, link));
}
