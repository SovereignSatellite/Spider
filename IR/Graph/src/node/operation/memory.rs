//! Memory access and management operations.

#![expect(
	unused_variables,
	unused_mut,
	reason = "macro-generated visitors may not consume every field"
)]

use alloc::sync::Arc;

use crate::{Link, Node};

/// A memory location specified by a base reference and an offset.
///
/// `Location` is an abstract (reference, offset) pair. Any operation that
/// finds the shape useful is welcome to reuse it.
#[derive(Clone, Copy)]
pub struct Location {
	/// The base reference.
	pub reference: Link,
	/// The offset from the base reference.
	pub offset: Link,
}

impl Location {
	handle_sources!((reference, link), (offset, link));
}

/// A memory creation node.
#[derive(Clone)]
pub struct MemoryNew {
	/// The initial data segments and their offsets.
	pub initializer: Vec<(Arc<[u8]>, u32)>,
	/// The minimum number of pages.
	pub minimum: u32,
	/// The maximum number of pages.
	pub maximum: u32,
}

impl MemoryNew {
	/// Adds a memory creation node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		initializer: Vec<(Arc<[u8]>, u32)>,
		minimum: u32,
		maximum: u32,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::MemoryNew(Self {
			initializer,
			minimum,
			maximum,
		});

		nodes.push(node);

		Link(id, 0)
	}

	handle_sources!((initializer, ignore), (minimum, ignore), (maximum, ignore));
}

/// Source and target type pairs for memory loads.
#[expect(
	non_camel_case_types,
	reason = "variants encode source/target type pairs"
)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum LoadType {
	/// Loads a signed 8-bit value into a 32-bit integer.
	I32_S8,
	/// Loads an unsigned 8-bit value into a 32-bit integer.
	I32_U8,
	/// Loads a signed 16-bit value into a 32-bit integer.
	I32_S16,
	/// Loads an unsigned 16-bit value into a 32-bit integer.
	I32_U16,
	/// Loads a 32-bit integer.
	I32,

	/// Loads a signed 8-bit value into a 64-bit integer.
	I64_S8,
	/// Loads an unsigned 8-bit value into a 64-bit integer.
	I64_U8,
	/// Loads a signed 16-bit value into a 64-bit integer.
	I64_S16,
	/// Loads an unsigned 16-bit value into a 64-bit integer.
	I64_U16,
	/// Loads a signed 32-bit value into a 64-bit integer.
	I64_S32,
	/// Loads an unsigned 32-bit value into a 64-bit integer.
	I64_U32,
	/// Loads a 64-bit integer.
	I64,

	/// Loads a 32-bit float.
	F32,
	/// Loads a 64-bit float.
	F64,
}

/// A memory load node.
#[derive(Clone, Copy)]
pub struct MemoryLoad {
	/// The source location to load from.
	pub source: Location,
	/// The load type.
	pub kind: LoadType,
}

impl MemoryLoad {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a memory load node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, source: Location, kind: LoadType) -> (Link, Link) {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::MemoryLoad(Self { source, kind });

		nodes.push(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}

	handle_sources!((source, method), (kind, ignore));
}

/// Source and target type pairs for memory stores.
#[expect(
	non_camel_case_types,
	reason = "variants encode source/target type pairs"
)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum StoreType {
	/// Stores the low 8 bits of a 32-bit integer.
	I32_I8,
	/// Stores the low 16 bits of a 32-bit integer.
	I32_I16,
	/// Stores a 32-bit integer.
	I32,

	/// Stores the low 8 bits of a 64-bit integer.
	I64_I8,
	/// Stores the low 16 bits of a 64-bit integer.
	I64_I16,
	/// Stores the low 32 bits of a 64-bit integer.
	I64_I32,
	/// Stores a 64-bit integer.
	I64,

	/// Stores a 32-bit float.
	F32,
	/// Stores a 64-bit float.
	F64,
}

/// A memory store node.
#[derive(Clone, Copy)]
pub struct MemoryStore {
	/// The destination location.
	pub destination: Location,
	/// The link to the value being stored.
	pub source: Link,
	/// The store type.
	pub kind: StoreType,
}

impl MemoryStore {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 1;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a memory store node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		destination: Location,
		source: Link,
		kind: StoreType,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::MemoryStore(Self {
			destination,
			source,
			kind,
		});

		nodes.push(node);

		Link(id, Self::STATE_PORT)
	}

	handle_sources!((destination, method), (source, link), (kind, ignore));
}

/// A memory size query node.
#[derive(Clone, Copy)]
pub struct MemorySize {
	/// The link to the memory being queried.
	pub source: Link,
}

impl MemorySize {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a memory size query node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> (Link, Link) {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::MemorySize(Self { source });

		nodes.push(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}

	handle_sources!((source, link));
}

/// A memory grow node.
#[derive(Clone, Copy)]
pub struct MemoryGrow {
	/// The link to the memory being grown.
	pub destination: Link,
	/// The number of pages to grow by.
	pub size: Link,
}

impl MemoryGrow {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a memory grow node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, destination: Link, size: Link) -> (Link, Link) {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::MemoryGrow(Self { destination, size });

		nodes.push(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}

	handle_sources!((destination, link), (size, link));
}

/// A memory fill node.
#[derive(Clone, Copy)]
pub struct MemoryFill {
	/// The destination location.
	pub destination: Location,
	/// The byte value to fill with.
	pub byte: Link,
	/// The number of bytes to fill.
	pub size: Link,
}

impl MemoryFill {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 1;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a memory fill node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, destination: Location, byte: Link, size: Link) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::MemoryFill(Self {
			destination,
			byte,
			size,
		});

		nodes.push(node);

		Link(id, Self::STATE_PORT)
	}

	handle_sources!((destination, method), (byte, link), (size, link));
}

/// A memory copy node.
#[derive(Clone, Copy)]
pub struct MemoryCopy {
	/// The destination location.
	pub destination: Location,
	/// The source location.
	pub source: Location,
	/// The number of bytes to copy.
	pub size: Link,
}

impl MemoryCopy {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the destination state token.
	pub const DESTINATION_STATE_PORT: u16 = 0;
	/// The port index for the source state token.
	pub const SOURCE_STATE_PORT: u16 = 1;

	/// Adds a memory copy node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		destination: Location,
		source: Location,
		size: Link,
	) -> (Link, Link) {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::MemoryCopy(Self {
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

/// A memory drop node.
#[derive(Clone, Copy)]
pub struct MemoryDrop {
	/// The link to the memory being dropped.
	pub source: Link,
}

impl MemoryDrop {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 1;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a memory drop node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::MemoryDrop(Self { source });

		nodes.push(node);

		Link(id, Self::STATE_PORT)
	}

	handle_sources!((source, link));
}
