use alloc::sync::Arc;

use hashbrown::{HashMap, hash_map::Entry};

use ir_graph::{Link, Node};

#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
pub fn value_port_count_of(node: &Node) -> u16 {
	match node {
		Node::Function(_)
		| Node::Import(_)
		| Node::Trap
		| Node::Null
		| Node::I32(_)
		| Node::I64(_)
		| Node::F32(_)
		| Node::F64(_)
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
		| Node::TableNew(_)
		| Node::TableGet(_)
		| Node::TableSize(_)
		| Node::TableGrow(_)
		| Node::MemoryNew(_)
		| Node::MemoryLoad(_)
		| Node::MemorySize(_)
		| Node::MemoryGrow(_) => 1,

		Node::Match(arc) => arc.lock().result_count(),
		Node::Repeat(arc) => arc.lock().result_count(),

		Node::ModuleArguments(_)
		| Node::ModuleResults(_)
		| Node::FunctionCaptures(_)
		| Node::FunctionArguments(_)
		| Node::FunctionResults(_)
		| Node::BranchArguments(_)
		| Node::BranchResults(_)
		| Node::RepeatArguments(_)
		| Node::RepeatResults(_)
		| Node::Fence(_)
		| Node::GlobalSet(_)
		| Node::TableSet(_)
		| Node::TableFill(_)
		| Node::TableCopy(_)
		| Node::TableDrop(_)
		| Node::MemoryStore(_)
		| Node::MemoryFill(_)
		| Node::MemoryCopy(_)
		| Node::MemoryDrop(_) => 0,

		Node::Host(_node) => 0,

		Node::Identity(node) => node.result_count(),

		Node::Apply(node) => node.result_count(),
	}
}

type ScopedLink = (Link, usize);

fn add_value_assignments(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	nodes: &[Node],
	scope: usize,
	id: u32,
) {
	let value_port_count = value_port_count_of(&nodes[id as usize]);

	for link in (0..value_port_count).map(|port| (Link(id, port), scope)) {
		let _ = assignments.try_insert(link, (Link::DANGLING, 0));
	}
}

pub struct ScalarFinder {
	handled: HashMap<(u32, usize), bool>,
}

impl ScalarFinder {
	pub fn new() -> Self {
		Self {
			handled: HashMap::new(),
		}
	}

	#[expect(
		clippy::too_many_arguments,
		reason = "pass-through parameters for region traversal"
	)]
	fn handle_effects(
		&mut self,
		assignments: &mut HashMap<ScopedLink, ScopedLink>,
		nodes: &[Node],
		scope: usize,
		id: u32,
		node: &Node,
	) {
		// Without at least one value reference we might discard the side effects
		// of these expressions.
		if !matches!(
			node,
			Node::Apply(_) | Node::TableGrow(_) | Node::MemoryGrow(_)
		) {
			return;
		}

		match self.handled.entry((id, scope)) {
			Entry::Vacant(entry) => {
				entry.insert(true);

				add_value_assignments(assignments, nodes, scope, id);
			}
			Entry::Occupied(mut entry) => {
				if !entry.insert(true) {
					add_value_assignments(assignments, nodes, scope, id);
				}
			}
		}
	}

	fn handle_repeat(
		&mut self,
		assignments: &mut HashMap<ScopedLink, ScopedLink>,
		nodes: &[Node],
		scope: usize,
		id: u32,
	) {
		match self.handled.entry((id, scope)) {
			Entry::Occupied(mut entry) => {
				if entry.insert(true) {
					return;
				}

				add_value_assignments(assignments, nodes, scope, id);
			}
			Entry::Vacant(entry) => {
				entry.insert(false);
			}
		}
	}

	fn handle_excess(
		&mut self,
		assignments: &mut HashMap<ScopedLink, ScopedLink>,
		nodes: &[Node],
		scope: usize,
		id: u32,
	) {
		if self.handled.insert((id, scope), true).unwrap_or_default() {
			return;
		}

		add_value_assignments(assignments, nodes, scope, id);
	}

	fn handle_uses(
		&mut self,
		assignments: &mut HashMap<ScopedLink, ScopedLink>,
		nodes: &[Node],
		scope: usize,
		node: &Node,
	) {
		node.for_each_outer(|Link(id, port)| {
			if port == 0 {
				self.handle_repeat(assignments, nodes, scope, id);
			} else {
				self.handle_excess(assignments, nodes, scope, id);
			}
		});
	}

	fn handle_root_uses(
		&mut self,
		assignments: &mut HashMap<ScopedLink, ScopedLink>,
		nodes: &[Node],
		scope: usize,
		roots: &[Link],
	) {
		for &Link(id, port) in roots {
			if port == 0 {
				self.handle_repeat(assignments, nodes, scope, id);
			} else {
				self.handle_excess(assignments, nodes, scope, id);
			}
		}
	}

	#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
	fn run_region(
		&mut self,
		assignments: &mut HashMap<ScopedLink, ScopedLink>,
		nodes: &[Node],
		scope: usize,
		roots: &[Link],
	) {
		// All uses are handled first since that contains all base assignments.
		for node in nodes {
			self.handle_uses(assignments, nodes, scope, node);
		}

		// Region output Links are additional uses.
		self.handle_root_uses(assignments, nodes, scope, roots);

		// Then, effects are handled from the missing assignments.
		for (id, node) in nodes.iter().enumerate() {
			let id = id.try_into().unwrap();

			self.handle_effects(assignments, nodes, scope, id, node);
		}

		// Recurse into nested control regions (not Functions).
		for node in nodes {
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

				Node::Match(arc) => {
					let matcher = arc.lock();

					for branch_arc in &matcher.branches {
						let branch = branch_arc.lock();
						let branch_scope = Arc::as_ptr(branch_arc) as usize;

						self.run_region(
							assignments,
							&branch.nodes,
							branch_scope,
							&branch.results().sources,
						);
					}
				}
				Node::Repeat(arc) => {
					let repeat = arc.lock();
					let repeat_scope = Arc::as_ptr(arc) as usize;

					self.run_region(
						assignments,
						&repeat.nodes,
						repeat_scope,
						&repeat.results().sources,
					);
				}
			}
		}
	}

	// We assign locals to all value ports in a node if...
	//   * Any value port is used out of local order.
	//   * Any value port has more than one use.
	//   * Any value port other than the first is in use.
	//   * No value port is used but it has side effects.
	pub fn run(
		&mut self,
		assignments: &mut HashMap<ScopedLink, ScopedLink>,
		nodes: &[Node],
		scope: usize,
		roots: &[Link],
	) {
		self.handled.clear();

		self.run_region(assignments, nodes, scope, roots);
	}
}
