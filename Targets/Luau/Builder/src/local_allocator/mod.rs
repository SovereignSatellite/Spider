use alloc::sync::Arc;

use hashbrown::HashMap;
use parking_lot::Mutex;

use ir_graph::{
	Link, Node,
	control::{Function, Match, Repeat},
};
use luau_tree::expression::Local;

use self::scalar_finder::ScalarFinder;

mod reference_finder;
mod scalar_finder;

type ScopedLink = (Link, usize);

pub struct LocalAllocator {
	preferences: HashMap<ScopedLink, ScopedLink>,
	functions: Vec<Arc<Mutex<Function>>>,

	scalar_finder: ScalarFinder,

	next_offset: u16,
}

impl LocalAllocator {
	pub fn new() -> Self {
		Self {
			preferences: HashMap::new(),
			functions: Vec::new(),

			scalar_finder: ScalarFinder::new(),

			next_offset: 0,
		}
	}

	fn find_functions_in(functions: &mut Vec<Arc<Mutex<Function>>>, nodes: &[Node]) {
		for node in nodes.iter().rev() {
			if let Node::Function(arc) = node {
				Self::find_functions_in(functions, &arc.lock().nodes);

				functions.push(Arc::clone(arc));
			} else if let Node::Match(arc) = node {
				for branch_arc in &arc.lock().branches {
					Self::find_functions_in(functions, &branch_arc.lock().nodes);
				}
			} else if let Node::Repeat(arc) = node {
				Self::find_functions_in(functions, &arc.lock().nodes);
			}
		}
	}

	fn alloc_slow(&mut self, assignments: &mut HashMap<ScopedLink, Local>, link: ScopedLink) {
		if assignments.contains_key(&link) {
			return;
		}

		let offset = self.next_offset;

		self.next_offset += 1;

		assignments.insert(link, Local::Slow { offset });
	}

	fn run_finders(&mut self, nodes: &[Node], scope: usize, roots: &[Link]) {
		reference_finder::run(&mut self.preferences, nodes, scope);
		self.scalar_finder
			.run(&mut self.preferences, nodes, scope, roots);
	}

	fn assign_preferred_ports(
		&mut self,
		assignments: &mut HashMap<ScopedLink, Local>,
		id: u32,
		scope: usize,
	) {
		let mut port = 0_u16;

		while self.preferences.contains_key(&(Link(id, port), scope)) {
			self.alloc_slow(assignments, (Link(id, port), scope));
			port += 1;
		}

		for extra in port..port.saturating_add(8) {
			let link = (Link(id, extra), scope);

			if self.preferences.contains_key(&link) {
				self.alloc_slow(assignments, link);
			}
		}
	}

	fn assign_match(
		&mut self,
		assignments: &mut HashMap<ScopedLink, Local>,
		id: u32,
		scope: usize,
		arc: &Arc<Mutex<Match>>,
	) {
		let matcher = arc.lock();

		for branch_arc in &matcher.branches {
			let branch_scope = Arc::as_ptr(branch_arc) as usize;

			self.assign_scope(assignments, &branch_arc.lock().nodes, branch_scope);
		}

		for port in 0..matcher.result_count() {
			self.alloc_slow(assignments, (Link(id, port), scope));
		}
	}

	fn assign_repeat(
		&mut self,
		assignments: &mut HashMap<ScopedLink, Local>,
		id: u32,
		scope: usize,
		arc: &Arc<Mutex<Repeat>>,
	) {
		let repeat = arc.lock();
		let repeat_scope = Arc::as_ptr(arc) as usize;

		self.assign_scope(assignments, &repeat.nodes, repeat_scope);

		for (offset, &result) in repeat.results().sources.iter().enumerate() {
			let port = u16::try_from(offset).unwrap();
			let result_key = (result, repeat_scope);

			self.alloc_slow(assignments, result_key);

			assignments.insert((Link(id, port), scope), assignments[&result_key]);
		}
	}

	fn assign_scope(
		&mut self,
		assignments: &mut HashMap<ScopedLink, Local>,
		nodes: &[Node],
		scope: usize,
	) {
		for (id, node) in nodes.iter().enumerate() {
			let id = u32::try_from(id).unwrap();

			self.assign_preferred_ports(assignments, id, scope);

			if let Node::Match(arc) = node {
				self.assign_match(assignments, id, scope, arc);
			} else if let Node::Repeat(arc) = node {
				self.assign_repeat(assignments, id, scope, arc);
			}
		}
	}

	fn register_boundary_preferences(&mut self, scope: usize, boundary_id: u32, port_count: u16) {
		for port in 0..port_count {
			let _ = self
				.preferences
				.try_insert((Link(boundary_id, port), scope), (Link::DANGLING, 0));
		}
	}

	#[expect(
		clippy::too_many_arguments,
		reason = "private function with inherently distinct parameters"
	)]
	fn handle_function(
		&mut self,
		stack_sizes: &mut HashMap<usize, u16>,
		assignments: &mut HashMap<ScopedLink, Local>,
		nodes: &[Node],
		scope: usize,
		roots: &[Link],
		capture_count: u16,
		argument_count: u16,
	) {
		self.run_finders(nodes, scope, roots);

		self.register_boundary_preferences(scope, 0, capture_count);
		self.register_boundary_preferences(scope, 1, argument_count);

		self.next_offset = 0;

		self.assign_scope(assignments, nodes, scope);

		stack_sizes.insert(scope, self.next_offset);
	}

	#[expect(
		clippy::too_many_arguments,
		reason = "allocator entry point requires all allocation state"
	)]
	pub fn run(
		&mut self,
		stack_sizes: &mut HashMap<usize, u16>,
		assignments: &mut HashMap<ScopedLink, Local>,
		module_nodes: &[Node],
		module_scope: usize,
		module_state: Link,
	) {
		self.preferences.clear();
		self.functions.clear();

		Self::find_functions_in(&mut self.functions, module_nodes);

		stack_sizes.clear();
		assignments.clear();

		self.handle_function(
			stack_sizes,
			assignments,
			module_nodes,
			module_scope,
			&[module_state],
			0,
			0,
		);

		while let Some(arc) = self.functions.pop() {
			let function = arc.lock();
			let scope = Arc::as_ptr(&arc) as usize;

			self.handle_function(
				stack_sizes,
				assignments,
				&function.nodes,
				scope,
				&function.results().sources,
				function.capture_count(),
				function.argument_count(),
			);
		}
	}
}
