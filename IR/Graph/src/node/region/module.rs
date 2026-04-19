//! The top-level module region and its boundary nodes.

use alloc::sync::{Arc, Weak};

use parking_lot::Mutex;

use crate::{Link, Node};

/// An exported symbol.
#[derive(Clone)]
pub struct Export {
	/// The export name.
	pub identifier: Arc<str>,
	/// The exported value link.
	pub reference: Link,
}

/// A module region.
pub struct Module {
	/// The nodes in this region.
	pub nodes: Vec<Node>,
}

impl Module {
	/// Node index of the arguments boundary node.
	pub const ARGUMENTS_ID: u32 = 0;

	/// Creates a new module region.
	pub fn create<F>(initializer: F) -> Arc<Mutex<Self>>
	where
		F: FnOnce(&mut Vec<Node>, u32) -> (Link, Vec<Export>),
	{
		let create = |weak: &Weak<Mutex<Self>>| {
			let mut nodes = Vec::new();

			let module_arguments = Arguments::add_into(&mut nodes, Weak::clone(weak));
			let (state, exports) = initializer(&mut nodes, module_arguments);

			Results::add_into(&mut nodes, Weak::clone(weak), state, exports);

			Mutex::new(Self { nodes })
		};

		Arc::new_cyclic(create)
	}

	/// Returns the index of the results boundary node.
	#[must_use]
	pub fn results_index(&self) -> usize {
		self.nodes
			.iter()
			.rposition(|node| matches!(node, Node::ModuleResults(_)))
			.unwrap_or_else(|| unreachable!())
	}

	/// Returns a reference to the results boundary node.
	#[must_use]
	pub fn results(&self) -> &Results {
		if let Node::ModuleResults(results) = &self.nodes[self.results_index()] {
			results
		} else {
			unreachable!()
		}
	}

	/// Returns a mutable reference to the results boundary node.
	pub fn results_mut(&mut self) -> &mut Results {
		let index = self.results_index();

		if let Node::ModuleResults(results) = &mut self.nodes[index] {
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

/// The boundary arguments node for a module region.
pub struct Arguments {
	/// The parent module.
	pub parent: Weak<Mutex<Module>>,
}

impl Arguments {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the environment.
	pub const ENVIRONMENT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a module arguments boundary node to the region.
	pub fn add_into(nodes: &mut Vec<Node>, parent: Weak<Mutex<Module>>) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::ModuleArguments(Self { parent });

		nodes.push(node);

		id
	}
}

/// The boundary results node for a module region.
pub struct Results {
	/// The parent module.
	pub parent: Weak<Mutex<Module>>,
	/// The final state link.
	pub state: Link,
	/// The exported symbols.
	pub exports: Vec<Export>,
}

impl Results {
	/// Adds a module results boundary node to the region.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		parent: Weak<Mutex<Module>>,
		state: Link,
		exports: Vec<Export>,
	) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::ModuleResults(Self {
			parent,
			state,
			exports,
		});

		nodes.push(node);

		id
	}

	pub(crate) fn for_each_outer<H: FnMut(Link)>(&self, mut handler: H) {
		handler(self.state);

		for export in &self.exports {
			handler(export.reference);
		}
	}

	pub(crate) fn for_each_mut_outer<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		handler(&mut self.state);

		for export in &mut self.exports {
			handler(&mut export.reference);
		}
	}
}
