//! Small cross-cutting node types: function application, pass-through,
//! ordering fence, and reference-null check.

#![expect(
	unused_variables,
	reason = "macro-generated visitors may not consume every field"
)]

use list::resizable::Resizable;

use crate::{Link, Node};

/// A node that passes through its sources unchanged.
#[derive(Clone)]
pub struct Identity {
	/// The source links.
	pub sources: Resizable<Link, 4>,
}

impl Identity {
	/// Adds an identity node and returns its node identifier.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, sources: Resizable<Link, 4>) -> u32 {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::Identity(Self { sources });

		nodes.push(node);

		id
	}

	/// Returns the number of output ports.
	#[must_use]
	pub fn result_count(&self) -> u16 {
		self.sources
			.len()
			.try_into()
			.unwrap_or_else(|_| unreachable!())
	}

	handle_sources!((sources, link_list));
}

/// A fence node that orders its sources.
#[derive(Clone)]
pub struct Fence {
	/// The ordered source links.
	pub sources: Resizable<Link, 4>,
}

impl Fence {
	/// Adds a fence node and returns its node identifier.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, sources: Resizable<Link, 4>) -> u32 {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::Fence(Self { sources });

		nodes.push(node);

		id
	}

	/// Returns the number of output ports.
	#[must_use]
	pub fn result_count(&self) -> u16 {
		self.sources
			.len()
			.try_into()
			.unwrap_or_else(|_| unreachable!())
	}

	handle_sources!((sources, link_list));
}

/// A function application node.
#[derive(Clone)]
pub struct Apply {
	/// The function being applied.
	pub function: Link,
	/// The argument values passed to the function.
	pub arguments: Vec<Link>,
	/// The number of results produced.
	pub result_count: u16,
}

impl Apply {
	/// Adds a function application node and returns its node identifier.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(
		nodes: &mut Vec<Node>,
		function: Link,
		arguments: Vec<Link>,
		result_count: u16,
	) -> u32 {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::Apply(Self {
			function,
			arguments,
			result_count,
		});

		nodes.push(node);

		id
	}

	/// Returns the number of output ports.
	#[must_use]
	pub const fn result_count(&self) -> u16 {
		self.result_count
	}

	handle_sources!(
		(function, link),
		(arguments, link_list),
		(result_count, ignore)
	);
}

/// A reference null check node.
#[derive(Clone, Copy)]
pub struct RefIsNull {
	/// The reference being checked.
	pub source: Link,
}

impl RefIsNull {
	/// Adds a reference null check node to the graph.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::RefIsNull(Self { source });

		nodes.push(node);

		Link(id, 0)
	}

	handle_sources!((source, link));
}
