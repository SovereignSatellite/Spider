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
	pub fn create<F>(
		parent: Weak<Mutex<Match>>,
		argument_count: u16,
		initializer: F,
	) -> Arc<Mutex<Self>>
	where
		F: FnOnce(&mut Vec<Node>, u32) -> Vec<Link>,
	{
		let create = |weak: &Weak<Mutex<Self>>| {
			let mut nodes = Vec::new();

			let branch_arguments =
				Arguments::add_into(&mut nodes, Weak::clone(weak), argument_count);
			let sources = initializer(&mut nodes, branch_arguments);

			Results::add_into(&mut nodes, Weak::clone(weak), sources);

			Mutex::new(Self { nodes, parent })
		};

		Arc::new_cyclic(create)
	}

	/// Returns the number of arguments.
	#[must_use]
	pub fn argument_count(&self) -> u16 {
		self.arguments().result_count()
	}

	/// Updates the argument boundary arity.
	pub fn set_argument_count(&mut self, result_count: u16) {
		self.arguments_mut().result_count = result_count;
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

	/// Returns a reference to the arguments boundary node.
	#[must_use]
	pub fn arguments(&self) -> &Arguments {
		if let Node::BranchArguments(arguments) = &self.nodes[Self::ARGUMENTS_ID as usize] {
			arguments
		} else {
			unreachable!()
		}
	}

	/// Returns a mutable reference to the arguments boundary node.
	pub fn arguments_mut(&mut self) -> &mut Arguments {
		if let Node::BranchArguments(arguments) = &mut self.nodes[Self::ARGUMENTS_ID as usize] {
			arguments
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

	/// Returns the ids of the arguments and results boundary nodes.
	#[must_use]
	pub fn roots(&self) -> (u32, u32) {
		let results = u32::try_from(self.results_index()).unwrap_or_else(|_| unreachable!());

		(Self::ARGUMENTS_ID, results)
	}
}

/// The boundary arguments node for a branch region.
pub struct Arguments {
	/// The parent branch.
	pub parent: Weak<Mutex<Branch>>,
	/// The number of output ports.
	pub result_count: u16,
}

impl Arguments {
	/// Adds a branch arguments boundary node to the region.
	pub fn add_into(nodes: &mut Vec<Node>, parent: Weak<Mutex<Branch>>, result_count: u16) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::BranchArguments(Self {
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
