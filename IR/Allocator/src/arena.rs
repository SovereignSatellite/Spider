//! The port arena: one entry per output port, laid out region by region.

use alloc::vec::Vec;

use ir_graph::{Link, Node};

const UNBOUND: u32 = u32::MAX;

/// Marks an entry for a port the caller chose not to materialize: it carries no
/// value and receives no register, and reads of it flow through to operands.
pub const DEFERRED: u32 = u32::MAX - 1;

/// One entry per output port, laid out region by region in walk order. An entry
/// holds a raw value id during collection and a physical register after
/// resolution.
pub struct Arena {
	region_starts: Vec<u32>,
	node_offsets: Vec<u32>,
	entries: Vec<u32>,
}

impl Arena {
	/// Creates an empty arena.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			region_starts: Vec::new(),
			node_offsets: Vec::new(),
			entries: Vec::new(),
		}
	}

	pub(crate) fn clear(&mut self) {
		self.region_starts.clear();
		self.node_offsets.clear();
		self.entries.clear();
	}

	pub(crate) fn begin_region(&mut self, nodes: &[Node]) -> usize {
		let nodes_start = self.node_offsets.len();
		let mut next_offset = u32::try_from(self.entries.len()).unwrap();

		self.region_starts.push(u32::try_from(nodes_start).unwrap());

		for node in nodes {
			self.node_offsets.push(next_offset);

			next_offset += u32::from(node.result_count());
		}

		self.entries
			.resize(usize::try_from(next_offset).unwrap(), UNBOUND);

		nodes_start
	}

	pub(crate) fn nodes_start(&self, region: u32) -> usize {
		usize::try_from(self.region_starts[usize::try_from(region).unwrap()]).unwrap()
	}

	pub(crate) fn offset(&self, nodes_start: usize, link: Link) -> usize {
		let base = self.node_offsets[nodes_start + usize::try_from(link.0).unwrap()];

		usize::try_from(base).unwrap() + usize::from(link.1)
	}

	/// Returns the number of nodes recorded for the given region.
	///
	/// # Panics
	///
	/// Panics if `region` is not less than the number of regions in the last run.
	#[must_use]
	pub fn node_count(&self, region: u32) -> usize {
		let index = usize::try_from(region).unwrap();
		let start = usize::try_from(self.region_starts[index]).unwrap();
		let end = self
			.region_starts
			.get(index + 1)
			.map_or(self.node_offsets.len(), |&next| {
				usize::try_from(next).unwrap()
			});

		end - start
	}

	/// Returns the register assigned to the port.
	#[must_use]
	pub fn register(&self, region: u32, link: Link) -> u32 {
		self.entries[self.offset(self.nodes_start(region), link)]
	}

	pub(crate) fn resolve_entries<Resolver: FnMut(u32) -> u32>(
		&mut self,
		mut resolver: Resolver,
	) -> u32 {
		let mut peak = 0;

		for entry in &mut self.entries {
			debug_assert_ne!(*entry, UNBOUND, "every port must be bound by the walk");

			if *entry == DEFERRED {
				continue;
			}

			let register = resolver(*entry);

			peak = peak.max(register + 1);
			*entry = register;
		}

		peak
	}

	pub(crate) fn read(&self, nodes_start: usize, link: Link) -> u32 {
		let entry = self.entries[self.offset(nodes_start, link)];

		debug_assert_ne!(entry, UNBOUND, "ports must be written before reads");
		debug_assert_ne!(
			entry, DEFERRED,
			"deferred ports must never be read directly"
		);

		entry
	}

	pub(crate) fn is_bound(&self, nodes_start: usize, link: Link) -> bool {
		self.entries[self.offset(nodes_start, link)] != UNBOUND
	}

	pub(crate) fn bind(&mut self, nodes_start: usize, link: Link, entry: u32) {
		let offset = self.offset(nodes_start, link);

		self.entries[offset] = entry;
	}
}

impl Default for Arena {
	fn default() -> Self {
		Self::new()
	}
}
