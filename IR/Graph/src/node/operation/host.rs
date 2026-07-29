//! Host environment import and export operations.

#![expect(
	unused_variables,
	unused_mut,
	reason = "macro-generated visitors may not consume every field"
)]

use alloc::sync::Arc;

use crate::{Link, Node};

/// A resolved reference into the runtime-provided host import registry.
#[derive(Clone)]
pub struct Import {
	/// The import namespace (e.g. `"wasi_snapshot_preview1"`).
	pub namespace: Arc<str>,
	/// The import identifier within the namespace (e.g. `"fd_write"`).
	pub identifier: Arc<str>,
}

impl Import {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 1;
	/// The port index for the imported value.
	pub const RESULT_PORT: u16 = 0;

	/// Adds an `Import` node to the graph and returns the imported value.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, namespace: Arc<str>, identifier: Arc<str>) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::Import(Self {
			namespace,
			identifier,
		});

		nodes.push(node);

		Link(id, Self::RESULT_PORT)
	}

	handle_sources!((namespace, ignore), (identifier, ignore));
}

/// A write into the runtime-provided host export registry under a named identifier.
#[derive(Clone)]
pub struct Export {
	/// The export identifier (e.g. `"memory"`).
	pub identifier: Arc<str>,
	/// The value being exported.
	pub value: Link,
}

impl Export {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 1;
	/// The port index for the state edge that forces this export to be observed.
	pub const STATE_PORT: u16 = 0;

	/// Adds an `Export` node to the graph and returns its state edge.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, identifier: Arc<str>, value: Link) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::Export(Self { identifier, value });

		nodes.push(node);

		Link(id, Self::STATE_PORT)
	}

	handle_sources!((identifier, ignore), (value, link));
}
