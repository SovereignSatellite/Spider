//! A branch region — one arm of a match dispatch.

#![expect(
	unused_variables,
	reason = "macro-generated visitors may not consume every field"
)]

use alloc::sync::{Arc, Weak};

use parking_lot::Mutex;

use crate::{Link, Node, node::region::matcher::Match};

/// A branch region within a match.
pub struct Branch {
	/// The nodes in this region.
	pub nodes: Vec<Node>,
	/// The parent match.
	pub parent: Weak<Mutex<Match>>,
}

impl Branch {
	/// Node index of the arguments boundary node.
	pub const ARGUMENTS_ID: u32 = 0;

	/// Creates a new branch region.
	pub fn create<F>(parent: Weak<Mutex<Match>>, initializer: F) -> Arc<Mutex<Self>>
	where
		F: FnOnce(&mut Vec<Node>, u32) -> Vec<Link>,
	{
		let create = |weak: &Weak<Mutex<Self>>| {
			let mut nodes = Vec::new();

			let branch_arguments = Arguments::add_into(&mut nodes, Weak::clone(weak));
			let sources = initializer(&mut nodes, branch_arguments);

			Results::add_into(&mut nodes, Weak::clone(weak), sources);

			Mutex::new(Self { nodes, parent })
		};

		Arc::new_cyclic(create)
	}

	/// Returns the number of arguments.
	#[must_use]
	pub fn argument_count(&self) -> u16 {
		let matcher = self.parent.upgrade().unwrap_or_else(|| unreachable!());

		matcher.lock().argument_count()
	}

	/// Returns the number of output ports.
	#[must_use]
	pub fn result_count(&self) -> u16 {
		self.results().argument_count()
	}

	/// Returns the index of the results boundary node.
	#[must_use]
	pub fn results_index(&self) -> usize {
		self.nodes
			.iter()
			.rposition(|node| matches!(node, Node::BranchResults(_)))
			.unwrap_or_else(|| unreachable!())
	}

	/// Returns a reference to the results boundary node.
	#[must_use]
	pub fn results(&self) -> &Results {
		if let Node::BranchResults(results) = &self.nodes[self.results_index()] {
			results
		} else {
			unreachable!()
		}
	}

	/// Returns a mutable reference to the results boundary node.
	pub fn results_mut(&mut self) -> &mut Results {
		let index = self.results_index();

		if let Node::BranchResults(results) = &mut self.nodes[index] {
			results
		} else {
			unreachable!()
		}
	}

	pub(super) fn for_each_root<H: FnMut(u32)>(&self, mut handler: H) {
		let result = u32::try_from(self.results_index()).unwrap_or_else(|_| unreachable!());

		handler(Self::ARGUMENTS_ID);
		handler(result);
	}
}

/// The boundary arguments node for a branch region.
pub struct Arguments {
	/// The parent branch.
	pub parent: Weak<Mutex<Branch>>,
}

impl Arguments {
	/// Adds a branch arguments boundary node to the region.
	pub fn add_into(nodes: &mut Vec<Node>, parent: Weak<Mutex<Branch>>) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::BranchArguments(Self { parent });

		nodes.push(node);

		id
	}

	/// Returns the number of output ports.
	#[must_use]
	pub fn result_count(&self) -> u16 {
		let branch = self.parent.upgrade().unwrap_or_else(|| unreachable!());

		branch.lock().argument_count()
	}
}

/// The boundary results node for a branch region.
pub struct Results {
	/// The parent branch.
	pub parent: Weak<Mutex<Branch>>,
	/// The result value links.
	pub sources: Vec<Link>,
}

impl Results {
	/// Adds a branch results boundary node to the region.
	pub fn add_into(nodes: &mut Vec<Node>, parent: Weak<Mutex<Branch>>, sources: Vec<Link>) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::BranchResults(Self { parent, sources });

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

	handle_sources!((parent, ignore), (sources, link_list));
}
