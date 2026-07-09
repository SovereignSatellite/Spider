//! Eliminates dead nodes and topologically sorts the survivors.

use core::mem;

use ir_graph::{Node, Region};

/// Eliminate unreachable nodes and compact the remainder in dependency order.
pub struct TopologicalCompactor {
	nodes: Vec<Node>,
	ids: Vec<u32>,
}

impl TopologicalCompactor {
	/// Creates a new compactor.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			nodes: Vec::new(),
			ids: Vec::new(),
		}
	}

	fn handle_node(&mut self, nodes: &mut Vec<Node>, id: u32) {
		let id = usize::try_from(id).unwrap();

		if self.ids[id] != u32::MAX {
			return;
		}

		let node = mem::take(&mut nodes[id]);

		stacker::maybe_grow(0x1_0000, 0x10_0000, || {
			node.for_each_outer(|link| self.handle_node(nodes, link.0));
		});

		self.ids[id] = self.nodes.len().try_into().unwrap();

		self.nodes.push(node);
	}

	fn handle_nodes(&mut self, nodes: &mut Vec<Node>, arguments: u32, results: u32) {
		self.nodes.clear();

		self.ids.clear();
		self.ids.resize(nodes.len(), u32::MAX);

		self.handle_node(nodes, arguments);
		self.handle_node(nodes, results);

		mem::swap(nodes, &mut self.nodes);
	}

	fn remap_link(&self, link: &mut ir_graph::Link) {
		let id = usize::try_from(link.0).unwrap();

		link.0 = self.ids[id];
	}

	fn handle_edges(&self, nodes: &mut [Node]) {
		for node in nodes.iter_mut() {
			node.for_each_mut_outer(|link| self.remap_link(link));
		}
	}

	/// Compacts a single region in place.
	///
	/// The arguments boundary is visited before the results boundary so it
	/// keeps its well-known position at the start of the region.
	pub fn run(&mut self, region: &mut Region) {
		let (arguments, results) = region.roots();
		let nodes = region.nodes_mut();

		self.handle_nodes(nodes, arguments, results);
		self.handle_edges(nodes);
	}
}

impl Default for TopologicalCompactor {
	fn default() -> Self {
		Self::new()
	}
}
