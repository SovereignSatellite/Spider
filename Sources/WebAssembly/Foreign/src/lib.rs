//! WebAssembly-specific `Foreign` types.

#![no_std]

extern crate alloc;

use alloc::{boxed::Box, sync::Arc, vec::Vec};

use ir_graph::{Link, Node, foreign::Foreign};

/// A resolved reference into the runtime-provided WebAssembly import registry.
pub struct Import {
	/// The WebAssembly import namespace (e.g. `"wasi_snapshot_preview1"`).
	pub namespace: Arc<str>,
	/// The WebAssembly import identifier within the namespace (e.g. `"fd_write"`).
	pub identifier: Arc<str>,
}

impl Import {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 1;
	/// The port index for the imported value.
	pub const RESULT_PORT: u16 = 0;

	/// Adds an `Import` node to the graph and returns the imported value.
	pub fn add_into(nodes: &mut Vec<Node>, namespace: Arc<str>, identifier: Arc<str>) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::Foreign(Box::new(Self {
			namespace,
			identifier,
		}));

		nodes.push(node);

		Link(id, Self::RESULT_PORT)
	}
}

impl Foreign for Import {
	fn identifier(&self) -> &'static str {
		"Import"
	}

	fn result_count(&self) -> u16 {
		Self::RESULT_COUNT
	}
}

/// A write into the runtime-provided WebAssembly export table under a named identifier.
pub struct Export {
	/// The WebAssembly export identifier (e.g. `"memory"`).
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
	pub fn add_into(nodes: &mut Vec<Node>, identifier: Arc<str>, value: Link) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::Foreign(Box::new(Self { identifier, value }));

		nodes.push(node);

		Link(id, Self::STATE_PORT)
	}
}

impl Foreign for Export {
	fn identifier(&self) -> &'static str {
		"Export"
	}

	fn result_count(&self) -> u16 {
		Self::RESULT_COUNT
	}

	fn forwarded_operand(&self, port: u16) -> Option<Link> {
		match port {
			Self::STATE_PORT => Some(self.value),
			_ => None,
		}
	}

	fn for_each_outer(&self, handler: &mut dyn FnMut(Link)) {
		handler(self.value);
	}

	fn for_each_mut_outer(&mut self, handler: &mut dyn FnMut(&mut Link)) {
		handler(&mut self.value);
	}
}
