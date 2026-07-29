//! Table (indexed-reference-array) operations.

use crate::{Link, Node};

use super::memory::Location;

/// A table creation node.
#[derive(Clone)]
pub struct TableNew {
	/// The initial elements and their offsets.
	pub initializer: Vec<(Link, u32)>,
	/// The minimum number of elements.
	pub minimum: u32,
	/// The maximum number of elements.
	pub maximum: u32,
}

impl TableNew {
	/// Adds a table creation node to the graph.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(
		nodes: &mut Vec<Node>,
		initializer: Vec<(Link, u32)>,
		minimum: u32,
		maximum: u32,
	) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::TableNew(Self {
			initializer,
			minimum,
			maximum,
		});

		nodes.push(node);

		Link(id, 0)
	}

	pub(crate) fn for_each_outer<H: FnMut(Link)>(&self, mut handler: H) {
		let Self { initializer, .. } = self;

		for item in initializer {
			handler(item.0);
		}
	}

	pub(crate) fn for_each_mut_outer<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		let Self { initializer, .. } = self;

		for item in initializer {
			handler(&mut item.0);
		}
	}
}

/// A table element read node.
#[derive(Clone, Copy)]
pub struct TableGet {
	/// The source location.
	pub source: Location,
}

impl TableGet {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a table read node and returns its value and state links.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, source: Location) -> (Link, Link) {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::TableGet(Self { source });

		nodes.push(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}

	handle_sources!((source, method));
}

/// A table element write node.
#[derive(Clone, Copy)]
pub struct TableSet {
	/// The destination location.
	pub destination: Location,
	/// The value being stored.
	pub source: Link,
}

impl TableSet {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 1;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a table write node and returns its state link.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, destination: Location, source: Link) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::TableSet(Self {
			destination,
			source,
		});

		nodes.push(node);

		Link(id, Self::STATE_PORT)
	}

	handle_sources!((destination, method), (source, link));
}

/// A table size query node.
#[derive(Clone, Copy)]
pub struct TableSize {
	/// The table being queried.
	pub source: Link,
}

impl TableSize {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a table size query node and returns its value and state links.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> (Link, Link) {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::TableSize(Self { source });

		nodes.push(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}

	handle_sources!((source, link));
}

/// A table grow node.
#[derive(Clone, Copy)]
pub struct TableGrow {
	/// The table being grown.
	pub destination: Link,
	/// The initial value for new elements.
	pub initializer: Link,
	/// The number of elements to grow by.
	pub size: Link,
}

impl TableGrow {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a table grow node and returns its value and state links.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(
		nodes: &mut Vec<Node>,
		destination: Link,
		initializer: Link,
		size: Link,
	) -> (Link, Link) {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::TableGrow(Self {
			destination,
			initializer,
			size,
		});

		nodes.push(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}

	handle_sources!((destination, link), (initializer, link), (size, link));
}

/// A table fill node.
#[derive(Clone, Copy)]
pub struct TableFill {
	/// The destination location.
	pub destination: Location,
	/// The fill value.
	pub source: Link,
	/// The number of elements to fill.
	pub size: Link,
}

impl TableFill {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 1;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a table fill node and returns its state link.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(
		nodes: &mut Vec<Node>,
		destination: Location,
		source: Link,
		size: Link,
	) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::TableFill(Self {
			destination,
			source,
			size,
		});

		nodes.push(node);

		Link(id, Self::STATE_PORT)
	}

	handle_sources!((destination, method), (source, link), (size, link));
}

/// A table copy node.
#[derive(Clone, Copy)]
pub struct TableCopy {
	/// The destination location.
	pub destination: Location,
	/// The source location.
	pub source: Location,
	/// The number of elements to copy.
	pub size: Link,
}

impl TableCopy {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the destination state token.
	pub const DESTINATION_STATE_PORT: u16 = 0;
	/// The port index for the source state token.
	pub const SOURCE_STATE_PORT: u16 = 1;

	/// Adds a table copy node and returns its destination and source state links.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(
		nodes: &mut Vec<Node>,
		destination: Location,
		source: Location,
		size: Link,
	) -> (Link, Link) {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::TableCopy(Self {
			destination,
			source,
			size,
		});

		nodes.push(node);

		(
			Link(id, Self::DESTINATION_STATE_PORT),
			Link(id, Self::SOURCE_STATE_PORT),
		)
	}

	handle_sources!((destination, method), (source, method), (size, link));
}

/// A table drop node.
#[derive(Clone, Copy)]
pub struct TableDrop {
	/// The table being dropped.
	pub source: Link,
}

impl TableDrop {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 1;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a table drop node and returns its state link.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::TableDrop(Self { source });

		nodes.push(node);

		Link(id, Self::STATE_PORT)
	}

	handle_sources!((source, link));
}
