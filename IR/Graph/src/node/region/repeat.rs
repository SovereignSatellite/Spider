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

	/// Creates a detached repeat region.
	#[must_use = "detached regions are dropped unless the returned handle is retained"]
	pub fn create<F>(arguments: Vec<Link>, initializer: F) -> Arc<Mutex<Self>>
	where
		F: FnOnce(&mut Vec<Node>, u32) -> (Vec<Link>, Link),
	{
		let create = |weak: &Weak<Mutex<Self>>| {
			let mut nodes = Vec::new();

			let Ok(argument_count) = arguments.len().try_into() else {
				unreachable!()
			};

			let repeat_arguments =
				Arguments::add_into(&mut nodes, Weak::clone(weak), argument_count);
			let (sources, condition) = initializer(&mut nodes, repeat_arguments);

			Results::add_into(&mut nodes, Weak::clone(weak), sources, condition);

			Mutex::new(Self { arguments, nodes })
		};

		Arc::new_cyclic(create)
	}

	/// Adds a repeat region node and returns its node identifier.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into<F>(nodes: &mut Vec<Node>, arguments: Vec<Link>, initializer: F) -> u32
	where
		F: FnOnce(&mut Vec<Node>, u32) -> (Vec<Link>, Link),
	{
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
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

	/// Updates the argument boundary arity.
	pub fn set_argument_count(&mut self, argument_count: u16) {
		self.arguments_mut().result_count = argument_count;
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

	/// Returns a reference to the arguments boundary node.
	#[must_use]
	pub fn arguments(&self) -> &Arguments {
		if let Node::RepeatArguments(arguments) = &self.nodes[Self::ARGUMENTS_ID as usize] {
			arguments
		} else {
			unreachable!()
		}
	}

	/// Returns a mutable reference to the arguments boundary node.
	pub fn arguments_mut(&mut self) -> &mut Arguments {
		if let Node::RepeatArguments(arguments) = &mut self.nodes[Self::ARGUMENTS_ID as usize] {
			arguments
		} else {
			unreachable!()
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
		let Ok(results) = u32::try_from(self.results_index()) else {
			unreachable!()
		};

		(Self::ARGUMENTS_ID, results)
	}
}

/// The boundary arguments node for a repeat region.
pub struct Arguments {
	/// The parent repeat.
	pub parent: Weak<Mutex<Repeat>>,
	/// The number of output ports.
	pub result_count: u16,
}

impl Arguments {
	fn add_into(nodes: &mut Vec<Node>, parent: Weak<Mutex<Repeat>>, result_count: u16) -> u32 {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::RepeatArguments(Self {
			parent,
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
	fn add_into(
		nodes: &mut Vec<Node>,
		parent: Weak<Mutex<Repeat>>,
		sources: Vec<Link>,
		condition: Link,
	) {
		let node = Node::RepeatResults(Self {
			parent,
			sources,
			condition,
		});

		nodes.push(node);
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
