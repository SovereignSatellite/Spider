//! Turing machine tape IO operations as concrete `Foreign` types.

#![no_std]

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};

use ir_graph::{Link, Node, foreign::Foreign};

/// Reads one character from the tape, yielding a new state token and the character.
#[derive(Clone, Copy)]
pub struct Ask {
	/// The prior IO state token.
	pub state: Link,
}

impl Ask {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the character read from the tape.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the outgoing state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds an `Ask` node to the graph and returns (character, state).
	pub fn add_into(nodes: &mut Vec<Node>, state: Link) -> (Link, Link) {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::Foreign(Box::new(Self { state }));

		nodes.push(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl Foreign for Ask {
	fn identifier(&self) -> &'static str {
		"TuringAsk"
	}

	fn result_count(&self) -> u16 {
		Self::RESULT_COUNT
	}

	fn for_each_outer(&self, handler: &mut dyn FnMut(Link)) {
		handler(self.state);
	}

	fn for_each_mut_outer(&mut self, handler: &mut dyn FnMut(&mut Link)) {
		handler(&mut self.state);
	}
}

/// Writes one character to the tape, yielding a new state token.
#[derive(Clone, Copy)]
pub struct Tell {
	/// The prior IO state token.
	pub state: Link,
	/// The character to write.
	pub character: Link,
}

impl Tell {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 1;
	/// The port index for the outgoing state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a `Tell` node to the graph and returns the outgoing state token.
	pub fn add_into(nodes: &mut Vec<Node>, state: Link, character: Link) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::Foreign(Box::new(Self { state, character }));

		nodes.push(node);

		Link(id, Self::STATE_PORT)
	}
}

impl Foreign for Tell {
	fn identifier(&self) -> &'static str {
		"TuringTell"
	}

	fn result_count(&self) -> u16 {
		Self::RESULT_COUNT
	}

	fn for_each_outer(&self, handler: &mut dyn FnMut(Link)) {
		handler(self.state);
		handler(self.character);
	}

	fn for_each_mut_outer(&mut self, handler: &mut dyn FnMut(&mut Link)) {
		handler(&mut self.state);
		handler(&mut self.character);
	}
}
