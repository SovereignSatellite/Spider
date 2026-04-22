//! The theta-like repeat region and its boundary nodes.

#![expect(
	unused_variables,
	reason = "macro-generated visitors may not consume every field"
)]

use alloc::sync::{Arc, Weak};

use parking_lot::Mutex;

use crate::{Link, Node};

/// A repeat (loop) region.
pub struct Repeat {
	/// The argument links.
	pub arguments: Vec<Link>,
	/// The nodes in this region.
	pub nodes: Vec<Node>,
}

impl Repeat {
	/// Node index of the arguments boundary node.
	pub const ARGUMENTS_ID: u32 = 0;

	/// Creates a new repeat region.
	pub fn create<F>(arguments: Vec<Link>, initializer: F) -> Arc<Mutex<Self>>
	where
		F: FnOnce(&mut Vec<Node>, u32) -> (Vec<Link>, Link),
	{
		let create = |weak: &Weak<Mutex<Self>>| {
			let mut nodes = Vec::new();

			let repeat_arguments = Arguments::add_into(&mut nodes, Weak::clone(weak));
			let (sources, condition) = initializer(&mut nodes, repeat_arguments);

			Results::add_into(&mut nodes, Weak::clone(weak), sources, condition);

			Mutex::new(Self { arguments, nodes })
		};

		Arc::new_cyclic(create)
	}

	/// Adds a repeat region node to the graph.
	pub fn add_into<F>(nodes: &mut Vec<Node>, arguments: Vec<Link>, initializer: F) -> u32
	where
		F: FnOnce(&mut Vec<Node>, u32) -> (Vec<Link>, Link),
	{
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::Repeat(Self::create(arguments, initializer));

		nodes.push(node);

		id
	}

	/// Returns the number of arguments.
	#[must_use]
	pub fn argument_count(&self) -> u16 {
		self.arguments
			.len()
			.try_into()
			.unwrap_or_else(|_| unreachable!())
	}

	/// Returns the number of output ports.
	#[must_use]
	pub fn result_count(&self) -> u16 {
		self.results().argument_count()
	}

	/// Visits each outer link (arguments).
	pub fn for_each_outer<H: FnMut(Link)>(&self, mut handler: H) {
		for &link in &self.arguments {
			handler(link);
		}
	}

	/// Mutably visits each outer link (arguments).
	pub fn for_each_mut_outer<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		for link in &mut self.arguments {
			handler(link);
		}
	}

	/// Returns the index of the results boundary node.
	#[must_use]
	pub fn results_index(&self) -> usize {
		self.nodes
			.iter()
			.rposition(|node| matches!(node, Node::RepeatResults(_)))
			.unwrap_or_else(|| unreachable!())
	}

	/// Returns a reference to the results boundary node.
	#[must_use]
	pub fn results(&self) -> &Results {
		if let Node::RepeatResults(results) = &self.nodes[self.results_index()] {
			results
		} else {
			unreachable!()
		}
	}

	/// Returns a mutable reference to the results boundary node.
	pub fn results_mut(&mut self) -> &mut Results {
		let index = self.results_index();

		if let Node::RepeatResults(results) = &mut self.nodes[index] {
			results
		} else {
			unreachable!()
		}
	}

	/// Returns the ids of the arguments and results boundary nodes.
	#[must_use]
	pub fn roots(&self) -> (u32, u32) {
		let results = u32::try_from(self.results_index()).unwrap_or_else(|_| unreachable!());

		(Self::ARGUMENTS_ID, results)
	}
}

/// The boundary arguments node for a repeat region.
pub struct Arguments {
	/// The parent repeat.
	pub parent: Weak<Mutex<Repeat>>,
}

impl Arguments {
	/// Adds a repeat arguments boundary node to the region.
	pub fn add_into(nodes: &mut Vec<Node>, parent: Weak<Mutex<Repeat>>) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::RepeatArguments(Self { parent });

		nodes.push(node);

		id
	}

	/// Returns the number of output ports.
	#[must_use]
	pub fn result_count(&self) -> u16 {
		let repeat = self.parent.upgrade().unwrap_or_else(|| unreachable!());

		repeat.lock().argument_count()
	}
}

/// The boundary results node for a repeat region.
pub struct Results {
	/// The parent repeat.
	pub parent: Weak<Mutex<Repeat>>,
	/// The result value links fed back to the loop or out.
	pub sources: Vec<Link>,
	/// The loop continuation condition.
	pub condition: Link,
}

impl Results {
	/// Adds a repeat results boundary node to the region.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		parent: Weak<Mutex<Repeat>>,
		sources: Vec<Link>,
		condition: Link,
	) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::RepeatResults(Self {
			parent,
			sources,
			condition,
		});

		nodes.push(node);

		id
	}

	/// Returns the number of result values.
	#[must_use]
	pub fn argument_count(&self) -> u16 {
		self.sources
			.len()
			.try_into()
			.unwrap_or_else(|_| unreachable!())
	}

	handle_sources!((parent, ignore), (sources, link_list), (condition, link));
}
