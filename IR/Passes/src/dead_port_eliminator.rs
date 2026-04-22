//! Dead port elimination.

use alloc::sync::Arc;

use hashbrown::HashMap;
use parking_lot::Mutex;
use set::Set;

use ir_graph::{
	Link, Node,
	region::{Branch, Match, Repeat},
};

/// Eliminates unused ports from control flow region nodes.
pub struct DeadPortEliminator {
	map: HashMap<Link, Link>,
	live: Set,
	remap: Vec<u16>,
}

impl DeadPortEliminator {
	/// Creates a new dead port eliminator.
	#[must_use]
	pub fn new() -> Self {
		Self {
			map: HashMap::new(),
			live: Set::new(),
			remap: Vec::new(),
		}
	}

	fn mark_use(&mut self, boundary: u32, link: Link) {
		if link.0 == boundary {
			self.live.grow_insert(link.1.into());
		}
	}

	fn mark_nodes(&mut self, nodes: &[Node], boundary: u32) {
		for node in nodes {
			node.for_each_outer(|link| self.mark_use(boundary, link));
		}
	}

	fn mark_external(&mut self, id: u32, nodes: &[Node]) {
		self.live.clear();
		self.mark_nodes(nodes, id);
	}

	fn mark_branches(&mut self, branches: &[Arc<Mutex<Branch>>]) {
		self.live.clear();

		for branch in branches {
			let guard = branch.lock();

			self.mark_nodes(&guard.nodes, 0);
		}
	}

	fn build_remap(&mut self, length: u16) -> bool {
		self.remap.clear();
		self.remap.resize(length.into(), u16::MAX);

		let mut cursor = 0_u16;

		for old in 0..usize::from(length) {
			if self.live.contains(old) {
				self.remap[old] = cursor;
				cursor += 1;
			}
		}

		cursor != length
	}

	fn remap_link(&self, boundary: u32, link: &mut Link) {
		if link.0 != boundary {
			return;
		}

		link.1 = self.remap[usize::from(link.1)];

		debug_assert_ne!(link.1, u16::MAX, "link port must not be dangling");
	}

	fn remap_nodes(&self, nodes: &mut [Node], boundary: u32) {
		for node in nodes {
			node.for_each_mut_outer(|link| self.remap_link(boundary, link));
		}
	}

	fn trim_slots(&self, slots: &mut Vec<Link>) {
		let mut iter = self.remap.iter().copied();

		slots.retain(|_| iter.next().unwrap() != u16::MAX);
	}

	fn record_outer_remap(&mut self, id: u32) {
		for (old, &new) in self.remap.iter().enumerate() {
			let old = u16::try_from(old).unwrap();

			if new != u16::MAX && new != old {
				self.map.insert(Link(id, old), Link(id, new));
			}
		}
	}

	fn find_match_outputs(&mut self, id: u32, matcher: &Match) {
		if !self.build_remap(matcher.result_count()) {
			return;
		}

		for branch in &matcher.branches {
			let mut guard = branch.lock();

			self.trim_slots(&mut guard.results_mut().sources);
		}

		self.record_outer_remap(id);
	}

	fn find_match_inputs(&mut self, matcher: &mut Match) {
		self.mark_branches(&matcher.branches);

		if !self.build_remap(matcher.argument_count()) {
			return;
		}

		for branch in &matcher.branches {
			let mut guard = branch.lock();

			self.remap_nodes(&mut guard.nodes, 0);
		}

		self.trim_slots(&mut matcher.arguments);
	}

	fn find_match(&mut self, id: u32, arc: &Arc<Mutex<Match>>, nodes: &[Node]) {
		self.mark_external(id, nodes);

		let mut guard = arc.lock();

		self.find_match_outputs(id, &guard);
		self.find_match_inputs(&mut guard);
	}

	fn find_repeat(&mut self, id: u32, arc: &Arc<Mutex<Repeat>>, nodes: &[Node]) {
		self.mark_external(id, nodes);

		let mut guard = arc.lock();

		self.mark_nodes(&guard.nodes, 0);

		if !self.build_remap(guard.result_count()) {
			return;
		}

		self.remap_nodes(&mut guard.nodes, 0);
		self.trim_slots(&mut guard.arguments);
		self.trim_slots(&mut guard.results_mut().sources);

		drop(guard);

		self.record_outer_remap(id);
	}

	#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
	fn find_all(&mut self, nodes: &[Node]) {
		self.map.clear();

		for (index, node) in nodes.iter().enumerate() {
			let id = u32::try_from(index).unwrap();

			match node {
				Node::Match(arc) => self.find_match(id, arc, nodes),
				Node::Repeat(arc) => self.find_repeat(id, arc, nodes),

				Node::Function(_)
				| Node::ModuleArguments(_)
				| Node::ModuleResults(_)
				| Node::FunctionArguments(_)
				| Node::FunctionResults(_)
				| Node::BranchArguments(_)
				| Node::BranchResults(_)
				| Node::RepeatArguments(_)
				| Node::RepeatResults(_)
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
				| Node::Aggregate(_)
				| Node::Extract(_)
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

	fn assign_single(&self, link: &mut Link) {
		if let Some(&new) = self.map.get(link) {
			*link = new;
		}
	}

	fn assign_all(&self, nodes: &mut [Node]) {
		for node in nodes.iter_mut() {
			node.for_each_mut_outer(|link| self.assign_single(link));
		}
	}

	/// Runs the dead port elimination pass on the region.
	pub fn run(&mut self, nodes: &mut [Node]) {
		self.find_all(nodes);
		self.assign_all(nodes);
	}
}

impl Default for DeadPortEliminator {
	fn default() -> Self {
		Self::new()
	}
}
