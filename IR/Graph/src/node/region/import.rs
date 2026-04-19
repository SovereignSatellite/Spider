//! External imports.

#![expect(
	unused_variables,
	reason = "macro-generated visitors may not consume every field"
)]

use alloc::sync::Arc;

use crate::{Link, Node};

/// An external import node.
#[derive(Clone)]
pub struct Import {
	/// The environment link.
	pub environment: Link,
	/// The import namespace.
	pub namespace: Arc<str>,
	/// The import name.
	pub identifier: Arc<str>,
}

impl Import {
	/// Adds an import node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		environment: Link,
		namespace: Arc<str>,
		identifier: Arc<str>,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::Import(
			Self {
				environment,
				namespace,
				identifier,
			}
			.into(),
		);

		nodes.push(node);

		Link(id, 0)
	}

	handle_sources!(
		(environment, link),
		(namespace, ignore),
		(identifier, ignore)
	);
}
