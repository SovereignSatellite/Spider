//! The interval collector: one walk producing exact value intervals, affinity
//! hints, and the per-port arena bindings.

use alloc::{sync::Arc, vec::Vec};
use core::mem;

use parking_lot::Mutex;

use ir_graph::{
	Link, Node, Shape,
	region::{Branch, Function, Match, Repeat, branch, repeat},
	tracer::trace_match,
};

use crate::{
	arena::{Arena, DEFERRED},
	policy::Policy,
	value::{self, Value},
};

/// Collects exact value intervals and affinity hints for one emitted function.
pub struct Collector {
	values: Vec<Value>,
	arena: Arena,
	binding_stack: Vec<u32>,
	nodes_start: usize,
	clock: u32,
}

impl Collector {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			values: Vec::new(),
			arena: Arena::new(),
			binding_stack: Vec::new(),
			nodes_start: 0,
			clock: 0,
		}
	}

	fn enter_region(&mut self, nodes: &[Node]) -> usize {
		let saved = self.nodes_start;

		self.nodes_start = self.arena.begin_region(nodes);

		saved
	}

	fn read_port(&self, link: Link) -> u32 {
		self.arena.read(self.nodes_start, link)
	}

	fn bind_port(&mut self, link: Link, entry: u32) {
		self.arena.bind(self.nodes_start, link, entry);
	}

	fn mint(&mut self, kind: u8) -> u32 {
		debug_assert_ne!(kind, 0, "policies must return nonzero kind masks");

		let id = u32::try_from(self.values.len()).unwrap();

		self.values.push(Value::at(self.clock, kind));

		id
	}

	fn set_affinity(&mut self, value: u32, affinity: u32) {
		if value != affinity {
			self.values[usize::try_from(value).unwrap()].affinity = affinity;
			self.values[usize::try_from(affinity).unwrap()].is_reserved = true;
		}
	}

	fn observe(&mut self, policy: &dyn Policy, nodes: &[Node], link: Link) {
		let node = &nodes[usize::try_from(link.0).unwrap()];

		if !policy.should_materialize(node) {
			node.for_each_outer(|operand| self.observe(policy, nodes, operand));

			return;
		}

		let value = self.read_port(link);

		self.values[usize::try_from(value).unwrap()].end = self.clock;
	}

	fn observe_sources(&mut self, policy: &dyn Policy, nodes: &[Node], sources: &[Link]) {
		for &source in sources {
			self.observe(policy, nodes, source);
		}
	}

	// Deferral elides only the value port; a forwarded state token on a later port
	// still binds, so the node's result count is irrelevant.
	fn port_entry(
		&mut self,
		policy: &dyn Policy,
		materialized: bool,
		node: &Node,
		port: u16,
	) -> u32 {
		if !materialized && port == 0 {
			DEFERRED
		} else if let Some(operand) = node.forwarded_operand(port) {
			self.read_port(operand)
		} else {
			self.mint(policy.kind(node, port))
		}
	}

	fn emit_ports(&mut self, policy: &dyn Policy, materialized: bool, id: u32, node: &Node) {
		for port in 0..node.result_count() {
			let link = Link(id, port);

			if self.arena.is_bound(self.nodes_start, link) {
				continue;
			}

			let entry = self.port_entry(policy, materialized, node, port);

			self.bind_port(link, entry);
		}
	}

	fn visit_generic(&mut self, policy: &dyn Policy, nodes: &[Node], id: u32, node: &Node) {
		let materialized = policy.should_materialize(node);

		// A deferred node is inlined at its single use, so its operands are observed
		// there; a materialized node reads its operands here at its definition.
		if materialized {
			node.for_each_outer(|link| self.observe(policy, nodes, link));
		}

		self.clock += 1;
		self.emit_ports(policy, materialized, id, node);
		self.clock += 1;
	}

	fn bind_branch_arguments(&mut self, arguments_start: usize, sources_start: usize) {
		for index in arguments_start..sources_start {
			let port = u16::try_from(index - arguments_start).unwrap();

			self.bind_port(Link(Branch::ARGUMENTS_ID, port), self.binding_stack[index]);
		}
	}

	// A fresh, branch-local result (id at or above `branch_base`) with no affinity yet
	// prefers its column's previous-arm register, so the per-arm result transfer
	// coalesces away. Forwarded ancestors and values already coalescing inward keep
	// their own register.
	fn should_coalesce_result(&self, value: u32, previous: u32, branch_base: u32) -> bool {
		previous != value::NONE
			&& value >= branch_base
			&& self.values[usize::try_from(value).unwrap()].affinity == value::NONE
	}

	fn coalesce_branch_result(&mut self, slot: usize, source: Link, branch_base: u32) {
		let value = self.read_port(source);
		let previous = self.binding_stack[slot];

		if self.should_coalesce_result(value, previous, branch_base) {
			self.set_affinity(value, previous);
		}

		self.binding_stack[slot] = value;
	}

	fn coalesce_branch_results(
		&mut self,
		results: &branch::Results,
		sources_start: usize,
		branch_base: u32,
	) {
		for (column, &source) in results.sources.iter().enumerate() {
			self.coalesce_branch_result(sources_start + column, source, branch_base);
		}
	}

	fn visit_branch(
		&mut self,
		policy: &dyn Policy,
		arc: &Arc<Mutex<Branch>>,
		arguments_start: usize,
		sources_start: usize,
	) {
		let guard = arc.lock();
		let saved_nodes = self.enter_region(&guard.nodes);

		self.bind_branch_arguments(arguments_start, sources_start);

		// Values minted by this branch's walk are its own; lower ids are forwarded
		// ancestors that must not be coalesced into a match-local register.
		let branch_base = u32::try_from(self.values.len()).unwrap();

		self.walk(policy, &guard.nodes);
		self.coalesce_branch_results(guard.results(), sources_start, branch_base);

		drop(guard);

		self.nodes_start = saved_nodes;
	}

	fn uniform_column(&self, matcher: &Match, column: usize) -> Option<u32> {
		let origin = trace_match(matcher, u16::try_from(column).unwrap())?;

		Some(self.read_port(origin))
	}

	fn push_match_arguments(&mut self, matcher: &Match) -> usize {
		let arguments_start = self.binding_stack.len();

		for &argument in &matcher.arguments {
			self.binding_stack.push(self.read_port(argument));
		}

		arguments_start
	}

	fn mint_match_result(
		&mut self,
		policy: &dyn Policy,
		node: &Node,
		port: u16,
		source: u32,
	) -> u32 {
		let result = self.mint(policy.kind(node, port));

		self.set_affinity(result, source);

		result
	}

	fn bind_match_results(
		&mut self,
		policy: &dyn Policy,
		nodes: &[Node],
		id: u32,
		matcher: &Match,
	) {
		let node = &nodes[usize::try_from(id).unwrap()];
		let result_count = usize::from(matcher.result_count());

		// The branches left their result sources as the top of the binding stack.
		let sources_start = self.binding_stack.len() - result_count;

		for column in 0..result_count {
			let port = u16::try_from(column).unwrap();

			let entry = if let Some(source) = self.uniform_column(matcher, column) {
				source
			} else {
				let source = self.binding_stack[sources_start + column];

				self.mint_match_result(policy, node, port, source)
			};

			self.bind_port(Link(id, port), entry);
		}
	}

	fn visit_match(
		&mut self,
		policy: &dyn Policy,
		nodes: &[Node],
		id: u32,
		arc: &Arc<Mutex<Match>>,
	) {
		let matcher = arc.lock();

		self.observe(policy, nodes, matcher.condition);
		self.clock += 1;

		let arguments_start = self.push_match_arguments(&matcher);
		let sources_start = self.binding_stack.len();

		self.binding_stack.resize(
			sources_start + usize::from(matcher.result_count()),
			value::NONE,
		);

		for branch in &matcher.branches {
			self.visit_branch(policy, branch, arguments_start, sources_start);
		}

		self.bind_match_results(policy, nodes, id, &matcher);
		self.clock += 1;

		drop(matcher);

		self.binding_stack.truncate(arguments_start);
	}

	fn mint_repeat_carries(&mut self, policy: &dyn Policy, node: &Node, seeds: &[Link]) {
		for (port, &seed) in (0_u16..).zip(seeds) {
			let source = self.read_port(seed);
			let carry = self.mint(policy.kind(node, port));

			self.set_affinity(carry, source);
			self.binding_stack.push(carry);
		}
	}

	fn bind_repeat_carries(&mut self, node_id: u32, carries_start: usize, carry_count: usize) {
		for index in 0..carry_count {
			let port = u16::try_from(index).unwrap();

			self.bind_port(
				Link(node_id, port),
				self.binding_stack[carries_start + index],
			);
		}
	}

	fn visit_repeat(
		&mut self,
		policy: &dyn Policy,
		nodes: &[Node],
		id: u32,
		arc: &Arc<Mutex<Repeat>>,
	) {
		let guard = arc.lock();
		let node = &nodes[usize::try_from(id).unwrap()];
		let carries_start = self.binding_stack.len();
		let carry_count = guard.arguments.len();

		self.observe_sources(policy, nodes, &guard.arguments);
		self.clock += 1;
		self.mint_repeat_carries(policy, node, &guard.arguments);
		self.clock += 1;

		let saved_nodes = self.enter_region(&guard.nodes);

		self.bind_repeat_carries(Repeat::ARGUMENTS_ID, carries_start, carry_count);
		self.walk(policy, &guard.nodes);

		drop(guard);

		self.nodes_start = saved_nodes;

		// `record_repeat_rotation` overwrote each carry slot with its rotation source,
		// so binding the output ports here gives them the values the loop carries out.
		self.bind_repeat_carries(id, carries_start, carry_count);
		self.binding_stack.truncate(carries_start);
	}

	// Each rotation source prefers the carry it feeds, so the rotation transfer
	// coalesces away. Recording the source over its carry lets the enclosing
	// repeat bind its output ports to the sources they carry.
	fn record_repeat_rotation(&mut self, sources: &[Link]) {
		let carries_base = self.binding_stack.len() - sources.len();

		for (index, &source) in sources.iter().enumerate() {
			let carry = self.binding_stack[carries_base + index];
			let producer = self.read_port(source);

			self.set_affinity(producer, carry);
			self.binding_stack[carries_base + index] = producer;
		}
	}

	fn visit_repeat_results(
		&mut self,
		policy: &dyn Policy,
		nodes: &[Node],
		results: &repeat::Results,
	) {
		self.observe(policy, nodes, results.condition);
		self.observe_sources(policy, nodes, &results.sources);
		self.record_repeat_rotation(&results.sources);
		self.clock += 2;
	}

	fn walk(&mut self, policy: &dyn Policy, nodes: &[Node]) {
		for (id, node) in nodes.iter().enumerate() {
			let id = u32::try_from(id).unwrap();

			self.visit(policy, nodes, id, node);
		}
	}

	fn visit(&mut self, policy: &dyn Policy, nodes: &[Node], id: u32, node: &Node) {
		match node.shape() {
			// A nested function produces its closure as an ordinary value here; its
			// body is a separate allocation scope the emitter walks on its own.
			Shape::Plain | Shape::Function(_) => self.visit_generic(policy, nodes, id, node),
			Shape::Match(arc) => self.visit_match(policy, nodes, id, arc),
			Shape::Repeat(arc) => self.visit_repeat(policy, nodes, id, arc),
			Shape::BranchResults(results) => {
				self.observe_sources(policy, nodes, &results.sources);
				self.clock += 2;
			}
			Shape::RepeatResults(results) => self.visit_repeat_results(policy, nodes, results),
		}
	}

	pub fn values(&self) -> &[Value] {
		&self.values
	}

	// Each emitted function keeps its own arena because a parent reads its arena
	// while a nested function is being emitted.
	pub fn resolve(&mut self, assignments: &[u32]) -> (Arena, u32) {
		let peak = self
			.arena
			.resolve_entries(|raw| assignments[usize::try_from(raw).unwrap()]);

		(mem::take(&mut self.arena), peak)
	}

	// The sweeper pins values 0..`parameter_count` to registers 0..`parameter_count`,
	// which is sound only because the argument ports are minted as exactly those
	// first values, in port order, never deferred or forwarded away.
	fn parameters_pin_to_argument_ports(&self, nodes: &[Node]) -> bool {
		let arguments = &nodes[Function::ARGUMENTS_ID as usize];

		(0..arguments.result_count()).all(|port| {
			self.arena.register(0, Link(Function::ARGUMENTS_ID, port)) == u32::from(port)
		})
	}

	fn reset(&mut self) {
		self.values.clear();
		self.arena.clear();
		self.binding_stack.clear();
		self.nodes_start = 0;
		self.clock = 0;
	}

	pub fn run(&mut self, policy: &dyn Policy, nodes: &[Node]) {
		self.reset();

		self.enter_region(nodes);
		self.walk(policy, nodes);

		debug_assert!(
			self.parameters_pin_to_argument_ports(nodes),
			"the sweeper pins the first values to registers, so they must be the argument ports"
		);
	}
}
