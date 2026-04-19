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

		node.for_each_outer(|link| self.handle_node(nodes, link.0));

		self.ids[id] = self.nodes.len().try_into().unwrap();

		self.nodes.push(node);
	}

	fn handle_nodes(&mut self, nodes: &mut Vec<Node>, roots: &[u32]) {
		self.nodes.clear();

		self.ids.clear();
		self.ids.resize(nodes.len(), u32::MAX);

		for &root in roots {
			self.handle_node(nodes, root);
		}

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
	/// Roots are processed left to right. Boundary nodes appear first
	/// so they retain their well-known positions.
	pub fn run(&mut self, region: &mut Region) {
		let mut roots = [0_u32; 3];
		let mut count = 0;

		region.for_each_root(|id| {
			roots[count] = id;
			count += 1;
		});

		let nodes = region.nodes_mut();

		self.handle_nodes(nodes, &roots[..count]);
		self.handle_edges(nodes);
	}
}

impl Default for TopologicalCompactor {
	fn default() -> Self {
		Self::new()
	}
}
