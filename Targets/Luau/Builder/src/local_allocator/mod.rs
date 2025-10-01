use core::ops::Range;

use alloc::vec::Vec;
use data_flow_graph::{DataFlowGraph, Link};
use hashbrown::HashMap;
use luau_tree::expression::Local;

use self::{
	argument_finder::{ArgumentFinder, get_region_range},
	local_provider::LocalProvider,
	scalar_finder::ScalarFinder,
};

mod argument_finder;
mod index_provider;
mod local_provider;
mod reference_finder;
mod scalar_finder;

pub struct Declarations {
	pub locals: Range<u32>,
	pub stack: u16,
}

pub struct LocalAllocator {
	preferences: HashMap<Link, Link>,
	functions: Vec<Range<u32>>,
	arguments: Vec<(Link, Link)>,

	provider: LocalProvider,
	scalar_finder: ScalarFinder,
	argument_finder: ArgumentFinder,
}

impl LocalAllocator {
	pub fn new() -> Self {
		Self {
			preferences: HashMap::new(),
			functions: Vec::new(),
			arguments: Vec::new(),

			provider: LocalProvider::new(),
			scalar_finder: ScalarFinder::new(),
			argument_finder: ArgumentFinder::new(),
		}
	}

	fn find_functions(&mut self, graph: &DataFlowGraph) {
		self.functions.extend(
			graph.nodes().rev().filter_map(|node| {
				get_region_range(graph, node).map(|(start, end)| start..end + 1)
			}),
		);
	}

	fn handle_arguments(
		&mut self,
		assignments: &mut HashMap<Link, Local>,
		graph: &DataFlowGraph,
		id: u32,
	) {
		let mut arguments = core::mem::take(&mut self.arguments);

		self.argument_finder
			.run(&mut arguments, &self.preferences, graph, id);

		// In the first pass we ensure all producers outside the list
		// have their variables reused.
		arguments.retain(|&(argument, preferred)| {
			argument == preferred
				|| !self
					.provider
					.try_revive_into(assignments, argument, preferred)
		});

		arguments.sort_unstable();

		// In the second pass we assign new variables where needed and
		// reuse producers within the list.
		for &(argument, preferred) in arguments.iter().rev() {
			if self
				.provider
				.try_revive_into(assignments, argument, preferred)
			{
				continue;
			}

			self.provider.try_pull_into(assignments, argument);
		}

		self.arguments = arguments;
	}

	fn handle_definitions(&mut self, assignments: &mut HashMap<Link, Local>, id: u32) {
		// We might have some locals that require assignment but have no uses, which means we must
		// manually declare them with an empty lifetime.
		for link in (0..)
			.map(|port| Link(id, port))
			.take_while(|link| self.preferences.contains_key(link))
		{
			self.provider.try_pull_into(assignments, link);
		}
	}

	fn handle_node(
		&mut self,
		assignments: &mut HashMap<Link, Local>,
		graph: &DataFlowGraph,
		id: u32,
	) -> u32 {
		let (start, end) = get_region_range(graph, graph.get(id)).unwrap_or((id, id));

		self.handle_definitions(assignments, end);

		self.provider.push_until(start);

		self.handle_arguments(assignments, graph, start);

		start
	}

	fn handle_scope(
		&mut self,
		assignments: &mut HashMap<Link, Local>,
		graph: &DataFlowGraph,
		mut range: Range<u32>,
	) {
		self.handle_definitions(assignments, range.next().unwrap());
		self.handle_arguments(assignments, graph, range.next_back().unwrap());

		while let Some(id) = range.next_back() {
			range.end = self.handle_node(assignments, graph, id);
		}
	}

	fn handle_function(
		&mut self,
		assignments: &mut HashMap<Link, Local>,
		graph: &DataFlowGraph,
		range: Range<u32>,
	) -> Declarations {
		let (first, _) = self.provider.get_names();

		self.handle_scope(assignments, graph, range);

		let (last, table) = self.provider.get_names();

		self.provider.refresh();

		Declarations {
			locals: first..last,
			stack: table.try_into().unwrap(),
		}
	}

	pub fn run(
		&mut self,
		declarations: &mut HashMap<u32, Declarations>,
		assignments: &mut HashMap<Link, Local>,
		graph: &DataFlowGraph,
	) {
		self.preferences.clear();
		self.provider.clear();
		self.argument_finder.clear();

		reference_finder::run(&mut self.preferences, graph);
		self.scalar_finder.run(&mut self.preferences, graph);

		self.find_functions(graph);

		declarations.clear();
		assignments.clear();

		while let Some(range) = self.functions.pop() {
			let declaration = self.handle_function(assignments, graph, range.clone());

			declarations.insert(range.start, declaration);
		}
	}
}
