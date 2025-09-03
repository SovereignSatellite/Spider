use alloc::vec::Vec;
use control_flow_graph::{ControlFlowGraph, instruction::Name};

use super::continuation_finder::ContinuationFinder;

pub struct Single {
	edges: Vec<(u16, u16)>,
	points: Vec<u16>,
	separators: Vec<usize>,

	continuation_finder: ContinuationFinder,
}

impl Single {
	pub const fn new() -> Self {
		Self {
			edges: Vec::new(),
			points: Vec::new(),
			separators: Vec::new(),

			continuation_finder: ContinuationFinder::new(),
		}
	}

	fn has_assignment_in_branch(&self, graph: &ControlFlowGraph) -> bool {
		self.points.iter().any(|&id| {
			graph.predecessors(id).any(|id| {
				graph.has_assignment(id, Name::A) && self.edges.iter().any(|edge| edge.0 == id)
			})
		})
	}

	fn has_assignment_in_tail(&self, graph: &ControlFlowGraph) -> bool {
		self.points.iter().any(|&id| {
			graph.predecessors(id).any(|id| {
				graph.has_assignment(id, Name::A) && self.edges.iter().all(|edge| edge.0 != id)
			})
		})
	}

	fn exclude_last_assignments(&mut self, graph: &ControlFlowGraph) {
		let excluded = self.points.iter().flat_map(|&id| {
			graph.predecessors(id).filter_map(|id| {
				if graph.has_assignment(id, Name::A) {
					let mut predecessors = graph.predecessors(id);

					let id = if let Some(id) = predecessors.next()
						&& predecessors.next().is_none()
						&& graph.has_assignment(id, Name::C)
					{
						id
					} else {
						id
					};

					Some(id)
				} else {
					None
				}
			})
		});

		self.continuation_finder.set_excluded(excluded);
	}

	fn find_continuations(&mut self, graph: &ControlFlowGraph, entry: u16, id: u16) {
		let mut predecessors = graph.predecessors(id).filter(|&other| other != id);

		if predecessors.next().is_some() && predecessors.next().is_some() {
			self.edges.push((entry, id));
		} else {
			self.continuation_finder.run(graph, id);
			self.continuation_finder.edges_into(graph, &mut self.edges);
		}

		self.separators.push(self.edges.len());
	}

	fn find_all_continuations(&mut self, graph: &ControlFlowGraph, entry: u16) {
		self.edges.clear();
		self.separators.clear();

		for successor in graph.successors(entry) {
			self.find_continuations(graph, entry, successor);
		}

		self.points.clear();
		self.points.extend(self.edges.iter().map(|edge| edge.1));
		self.points.sort_unstable();
		self.points.dedup();
	}

	fn set_new_continuation(&mut self, graph: &mut ControlFlowGraph) -> u16 {
		let selection = graph.add_selection(Name::A);

		for (from, to) in &mut self.edges {
			let point = self.points.binary_search(to).unwrap().try_into().unwrap();
			let assignment = graph.add_assignment(Name::A, point);

			graph.replace_edge(*from, *to, assignment);
			graph.add_edge(assignment, selection);

			*from = assignment;
			*to = selection;
		}

		for &point in &self.points {
			graph.add_edge(selection, point);
		}

		selection
	}

	fn find_or_set_continuation(&mut self, graph: &mut ControlFlowGraph, entry: u16) -> u16 {
		if let &[point] = self.points.as_slice() {
			point
		} else {
			if self.has_assignment_in_tail(graph) && self.has_assignment_in_branch(graph) {
				self.exclude_last_assignments(graph);
				self.find_all_continuations(graph, entry);
			}

			self.set_new_continuation(graph)
		}
	}

	fn set_continuation_merges(&self, graph: &mut ControlFlowGraph, point: u16) {
		let mut start = 0;

		for &end in &self.separators {
			let continuations = &self.edges[start..end];

			start = end;

			if continuations.len() > 1 {
				let dummy = graph.add_no_operation();

				for &(from, to) in continuations {
					debug_assert_eq!(point, to, "only one continuation should remain");

					graph.replace_edge(from, to, dummy);
				}

				graph.add_edge(dummy, point);
			}
		}
	}

	// We add dummy nodes to empty branches to ensure symmetry. This is done
	// last as we don't always know which branches are empty at the start.
	fn fill_empty_branches(graph: &mut ControlFlowGraph, entry: u16, point: u16) {
		let count = graph.successors(entry).filter(|&id| id == point).count();

		for _ in 0..count {
			let dummy = graph.add_no_operation();

			graph.replace_edge(entry, point, dummy);
			graph.add_edge(dummy, point);
		}
	}

	pub fn run(&mut self, graph: &mut ControlFlowGraph, entry: u16) -> u16 {
		self.continuation_finder.set_excluded(core::iter::empty());

		self.find_all_continuations(graph, entry);

		let point = self.find_or_set_continuation(graph, entry);

		self.set_continuation_merges(graph, point);
		Self::fill_empty_branches(graph, entry, point);

		point
	}
}
