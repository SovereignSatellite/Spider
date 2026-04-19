//! JSON printer for data flow graphs.

extern crate alloc;

use alloc::sync::Arc;
use core::str;
use std::io::{Result, Write};

use parking_lot::Mutex;

use ir_graph::{
	Node,
	region::{Function, Match, Module, Repeat},
};

use self::{color::Color, interner::Interner, names::Names};

mod color;
mod interner;
mod label;
mod names;

fn write_integers(values: &[u32], output: &mut dyn Write) -> Result<()> {
	let mut values = values.iter();

	if let Some(first) = values.next() {
		write!(output, "{first}")?;

		values.try_for_each(|value| write!(output, ", {value}"))?;
	}

	Ok(())
}

fn write_strings(values: &[Arc<str>], output: &mut dyn Write) -> Result<()> {
	let mut values = values.iter();

	if let Some(first) = values.next() {
		let first = first.as_bytes().escape_ascii();

		write!(output, "\"{first}\"")?;

		values.try_for_each(|value| {
			let value = value.as_bytes().escape_ascii();

			write!(output, ", \"{value}\"")
		})?;
	}

	Ok(())
}

/// A JSON printer for data flow graphs.
pub struct JsonPrinter {
	subgraphs: Vec<u32>,
	nodes: Vec<u32>,
	edges: Vec<u32>,

	scratch: Vec<u8>,
	interner: Interner,

	names: Names,
}

impl JsonPrinter {
	/// Creates a new JSON printer.
	#[must_use]
	pub fn new() -> Self {
		Self {
			subgraphs: Vec::new(),
			nodes: Vec::new(),
			edges: Vec::new(),

			scratch: Vec::new(),
			interner: Interner::new(),

			names: Names::new(),
		}
	}

	fn clear(&mut self) {
		self.subgraphs.clear();
		self.nodes.clear();
		self.edges.clear();
		self.interner.clear();
		self.names.clear();
	}

	fn get_node_label(&mut self, node: &Node) -> u32 {
		let name = label::get_static(node).unwrap_or_else(|| {
			self.scratch.clear();

			label::write(node, &mut self.scratch).unwrap();
			str::from_utf8(&self.scratch).unwrap()
		});

		self.interner.resolve(name)
	}

	fn record_module(&mut self, id: u32) {
		let name = self.interner.resolve("Module");
		let color = self.interner.resolve(Color::Brown.as_string());

		self.nodes.push(id);
		self.nodes.push(name);
		self.nodes.push(color);
	}

	fn record_node(&mut self, node: &Node, id: u32) {
		let name = self.get_node_label(node);
		let color = self
			.interner
			.resolve(Color::from_reference(node).as_string());

		self.nodes.push(id);
		self.nodes.push(name);
		self.nodes.push(color);
	}

	fn record_subgraph(&mut self, parent: u32, entry: u32, exit: u32) {
		self.subgraphs.push(parent);
		self.subgraphs.push(entry);
		self.subgraphs.push(exit);
	}

	fn handle_module(&mut self, module: &Arc<Mutex<Module>>, parent: u32) {
		let guard = module.lock();
		let (entry, exit) = self.handle_nodes(&guard.nodes);

		drop(guard);

		self.record_subgraph(parent, entry, exit);
	}

	fn handle_function(&mut self, region: &Arc<Mutex<Function>>, parent: u32) {
		let guard = region.lock();
		let (entry, exit) = self.handle_nodes(&guard.nodes);

		drop(guard);

		self.record_subgraph(parent, entry, exit);
	}

	fn handle_match(&mut self, region: &Arc<Mutex<Match>>, parent: u32) {
		let guard = region.lock();

		for branch in &guard.branches {
			let branch = branch.lock();
			let (entry, exit) = self.handle_nodes(&branch.nodes);

			drop(branch);

			self.record_subgraph(parent, entry, exit);
		}
	}

	fn handle_repeat(&mut self, region: &Arc<Mutex<Repeat>>, parent: u32) {
		let guard = region.lock();
		let (entry, exit) = self.handle_nodes(&guard.nodes);

		drop(guard);

		self.record_subgraph(parent, entry, exit);
	}

	#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
	fn assign_ids(&mut self, nodes: &[Node]) {
		for node in nodes {
			let global = self.names.assign();

			match node {
				Node::Function(region) => self.handle_function(region, global),
				Node::Match(region) => self.handle_match(region, global),
				Node::Repeat(region) => self.handle_repeat(region, global),

				Node::ModuleArguments(_)
				| Node::ModuleResults(_)
				| Node::FunctionCaptures(_)
				| Node::FunctionArguments(_)
				| Node::FunctionResults(_)
				| Node::BranchArguments(_)
				| Node::BranchResults(_)
				| Node::RepeatArguments(_)
				| Node::RepeatResults(_)
				| Node::Import(_)
				| Node::Foreign(_)
				| Node::Trap
				| Node::Null
				| Node::I32(_)
				| Node::I64(_)
				| Node::F32(_)
				| Node::F64(_)
				| Node::Identity(_)
				| Node::Fence(_)
				| Node::Apply(_)
				| Node::RefIsNull(_)
				| Node::IntegerUnaryOperation(_)
				| Node::IntegerBinaryOperation(_)
				| Node::IntegerCompareOperation(_)
				| Node::IntegerNarrow(_)
				| Node::IntegerWiden(_)
				| Node::IntegerSignExtend(_)
				| Node::IntegerConvertToNumber(_)
				| Node::IntegerTransmuteToNumber(_)
				| Node::NumberUnaryOperation(_)
				| Node::NumberBinaryOperation(_)
				| Node::NumberCompareOperation(_)
				| Node::NumberNarrow(_)
				| Node::NumberWiden(_)
				| Node::NumberTruncateToInteger(_)
				| Node::NumberTransmuteToInteger(_)
				| Node::MutableNew(_)
				| Node::MutableGet(_)
				| Node::MutableSet(_)
				| Node::TableNew(_)
				| Node::TableGet(_)
				| Node::TableSet(_)
				| Node::TableSize(_)
				| Node::TableGrow(_)
				| Node::TableFill(_)
				| Node::TableCopy(_)
				| Node::TableDrop(_)
				| Node::MemoryNew(_)
				| Node::MemoryLoad(_)
				| Node::MemoryStore(_)
				| Node::MemorySize(_)
				| Node::MemoryGrow(_)
				| Node::MemoryFill(_)
				| Node::MemoryCopy(_)
				| Node::MemoryDrop(_) => {}
			}
		}
	}

	fn find_nodes(&mut self, nodes: &[Node]) {
		for (local, node) in nodes.iter().enumerate() {
			let global = self.names.resolve(local);

			self.record_node(node, global);
		}
	}

	fn find_edges(&mut self, nodes: &[Node]) {
		for (local, node) in nodes.iter().enumerate() {
			let global = self.names.resolve(local);
			let mut port = 0_u32;

			node.for_each_outer(|link| {
				let source = self.names.resolve(link.0.try_into().unwrap());

				self.edges.push(source);
				self.edges.push(link.1.into());
				self.edges.push(global);
				self.edges.push(port);

				port += 1;
			});
		}
	}

	fn handle_nodes(&mut self, nodes: &[Node]) -> (u32, u32) {
		self.names.enter_scope();
		self.assign_ids(nodes);

		let entry = self.names.entry();
		let exit = self.names.exit();

		self.find_nodes(nodes);
		self.find_edges(nodes);
		self.names.leave_scope();

		(entry, exit)
	}

	fn print_all_fields(&self, out: &mut dyn Write) -> Result<()> {
		write!(out, r#"{{"subgraphs":["#)?;
		write_integers(&self.subgraphs, out)?;
		write!(out, r#"],"nodes":["#)?;
		write_integers(&self.nodes, out)?;
		write!(out, r#"],"edges":["#)?;
		write_integers(&self.edges, out)?;
		write!(out, r#"],"strings":["#)?;
		write_strings(self.interner.list(), out)?;
		write!(out, "]}}")
	}

	/// Prints the module as JSON to the given writer.
	///
	/// # Errors
	///
	/// Returns an error if writing to the output fails.
	pub fn print(&mut self, module: &Arc<Mutex<Module>>, out: &mut dyn Write) -> Result<()> {
		self.clear();

		let parent = self.names.assign();

		self.record_module(parent);
		self.handle_module(module, parent);
		self.print_all_fields(out)
	}
}

impl Default for JsonPrinter {
	fn default() -> Self {
		Self::new()
	}
}
