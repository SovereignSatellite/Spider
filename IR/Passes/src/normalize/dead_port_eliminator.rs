//! Dead port elimination.

use alloc::sync::Arc;
use core::mem;

use parking_lot::Mutex;

use ir_graph::{
	Link, Node, Shape,
	region::{Branch, Match, Repeat},
};

const UNMARKED: u16 = 0;
const MARKED: u16 = 1;
const DEAD: u16 = u16::MAX;

/// Eliminates unused ports from control flow region nodes.
pub struct DeadPortEliminator {
	remap: Vec<u16>,
}

fn set_branch_argument_counts(branches: &[Arc<Mutex<Branch>>], argument_count: u16) {
	for branch in branches {
		branch.lock().set_argument_count(argument_count);
	}
}

impl DeadPortEliminator {
	/// Creates a new dead port eliminator.
	#[must_use]
	pub const fn new() -> Self {
		Self { remap: Vec::new() }
	}

	fn begin(&mut self, length: u16) {
		self.remap.clear();
		self.remap.resize(length.into(), UNMARKED);
	}

	fn mark_use(&mut self, boundary: u32, link: Link) {
		if link.0 == boundary {
			self.remap[usize::from(link.1)] = MARKED;
		}
	}

	fn mark_nodes(&mut self, nodes: &[Node], boundary: u32) {
		for node in nodes {
			node.for_each_outer(|link| self.mark_use(boundary, link));
		}
	}

	fn mark_branches(&mut self, branches: &[Arc<Mutex<Branch>>]) {
		for branch in branches {
			let guard = branch.lock();

			self.mark_nodes(&guard.nodes, Branch::ARGUMENTS_ID);
		}
	}

	fn build_remap(&mut self) -> bool {
		let mut cursor = 0_u16;

		for slot in &mut self.remap {
			if *slot == MARKED {
				*slot = cursor;
				cursor += 1;
			} else {
				*slot = DEAD;
			}
		}

		usize::from(cursor) != self.remap.len()
	}

	fn remap_link(&self, boundary: u32, link: &mut Link) {
		if link.0 != boundary {
			return;
		}

		link.1 = self.remap[usize::from(link.1)];

		debug_assert_ne!(link.1, DEAD, "link port must not be dangling");
	}

	fn remap_nodes(&self, nodes: &mut [Node], boundary: u32) {
		for node in nodes {
			node.for_each_mut_outer(|link| self.remap_link(boundary, link));
		}
	}

	fn trim_slots(&self, slots: &mut Vec<Link>) {
		let mut iter = self.remap.iter().copied();

		slots.retain(|_| iter.next().unwrap() != DEAD);
	}

	fn process_match_outputs(
		&mut self,
		id: u32,
		arc: &Arc<Mutex<Match>>,
		nodes: &mut [Node],
	) -> bool {
		self.begin(arc.lock().result_count());
		self.mark_nodes(nodes, id);

		if !self.build_remap() {
			return false;
		}

		let guard = arc.lock();

		for branch in &guard.branches {
			self.trim_slots(&mut branch.lock().results_mut().sources);
		}

		drop(guard);

		self.remap_nodes(nodes, id);

		true
	}

	fn trim_match_arguments(&self, matcher: &mut Match) {
		self.trim_slots(&mut matcher.arguments);

		set_branch_argument_counts(&matcher.branches, matcher.argument_count());
	}

	fn process_match_inputs(&mut self, arc: &Arc<Mutex<Match>>) -> bool {
		let mut guard = arc.lock();

		self.begin(guard.argument_count());
		self.mark_branches(&guard.branches);

		if !self.build_remap() {
			return false;
		}

		for branch in &guard.branches {
			self.remap_nodes(&mut branch.lock().nodes, Branch::ARGUMENTS_ID);
		}

		self.trim_match_arguments(&mut guard);
		drop(guard);

		true
	}

	fn process_match(&mut self, id: u32, arc: &Arc<Mutex<Match>>, nodes: &mut [Node]) -> bool {
		let trimmed_outputs = self.process_match_outputs(id, arc, nodes);
		let trimmed_inputs = self.process_match_inputs(arc);

		trimmed_outputs || trimmed_inputs
	}

	fn trim_repeat_ports(&self, repeat: &mut Repeat) {
		self.trim_slots(&mut repeat.arguments);
		repeat.set_argument_count(repeat.argument_count());

		self.trim_slots(&mut repeat.results_mut().sources);
	}

	fn mark_repeat_interior(&mut self, repeat: &Repeat) {
		let position = repeat.results_index();

		for (index, node) in repeat.nodes.iter().enumerate() {
			if index != position {
				node.for_each_outer(|link| self.mark_use(Repeat::ARGUMENTS_ID, link));
			}
		}

		let results = repeat.results();

		self.mark_use(Repeat::ARGUMENTS_ID, results.condition);

		// A diagonal source feeds the carry only to itself and never counts as a use.
		for (column, &source) in (0_u16..).zip(&results.sources) {
			if source != Link(Repeat::ARGUMENTS_ID, column) {
				self.mark_use(Repeat::ARGUMENTS_ID, source);
			}
		}
	}

	fn process_repeat(&mut self, id: u32, arc: &Arc<Mutex<Repeat>>, nodes: &mut [Node]) -> bool {
		self.begin(arc.lock().result_count());
		self.mark_nodes(nodes, id);

		let mut guard = arc.lock();

		self.mark_repeat_interior(&guard);

		if !self.build_remap() {
			return false;
		}

		// Trimming precedes the remap so a dying diagonal is gone before it would dangle.
		self.trim_repeat_ports(&mut guard);
		self.remap_nodes(&mut guard.nodes, Repeat::ARGUMENTS_ID);

		drop(guard);

		self.remap_nodes(nodes, id);

		true
	}

	fn process(&mut self, nodes: &mut [Node], index: usize) -> bool {
		let id = u32::try_from(index).unwrap();
		let node = mem::take(&mut nodes[index]);

		let trimmed = match node.shape() {
			Shape::Plain
			| Shape::Function(_)
			| Shape::BranchResults(_)
			| Shape::RepeatResults(_) => false,
			Shape::Match(arc) => self.process_match(id, arc, nodes),
			Shape::Repeat(arc) => self.process_repeat(id, arc, nodes),
		};

		nodes[index] = node;

		trimmed
	}

	/// Runs the dead port elimination pass on the region, reporting whether any
	/// port was trimmed.
	#[must_use = "propagate whether this pass changed the graph"]
	pub fn run(&mut self, nodes: &mut [Node]) -> bool {
		let mut trimmed = false;

		for index in 0..nodes.len() {
			trimmed |= self.process(nodes, index);
		}

		trimmed
	}
}

impl Default for DeadPortEliminator {
	fn default() -> Self {
		Self::new()
	}
}
