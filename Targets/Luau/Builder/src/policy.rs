use core::{any::Any, ptr::from_ref};

use ir_allocator::Policy;
use ir_graph::{Link, Node, Shape, region::Match};
use luau_foreign::{BufferStore, TableStore};

pub const PHYSICAL_REGISTERS: u32 = 100;

const ALL_KINDS: u8 = 0xFF;

static REGISTERS: [u8; PHYSICAL_REGISTERS as usize] = [ALL_KINDS; PHYSICAL_REGISTERS as usize];

#[derive(Clone, Copy, PartialEq, Eq)]
enum UseState {
	Unused,
	Deferrable,
	Blocked,
}

impl UseState {
	const fn block(&mut self) {
		*self = Self::Blocked;
	}

	// Deferral inlines only the value port (0); observing any other port — a live
	// state token — pins the node to a register, as does any second observer.
	const fn observe(&mut self, port: u16) {
		*self = if port == 0 && matches!(*self, Self::Unused) {
			Self::Deferrable
		} else {
			Self::Blocked
		};
	}
}

fn state_of(states: &mut [UseState], link: Link) -> &mut UseState {
	&mut states[usize::try_from(link.0).unwrap()]
}

fn classify_plain_uses(node: &Node, states: &mut [UseState]) {
	node.for_each_outer(|link| state_of(states, link).observe(link.1));
}

fn classify_blocking_uses(node: &Node, states: &mut [UseState]) {
	node.for_each_outer(|link| state_of(states, link).block());
}

fn classify_match_uses(matcher: &Match, states: &mut [UseState]) {
	// Arguments are passed as register transfers, never inlinable operands.
	for &argument in &matcher.arguments {
		state_of(states, argument).block();
	}

	// The condition is read as a test expression, but an if-chain re-tests it at
	// every internal node; only a two-branch match evaluates it once, so larger
	// matches read it from a register instead.
	if matcher.branches.len() <= 2 {
		state_of(states, matcher.condition).observe(matcher.condition.1);
	} else {
		state_of(states, matcher.condition).block();
	}
}

// The allocator's core hint map extended with this target's foreign ports.
fn reuse_hint_of(node: &Node, port: u16) -> Option<Link> {
	if let Node::Foreign(foreign) = node {
		let any: &dyn Any = foreign.as_ref();

		if let Some(store) = any.downcast_ref::<BufferStore>() {
			return (port == BufferStore::STATE_PORT).then_some(store.reference);
		}

		if let Some(store) = any.downcast_ref::<TableStore>() {
			return (port == TableStore::STATE_PORT).then_some(store.reference);
		}

		return None;
	}

	ir_allocator::reuse_hint(node, port)
}

fn classify_uses(node: &Node, states: &mut [UseState]) {
	match node.shape() {
		Shape::Plain => classify_plain_uses(node, states),
		Shape::Function(_)
		| Shape::Repeat(_)
		| Shape::BranchResults(_)
		| Shape::RepeatResults(_) => classify_blocking_uses(node, states),
		Shape::Match(arc) => classify_match_uses(&arc.lock(), states),
	}

	// The emitter copies each hinted port from its operand at the node's site,
	// so the operand must sit in a register there, never inlined.
	for port in 0..node.result_count() {
		if let Some(operand) = reuse_hint_of(node, port) {
			state_of(states, operand).block();
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
		&& reuse_hint_of(node, 0).is_none()
}

pub struct LuauPolicy {
	// Deferred nodes keyed by address: identities stay valid because the graph is
	// never mutated between `precompute` and the allocator and emitter walks.
	deferred: Vec<usize>,
	use_states: Vec<UseState>,
}

impl LuauPolicy {
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
		stacker::maybe_grow(crate::STACK_RED_ZONE, crate::STACK_SEGMENT, || {
			self.mark_use_states(nodes);
			self.record_deferred(nodes);
			self.descend_into_children(nodes);
		});
	}

	fn mark_use_states(&mut self, nodes: &[Node]) {
		self.use_states.clear();
		self.use_states.resize(nodes.len(), UseState::Unused);

		for node in nodes {
			classify_uses(node, &mut self.use_states);
		}
	}

	fn record_deferred(&mut self, nodes: &[Node]) {
		for (id, node) in nodes.iter().enumerate() {
			if self.use_states[id] == UseState::Deferrable && is_inlinable(node) {
				self.deferred.push(from_ref(node) as usize);
			}
		}
	}

	fn descend_into_children(&mut self, nodes: &[Node]) {
		for node in nodes {
			self.descend(node);
		}
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

impl Policy for LuauPolicy {
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
		reuse_hint_of(node, port)
	}
}
