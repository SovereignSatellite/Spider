//! Native structured data, aggregates and field extraction.

#![expect(
	unused_variables,
	reason = "macro-generated visitors may not consume every field"
)]

use crate::{Link, Node};

/// An aggregate value composed from a list of field links.
pub struct Aggregate {
	/// The field value links, in order.
	pub fields: Vec<Link>,
}

impl Aggregate {
	/// Adds an aggregate node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, fields: Vec<Link>) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::Aggregate(Self { fields });

		nodes.push(node);

		Link(id, 0)
	}

	handle_sources!((fields, link_list));
}

/// A field extraction from an aggregate source.
#[derive(Clone, Copy)]
pub struct Extract {
	/// The aggregate source value.
	pub source: Link,
	/// The zero-based field index.
	pub index: u32,
}

impl Extract {
	/// Adds an extract node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, source: Link, index: u32) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::Extract(Self { source, index });

		nodes.push(node);

		Link(id, 0)
	}

	handle_sources!((source, link), (index, ignore));
}
