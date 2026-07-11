//! Memory access and management operations.

#![expect(
	unused_variables,
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

/// A memory creation node yielding a fixed-size memory, or `Null` when the
/// allocation fails.
#[derive(Clone)]
pub struct MemoryNew {
	/// The initial contents and their offsets.
	pub initializer: Vec<(Arc<[u8]>, u32)>,
	/// The size in bytes.
	pub size: Link,
}

impl MemoryNew {
	/// Adds a memory creation node to the graph.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, initializer: Vec<(Arc<[u8]>, u32)>, size: Link) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::MemoryNew(Self { initializer, size });

		nodes.push(node);

		Link(id, 0)
	}

	handle_sources!((initializer, ignore), (size, link));
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
	/// The source location.
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

	/// Adds a memory load node and returns its value and state links.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, source: Location, kind: LoadType) -> (Link, Link) {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
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
	/// The value being stored.
	pub source: Link,
	/// The store type.
	pub kind: StoreType,
}

impl MemoryStore {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 1;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a memory store node and returns its state link.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(
		nodes: &mut Vec<Node>,
		destination: Location,
		source: Link,
		kind: StoreType,
	) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
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

	/// Adds a memory fill node and returns its state link.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, destination: Location, byte: Link, size: Link) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
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

	/// Adds a memory copy node and returns its destination and source state links.
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

/// A memory deallocation node.
#[derive(Clone, Copy)]
pub struct MemoryDrop {
	/// The memory being dropped.
	pub source: Link,
}

impl MemoryDrop {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 1;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a memory drop node and returns its state link.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::MemoryDrop(Self { source });

		nodes.push(node);

		Link(id, Self::STATE_PORT)
	}

	handle_sources!((source, link));
}
