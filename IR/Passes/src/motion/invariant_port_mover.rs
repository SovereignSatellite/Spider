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

fn fixup_repeat_inner_references(repeat: &mut Repeat) {
	let position = repeat.results_index();
	let (inner, tail) = repeat.nodes.split_at_mut(position);
	let Node::RepeatResults(results) = &tail[0] else {
		unreachable!()
	};

	for node in inner.iter_mut() {
		node.for_each_mut_outer(|link| {
			if link.0 != 0 {
				return;
			}

			let result = results.sources[usize::from(link.1)];

			if result.0 == 0 && result.1 != link.1 {
				*link = result;
			}
		});
	}
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

	fn process_match(&mut self, id: u32, arc: &Arc<Mutex<Match>>) {
		let guard = arc.lock();

		for port in 0..guard.result_count() {
			if let Some(origin) = tracer::trace_match(&guard, port) {
				self.replacements
					.insert(Link(id, port), self.resolve_replacement(origin));
			}
		}
	}

	fn process_repeat(&mut self, id: u32, arc: &Arc<Mutex<Repeat>>) {
		let mut guard = arc.lock();

		for port in 0..guard.result_count() {
			let Some(origin) = tracer::trace_repeat(&mut self.visited, &guard, port) else {
				continue;
			};

			let canonical = self.visited.ascending().next().unwrap();
			let canonical = u16::try_from(canonical).unwrap();

			self.replacements
				.insert(Link(id, port), self.resolve_replacement(origin));

			guard.results_mut().sources[usize::from(port)] = Link(0, canonical);
		}
	}

	fn process_all(&mut self, nodes: &[Node]) {
		self.replacements.clear();

		for (index, node) in nodes.iter().enumerate() {
			let id = u32::try_from(index).unwrap();

			match node.shape() {
				Shape::Plain
				| Shape::Function(_)
				| Shape::BranchResults(_)
				| Shape::RepeatResults(_) => {}
				Shape::Match(arc) => self.process_match(id, arc),
				Shape::Repeat(arc) => self.process_repeat(id, arc),
			}
		}
	}

	fn apply_single(&self, link: &mut Link) {
		if let Some(&new) = self.replacements.get(link) {
			*link = new;
		}
	}

	fn apply_all(&self, nodes: &mut [Node]) {
		for node in nodes.iter_mut() {
			node.for_each_mut_outer(|link| self.apply_single(link));
		}

		for node in nodes.iter() {
			if let Node::Repeat(arc) = node {
				fixup_repeat_inner_references(&mut arc.lock());
			}
		}
	}

	/// Runs the invariant port motion pass on the region.
	pub fn run(&mut self, nodes: &mut [Node]) {
		self.process_all(nodes);
		self.apply_all(nodes);
	}
}

impl Default for InvariantPortMover {
	fn default() -> Self {
		Self::new()
	}
}
