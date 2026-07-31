//! Invariant port motion.

use alloc::sync::Arc;

use hashbrown::HashMap;
use parking_lot::Mutex;
use set::Set;

use ir_graph::{
	Link, Node, Shape,
	region::{Match, Repeat},
	tracer,
};

/// Moves invariant ports out of control flow regions.
pub struct InvariantPortMover {
	replacements: HashMap<Link, Link>,
	visited: Set,
}

fn fixup_repeat_inner_references(repeat: &mut Repeat) -> bool {
	let position = repeat.results_index();
	let (inner, tail) = repeat.nodes.split_at_mut(position);
	let Node::RepeatResults(results) = &tail[0] else {
		unreachable!()
	};

	let mut changed = false;

	for node in inner.iter_mut() {
		node.for_each_mut_outer(|link| {
			if link.0 != 0 {
				return;
			}

			let result = results.sources[usize::from(link.1)];

			if result.0 == 0 && result.1 != link.1 {
				*link = result;
				changed = true;
			}
		});
	}

	changed
}

impl InvariantPortMover {
	/// Creates a new invariant port mover.
	#[must_use]
	pub fn new() -> Self {
		Self {
			replacements: HashMap::new(),
			visited: Set::new(),
		}
	}

	fn resolve_replacement(&self, link: Link) -> Link {
		self.replacements.get(&link).copied().unwrap_or(link)
	}

	fn collect_match_replacements(&mut self, identifier: u32, match_region: &Arc<Mutex<Match>>) {
		let guard = match_region.lock();

		for port in 0..guard.result_count() {
			if let Some(origin) = tracer::trace_match(&guard, port) {
				self.replacements
					.insert(Link(identifier, port), self.resolve_replacement(origin));
			}
		}
	}

	fn collect_repeat_replacements(
		&mut self,
		identifier: u32,
		repeat_region: &Arc<Mutex<Repeat>>,
	) -> bool {
		let mut guard = repeat_region.lock();
		let mut changed = false;

		for port in 0..guard.result_count() {
			let Some(origin) = tracer::trace_repeat(&mut self.visited, &guard, port) else {
				continue;
			};

			let canonical_port = self.visited.ascending().next().unwrap();
			let canonical_port = u16::try_from(canonical_port).unwrap();

			self.replacements
				.insert(Link(identifier, port), self.resolve_replacement(origin));

			let replacement = Link(0, canonical_port);
			let source = &mut guard.results_mut().sources[usize::from(port)];

			if *source != replacement {
				*source = replacement;
				changed = true;
			}
		}
		drop(guard);

		changed
	}

	fn collect_replacements(&mut self, nodes: &[Node]) -> bool {
		self.replacements.clear();
		let mut changed = false;

		for (index, node) in nodes.iter().enumerate() {
			let identifier = u32::try_from(index).unwrap();

			match node.shape() {
				Shape::Plain
				| Shape::Function(_)
				| Shape::BranchResults(_)
				| Shape::RepeatResults(_) => {}
				Shape::Match(match_region) => {
					self.collect_match_replacements(identifier, match_region);
				}
				Shape::Repeat(repeat_region) => {
					changed |= self.collect_repeat_replacements(identifier, repeat_region);
				}
			}
		}

		changed
	}

	fn apply_replacement(&self, link: &mut Link) -> bool {
		let Some(&replacement) = self.replacements.get(link) else {
			return false;
		};
		let changed = *link != replacement;

		*link = replacement;

		changed
	}

	fn apply_replacements(&self, nodes: &mut [Node]) -> bool {
		let mut changed = false;

		for node in nodes.iter_mut() {
			node.for_each_mut_outer(|link| changed |= self.apply_replacement(link));
		}

		for node in nodes.iter() {
			if let Node::Repeat(repeat_region) = node {
				changed |= fixup_repeat_inner_references(&mut repeat_region.lock());
			}
		}

		changed
	}

	/// Run invariant port motion on the region, reporting whether any link changed.
	#[must_use = "propagate whether this pass changed the graph"]
	pub fn run(&mut self, nodes: &mut [Node]) -> bool {
		let nested_regions_changed = self.collect_replacements(nodes);

		self.apply_replacements(nodes) || nested_regions_changed
	}
}

impl Default for InvariantPortMover {
	fn default() -> Self {
		Self::new()
	}
}
