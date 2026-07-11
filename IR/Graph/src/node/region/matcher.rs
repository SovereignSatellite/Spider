//! The gamma-like match region that dispatches to branch sub-regions.

use alloc::sync::{Arc, Weak};

use parking_lot::Mutex;

use crate::{Link, Node, node::region::branch::Branch};

/// A match (conditional) region.
pub struct Match {
	/// The argument links.
	pub arguments: Vec<Link>,
	/// The condition link.
	pub condition: Link,
	/// The branch regions.
	pub branches: Vec<Arc<Mutex<Branch>>>,
}

impl Match {
	/// Creates a detached match region.
	#[must_use = "detached regions are dropped unless the returned handle is retained"]
	pub fn create<F>(arguments: Vec<Link>, condition: Link, initializer: F) -> Arc<Mutex<Self>>
	where
		F: FnOnce(&Weak<Mutex<Self>>, u16) -> Vec<Arc<Mutex<Branch>>>,
	{
		let create = |weak: &Weak<Mutex<Self>>| {
			let Ok(argument_count) = arguments.len().try_into() else {
				unreachable!()
			};

			let branches = initializer(weak, argument_count);

			Mutex::new(Self {
				arguments,
				condition,
				branches,
			})
		};

		Arc::new_cyclic(create)
	}

	/// Adds a match region node and returns its node identifier.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into<F>(
		nodes: &mut Vec<Node>,
		arguments: Vec<Link>,
		condition: Link,
		initializer: F,
	) -> u32
	where
		F: FnOnce(&Weak<Mutex<Self>>, u16) -> Vec<Arc<Mutex<Branch>>>,
	{
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::Match(Self::create(arguments, condition, initializer));

		nodes.push(node);

		id
	}

	/// Creates a detached if-else match region.
	#[must_use = "detached regions are dropped unless the returned handle is retained"]
	pub fn create_if<F, T>(
		arguments: Vec<Link>,
		condition: Link,
		on_false: F,
		on_true: T,
	) -> Arc<Mutex<Self>>
	where
		F: FnOnce(&mut Vec<Node>, u32) -> Vec<Link>,
		T: FnOnce(&mut Vec<Node>, u32) -> Vec<Link>,
	{
		Self::create(arguments, condition, |parent, argument_count| {
			vec![
				Branch::create(Weak::clone(parent), argument_count, on_false),
				Branch::create(Weak::clone(parent), argument_count, on_true),
			]
		})
	}

	/// Adds an if-else structure and returns its node identifier.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_if_into<F, T>(
		nodes: &mut Vec<Node>,
		arguments: Vec<Link>,
		condition: Link,
		on_false: F,
		on_true: T,
	) -> u32
	where
		F: FnOnce(&mut Vec<Node>, u32) -> Vec<Link>,
		T: FnOnce(&mut Vec<Node>, u32) -> Vec<Link>,
	{
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::Match(Self::create_if(arguments, condition, on_false, on_true));

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
		self.branches
			.first()
			.map_or(0, |branch| branch.lock().result_count())
	}

	/// Visits each outer link (arguments, condition).
	pub fn for_each_outer<H: FnMut(Link)>(&self, mut handler: H) {
		for &link in &self.arguments {
			handler(link);
		}

		handler(self.condition);
	}

	/// Mutably visits each outer link (arguments, condition).
	pub fn for_each_mut_outer<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		for link in &mut self.arguments {
			handler(link);
		}

		handler(&mut self.condition);
	}
}
