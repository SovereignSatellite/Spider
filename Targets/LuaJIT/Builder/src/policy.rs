use core::ptr::from_ref;

use ir_allocator::Policy;
use ir_graph::{Link, Node, Shape};

pub const PHYSICAL_REGISTERS: u32 = 100;

const ALL_KINDS: u8 = 0xFF;

static REGISTERS: [u8; PHYSICAL_REGISTERS as usize] = [ALL_KINDS; PHYSICAL_REGISTERS as usize];

#[derive(Clone, Copy, PartialEq, Eq)]
enum UseState {
	Unused,
	Deferrable,
	Blocked,
}

pub struct LuaJITPolicy {
	// Deferred nodes keyed by address: identities stay valid because the graph is
	// never mutated between `precompute` and the allocator and emitter walks.
	deferred: Vec<usize>,
	use_states: Vec<UseState>,
}

impl LuaJITPolicy {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			deferred: Vec::new(),
			use_states: Vec::new(),
		}
	}

	/// Records, for the whole function, every node whose single output is
	/// inlined at its one use rather than materialized into a register.
	pub fn precompute(&mut self, nodes: &[Node]) {
		self.deferred.clear();

		self.collect_deferrals(nodes);
		self.deferred.sort_unstable();
	}

	fn collect_deferrals(&mut self, nodes: &[Node]) {
		self.use_states.clear();
		self.use_states.resize(nodes.len(), UseState::Unused);

		for node in nodes {
			Self::classify_uses(node, &mut self.use_states);
		}

		for (id, node) in nodes.iter().enumerate() {
			if self.use_states[id] == UseState::Deferrable && Self::is_inlinable(node) {
				self.deferred.push(from_ref(node) as usize);
			}
		}

		for node in nodes {
			self.descend(node);
		}
	}

	fn classify_uses(node: &Node, states: &mut [UseState]) {
		let plain_consumer = matches!(node.shape(), Shape::Plain);

		node.for_each_outer(|link| {
			let state = &mut states[usize::try_from(link.0).unwrap()];

			// Deferral inlines only the value port (0); observing any other port — a
			// live state token — pins the node to a register.
			*state = if link.1 == 0 && *state == UseState::Unused && plain_consumer {
				UseState::Deferrable
			} else {
				UseState::Blocked
			};
		});

		for port in 0..node.result_count() {
			if let Some(operand) = ir_allocator::reuse_hint(node, port) {
				states[usize::try_from(operand.0).unwrap()] = UseState::Blocked;
			}
		}
	}

	fn is_inlinable(node: &Node) -> bool {
		// A boundary argument is bound to a register the emitter reads directly, a
		// call emits as a statement, and a hinted port-0 carries an operand's
		// value rather than one of its own, so none folds into an expression.
		!matches!(
			node,
			Node::FunctionArguments(_)
				| Node::BranchArguments(_)
				| Node::RepeatArguments(_)
				| Node::Apply(_)
		) && matches!(node.shape(), Shape::Plain)
			&& ir_allocator::reuse_hint(node, 0).is_none()
	}

	fn descend(&mut self, node: &Node) {
		match node.shape() {
			Shape::Plain | Shape::BranchResults(_) | Shape::RepeatResults(_) => {}
			Shape::Function(arc) => self.collect_deferrals(&arc.lock().nodes),
			Shape::Match(arc) => {
				let matcher = arc.lock();

				for branch in &matcher.branches {
					self.collect_deferrals(&branch.lock().nodes);
				}
			}
			Shape::Repeat(arc) => self.collect_deferrals(&arc.lock().nodes),
		}
	}
}

impl Policy for LuaJITPolicy {
	fn registers(&self) -> &[u8] {
		&REGISTERS
	}

	fn kind(&self, _node: &Node, _port: u16) -> u8 {
		ALL_KINDS
	}

	fn should_materialize(&self, node: &Node) -> bool {
		let address = from_ref(node) as usize;

		self.deferred.binary_search(&address).is_err()
	}

	fn reuse_hint(&self, node: &Node, port: u16) -> Option<Link> {
		ir_allocator::reuse_hint(node, port)
	}
}
