//! Invariant port motion.

use alloc::sync::Arc;

use hashbrown::HashMap;
use parking_lot::Mutex;
use set::Set;

use ir_graph::{
	Link, Node,
	control::{Match, Repeat},
};

use super::tracer;

/// Moves invariant ports out of control flow regions.
pub struct InvariantPortMover {
	map: HashMap<Link, Link>,
	visited: Set,
}

impl InvariantPortMover {
	/// Creates a new invariant port mover.
	#[must_use]
	pub fn new() -> Self {
		Self {
			map: HashMap::new(),
			visited: Set::new(),
		}
	}

	fn get_redirected(&self, link: Link) -> Link {
		self.map.get(&link).copied().unwrap_or(link)
	}

	fn find_match(&mut self, id: u32, arc: &Arc<Mutex<Match>>) {
		let guard = arc.lock();

		for port in 0..guard.result_count() {
			if let Some(origin) = tracer::trace_match(&guard, port) {
				self.map.insert(Link(id, port), self.get_redirected(origin));
			}
		}
	}

	fn find_repeat(&mut self, id: u32, arc: &Arc<Mutex<Repeat>>) {
		let mut guard = arc.lock();

		for port in 0..guard.result_count() {
			let Some(origin) = tracer::trace_repeat(&mut self.visited, &guard, port) else {
				continue;
			};

			let canonical = self.visited.ascending().next().unwrap();
			let canonical = u16::try_from(canonical).unwrap();

			self.map.insert(Link(id, port), self.get_redirected(origin));

			guard.results_mut().sources[usize::from(port)] = Link(0, canonical);
		}
	}

	#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
	fn find_all(&mut self, nodes: &[Node]) {
		self.map.clear();

		for (index, node) in nodes.iter().enumerate() {
			let id = u32::try_from(index).unwrap();

			match node {
				Node::Function(_)
				| Node::ModuleArguments(_)
				| Node::ModuleResults(_)
				| Node::FunctionCaptures(_)
				| Node::FunctionArguments(_)
				| Node::FunctionResults(_)
				| Node::BranchArguments(_)
				| Node::BranchResults(_)
				| Node::RepeatArguments(_)
				| Node::RepeatResults(_)
				| Node::Import(_)
				| Node::Host(_)
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
				| Node::IntegerExtend(_)
				| Node::IntegerConvertToNumber(_)
				| Node::IntegerTransmuteToNumber(_)
				| Node::NumberUnaryOperation(_)
				| Node::NumberBinaryOperation(_)
				| Node::NumberCompareOperation(_)
				| Node::NumberNarrow(_)
				| Node::NumberWiden(_)
				| Node::NumberTruncateToInteger(_)
				| Node::NumberTransmuteToInteger(_)
				| Node::GlobalNew(_)
				| Node::GlobalGet(_)
				| Node::GlobalSet(_)
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

				Node::Match(arc) => self.find_match(id, arc),
				Node::Repeat(arc) => self.find_repeat(id, arc),
			}
		}
	}

	fn assign_single(&self, link: &mut Link) {
		if let Some(&new) = self.map.get(link) {
			*link = new;
		}
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

	fn assign_all(&self, nodes: &mut [Node]) {
		for node in nodes.iter_mut() {
			node.for_each_mut_outer(|link| self.assign_single(link));
		}

		for node in nodes.iter() {
			if let Node::Repeat(arc) = node {
				Self::fixup_repeat_inner_references(&mut arc.lock());
			}
		}
	}

	/// Runs the invariant port motion pass on the region.
	pub fn run(&mut self, nodes: &mut [Node]) {
		self.find_all(nodes);
		self.assign_all(nodes);
	}
}

impl Default for InvariantPortMover {
	fn default() -> Self {
		Self::new()
	}
}
