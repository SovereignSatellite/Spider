//! The lambda-like function region and its boundary nodes.

#![expect(
	unused_variables,
	reason = "macro-generated visitors may not consume every field"
)]

use alloc::sync::{Arc, Weak};

use parking_lot::Mutex;

use crate::{Link, Node};

/// A function region.
pub struct Function {
	/// The number of argument ports exposed by the body's [`Arguments`] node.
	pub argument_count: u16,
	/// The nodes in this region.
	pub nodes: Vec<Node>,
}

impl Function {
	/// Node index of the arguments boundary node.
	pub const ARGUMENTS_ID: u32 = 0;

	/// Creates a new function region.
	pub fn create<F>(argument_count: u16, initializer: F) -> Arc<Mutex<Self>>
	where
		F: FnOnce(&mut Vec<Node>, u32) -> Vec<Link>,
	{
		let create = |weak: &Weak<Mutex<Self>>| {
			let mut nodes = Vec::new();

			let function_arguments =
				Arguments::add_into(&mut nodes, Weak::clone(weak), argument_count);
			let sources = initializer(&mut nodes, function_arguments);

			Results::add_into(&mut nodes, Weak::clone(weak), sources);

			Mutex::new(Self {
				argument_count,
				nodes,
			})
		};

		Arc::new_cyclic(create)
	}

	/// Adds a function region node to the graph.
	pub fn add_into<F>(nodes: &mut Vec<Node>, argument_count: u16, initializer: F) -> Link
	where
		F: FnOnce(&mut Vec<Node>, u32) -> Vec<Link>,
	{
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::Function(Self::create(argument_count, initializer));

		nodes.push(node);

		Link(id, 0)
	}

	/// Returns the index of the results boundary node.
	#[must_use]
	pub fn results_index(&self) -> usize {
		self.nodes
			.iter()
			.rposition(|node| matches!(node, Node::FunctionResults(_)))
			.unwrap_or_else(|| unreachable!())
	}

	/// Returns a reference to the results boundary node.
	#[must_use]
	pub fn results(&self) -> &Results {
		if let Node::FunctionResults(results) = &self.nodes[self.results_index()] {
			results
		} else {
			unreachable!()
		}
	}

	/// Returns a mutable reference to the results boundary node.
	pub fn results_mut(&mut self) -> &mut Results {
		let index = self.results_index();

		if let Node::FunctionResults(results) = &mut self.nodes[index] {
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

/// The boundary arguments node for a function region.
pub struct Arguments {
	/// The parent function.
	pub parent: Weak<Mutex<Function>>,
	/// The number of output ports.
	pub result_count: u16,
}

impl Arguments {
	/// Adds a function arguments boundary node to the region.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		parent: Weak<Mutex<Function>>,
		result_count: u16,
	) -> u32 {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::FunctionArguments(Self {
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

/// The boundary results node for a function region.
pub struct Results {
	/// The parent function.
	pub parent: Weak<Mutex<Function>>,
	/// The result value links.
	pub sources: Vec<Link>,
}

impl Results {
	/// Adds a function results boundary node to the region.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		parent: Weak<Mutex<Function>>,
		sources: Vec<Link>,
	) -> u32 {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::FunctionResults(Self { parent, sources });

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
