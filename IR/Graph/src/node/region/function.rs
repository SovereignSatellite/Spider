//! The lambda-like function region and its boundary nodes.

#![expect(
	unused_variables,
	reason = "macro-generated visitors may not consume every field"
)]

use alloc::sync::{Arc, Weak};

use list::resizable::Resizable;
use parking_lot::Mutex;

use crate::{Link, Node};

/// Value types for function signatures.
#[derive(Clone, Copy)]
pub enum ValueType {
	/// A 32-bit integer.
	I32,
	/// A 64-bit integer.
	I64,
	/// A 32-bit float.
	F32,
	/// A 64-bit float.
	F64,

	/// A reference.
	Reference,
}

/// A function region.
pub struct Function {
	/// The argument types.
	pub argument_types: Resizable<ValueType, 15>,
	/// The result types.
	pub result_types: Resizable<ValueType, 15>,
	/// The closure captures.
	pub captures: Vec<Link>,
	/// The nodes in this region.
	pub nodes: Vec<Node>,
}

impl Function {
	/// Node index of the captures boundary node.
	pub const CAPTURES_ID: u32 = 0;
	/// Node index of the arguments boundary node.
	pub const ARGUMENTS_ID: u32 = 1;

	/// Creates a new function region.
	pub fn create<F>(
		argument_types: Resizable<ValueType, 15>,
		result_types: Resizable<ValueType, 15>,
		captures: Vec<Link>,
		initializer: F,
	) -> Arc<Mutex<Self>>
	where
		F: FnOnce(&mut Vec<Node>, u32, u32) -> Vec<Link>,
	{
		let create = |weak: &Weak<Mutex<Self>>| {
			let mut nodes = Vec::new();

			let function_captures = Captures::add_into(&mut nodes, Weak::clone(weak));
			let function_arguments = Arguments::add_into(&mut nodes, Weak::clone(weak));
			let sources = initializer(&mut nodes, function_captures, function_arguments);

			Results::add_into(&mut nodes, Weak::clone(weak), sources);

			Mutex::new(Self {
				argument_types,
				result_types,
				captures,
				nodes,
			})
		};

		Arc::new_cyclic(create)
	}

	/// Adds a function region node to the graph.
	pub fn add_into<F>(
		nodes: &mut Vec<Node>,
		argument_types: Resizable<ValueType, 15>,
		result_types: Resizable<ValueType, 15>,
		captures: Vec<Link>,
		initializer: F,
	) -> Link
	where
		F: FnOnce(&mut Vec<Node>, u32, u32) -> Vec<Link>,
	{
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::Function(Self::create(
			argument_types,
			result_types,
			captures,
			initializer,
		));

		nodes.push(node);

		Link(id, 0)
	}

	/// Returns the number of captures.
	#[must_use]
	pub fn capture_count(&self) -> u16 {
		self.captures
			.len()
			.try_into()
			.unwrap_or_else(|_| unreachable!())
	}

	/// Returns the number of arguments.
	#[must_use]
	pub fn argument_count(&self) -> u16 {
		self.argument_types
			.len()
			.try_into()
			.unwrap_or_else(|_| unreachable!())
	}

	/// Visits each outer link (captures).
	pub fn for_each_outer<H: FnMut(Link)>(&self, mut handler: H) {
		for &link in &self.captures {
			handler(link);
		}
	}

	/// Mutably visits each outer link (captures).
	pub fn for_each_mut_outer<H: FnMut(&mut Link)>(&mut self, mut handler: H) {
		for link in &mut self.captures {
			handler(link);
		}
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

	pub(super) fn for_each_root<H: FnMut(u32)>(&self, mut handler: H) {
		let result = u32::try_from(self.results_index()).unwrap_or_else(|_| unreachable!());

		handler(Self::CAPTURES_ID);
		handler(Self::ARGUMENTS_ID);
		handler(result);
	}
}

/// The boundary captures node for a function region.
pub struct Captures {
	/// The parent function.
	pub parent: Weak<Mutex<Function>>,
}

impl Captures {
	/// Adds a function captures boundary node to the region.
	pub fn add_into(nodes: &mut Vec<Node>, parent: Weak<Mutex<Function>>) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::FunctionCaptures(Self { parent });

		nodes.push(node);

		id
	}

	/// Returns the number of output ports.
	#[must_use]
	pub fn result_count(&self) -> u16 {
		let function = self.parent.upgrade().unwrap_or_else(|| unreachable!());

		function.lock().capture_count()
	}
}

/// The boundary arguments node for a function region.
pub struct Arguments {
	/// The parent function.
	pub parent: Weak<Mutex<Function>>,
}

impl Arguments {
	/// Adds a function arguments boundary node to the region.
	pub fn add_into(nodes: &mut Vec<Node>, parent: Weak<Mutex<Function>>) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::FunctionArguments(Self { parent });

		nodes.push(node);

		id
	}

	/// Returns the number of output ports.
	#[must_use]
	pub fn result_count(&self) -> u16 {
		let function = self.parent.upgrade().unwrap_or_else(|| unreachable!());

		function.lock().argument_count()
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
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
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
