//! Inverts correlated head-controlled loops around their controlling Match.

use alloc::sync::Arc;
use core::mem;

use hashbrown::{HashMap, hash_map::Entry};
use parking_lot::Mutex;

use ir_graph::{
	Link, Node,
	operation::{
		Identity,
		integer::{CompareOperation, CompareOperator, Type},
	},
	region::{Branch, Match, Repeat},
};

use crate::analysis::{RepeatMatchCorrelation, analyze_repeat_match};

/// Own reusable working storage for head-controlled loop inversion.
#[derive(Default)]
pub struct HeadControlledLoopInverter {
	entry_ports: HashMap<Link, u16>,
}

impl HeadControlledLoopInverter {
	/// Create an inverter.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	/// Invert correlated head-controlled loops; compact a changed region before consuming it.
	#[must_use = "propagate whether this pass changed the graph"]
	pub fn run(&mut self, nodes: &mut Vec<Node>) -> bool {
		let node_count = nodes.len();
		let mut changed = false;

		for position in 0..node_count {
			changed |= try_invert(nodes, position, self);
		}

		changed
	}

	/// Apply a correlation produced for the Repeat at one node position.
	#[must_use = "propagate whether this pass changed the graph"]
	pub fn invert(
		&mut self,
		nodes: &mut Vec<Node>,
		position: usize,
		correlation: RepeatMatchCorrelation,
	) -> bool {
		let Node::Repeat(repeat) = &nodes[position] else {
			return false;
		};
		let repeat = Arc::clone(repeat);

		invert_repeat(nodes, position, &repeat, correlation, self)
	}

	fn try_add_entry_source(&mut self, source: Link, next_port: &mut usize) -> bool {
		let Entry::Vacant(entry) = self.entry_ports.entry(source) else {
			return true;
		};

		if *next_port == usize::from(u16::MAX) {
			return false;
		}

		let port = u16::try_from(*next_port).unwrap();

		entry.insert(port);
		*next_port += 1;

		true
	}
}

fn configure_matcher_arguments(
	inverter: &mut HeadControlledLoopInverter,
	repeat: &Repeat,
	matcher_identifier: u32,
	matcher: &mut Match,
) -> bool {
	inverter.entry_ports.clear();
	inverter.entry_ports.extend(
		matcher
			.arguments
			.iter()
			.enumerate()
			.map(|(port, &source)| (source, u16::try_from(port).unwrap())),
	);
	let mut next_port = matcher.arguments.len();
	let mut has_capacity = true;
	let first_suffix = usize::try_from(matcher_identifier + 1).unwrap();

	for node in &repeat.nodes[first_suffix..repeat.results_index()] {
		node.for_each_outer(|source| {
			if has_capacity && source.0 < matcher_identifier {
				has_capacity = inverter.try_add_entry_source(source, &mut next_port);
			}
		});
	}

	for &source in &repeat.results().sources {
		if has_capacity && source.0 < matcher_identifier {
			has_capacity = inverter.try_add_entry_source(source, &mut next_port);
		}
	}

	if !has_capacity {
		return false;
	}

	let original_argument_count = matcher.arguments.len();
	matcher.arguments.resize(next_port, Link::DANGLING);

	for (&source, &port) in &inverter.entry_ports {
		let port = usize::from(port);
		if port >= original_argument_count {
			matcher.arguments[port] = source;
		}
	}

	true
}

fn remap_pulled_source(
	inverter: &HeadControlledLoopInverter,
	matcher_identifier: u32,
	first_copy: u32,
	branch_outputs: &[Link],
	source: Link,
) -> Link {
	if source.0 < matcher_identifier {
		return Link(Branch::ARGUMENTS_ID, inverter.entry_ports[&source]);
	}

	if source.0 == matcher_identifier {
		return branch_outputs[usize::from(source.1)];
	}

	Link(first_copy + source.0 - matcher_identifier - 1, source.1)
}

fn route_suffix_into_branch(
	suffix: impl Iterator<Item = Node>,
	repeat_outputs: &[Link],
	matcher_identifier: u32,
	inverter: &HeadControlledLoopInverter,
	branch: &mut Branch,
) {
	let Some(Node::BranchResults(mut results)) = branch.nodes.pop() else {
		unreachable!()
	};
	let branch_outputs = mem::take(&mut results.sources);
	let first_copy = u32::try_from(branch.nodes.len()).unwrap();

	for mut node in suffix {
		node.for_each_mut_outer(|source| {
			*source = remap_pulled_source(
				inverter,
				matcher_identifier,
				first_copy,
				&branch_outputs,
				*source,
			);
		});
		branch.nodes.push(node);
	}

	results.sources.extend(repeat_outputs.iter().map(|&source| {
		remap_pulled_source(
			inverter,
			matcher_identifier,
			first_copy,
			&branch_outputs,
			source,
		)
	}));
	branch.nodes.push(Node::BranchResults(results));
}

fn pull_suffix_into_match_branches(
	repeat: &mut Repeat,
	matcher_identifier: u32,
	inverter: &HeadControlledLoopInverter,
	matcher: &Match,
) {
	let argument_count = matcher.argument_count();
	let first_suffix = usize::try_from(matcher_identifier + 1).unwrap();
	let results_index = repeat.results_index();
	let [copy_branch, move_branch] = matcher.branches.as_slice() else {
		unreachable!()
	};

	{
		let mut branch = copy_branch.lock();

		route_suffix_into_branch(
			repeat.nodes[first_suffix..results_index]
				.iter()
				.map(Node::duplicate_operation),
			&repeat.results().sources,
			matcher_identifier,
			inverter,
			&mut branch,
		);
		branch.set_argument_count(argument_count);
	}

	let repeat_outputs = mem::take(&mut repeat.results_mut().sources);
	{
		let mut branch = move_branch.lock();

		route_suffix_into_branch(
			repeat.nodes.drain(first_suffix..results_index),
			&repeat_outputs,
			matcher_identifier,
			inverter,
			&mut branch,
		);
		branch.set_argument_count(argument_count);
	}
}

fn try_pull_suffix(
	repeat: &mut Repeat,
	matcher_identifier: u32,
	inverter: &mut HeadControlledLoopInverter,
) -> Option<Arc<Mutex<Match>>> {
	let matcher_index = usize::try_from(matcher_identifier).unwrap();
	let Node::Match(matcher) = mem::take(&mut repeat.nodes[matcher_index]) else {
		unreachable!()
	};
	let mut matcher_guard = matcher.lock();
	if !configure_matcher_arguments(inverter, repeat, matcher_identifier, &mut matcher_guard) {
		drop(matcher_guard);
		repeat.nodes[matcher_index] = Node::Match(matcher);
		return None;
	}
	pull_suffix_into_match_branches(repeat, matcher_identifier, inverter, &matcher_guard);

	drop(matcher_guard);
	repeat.nodes.remove(matcher_index);

	Some(matcher)
}

const fn remap_non_argument_link(offset: u32, source: Link) -> Link {
	Link(offset + source.0 - 1, source.1)
}

fn remap_region_link(offset: u32, argument_sources: &[Link], source: Link) -> Link {
	if source.0 == 0 {
		return argument_sources[usize::from(source.1)];
	}

	remap_non_argument_link(offset, source)
}

fn copy_head(repeat: &Repeat, target_nodes: &mut Vec<Node>) -> u32 {
	let offset = u32::try_from(target_nodes.len()).unwrap();

	for node in &repeat.nodes[1..repeat.results_index()] {
		let mut copy = node.duplicate_operation();

		copy.for_each_mut_outer(|source| {
			*source = remap_region_link(offset, &repeat.arguments, *source);
		});

		target_nodes.push(copy);
	}

	offset
}

fn prepend_repetition_body(
	repeat_nodes: &mut Vec<Node>,
	repetition_branch: &mut Branch,
) -> (Vec<Link>, u32) {
	let Some(Node::BranchResults(mut results)) = repetition_branch.nodes.pop() else {
		unreachable!()
	};
	let sources = mem::take(&mut results.sources);
	let head_offset = u32::try_from(repetition_branch.nodes.len()).unwrap();

	repeat_nodes.splice(1..1, repetition_branch.nodes.drain(1..));
	repetition_branch.nodes.push(Node::BranchResults(results));

	(sources, head_offset)
}

fn remap_inner_head_link(head_offset: u32, repetition_results: &[Link], source: Link) -> Link {
	if source.0 == Repeat::ARGUMENTS_ID {
		return repetition_results[usize::from(source.1)];
	}

	remap_non_argument_link(head_offset, source)
}

fn remap_inner_head(head_nodes: &mut [Node], head_offset: u32, repetition_results: &[Link]) {
	for node in head_nodes {
		node.for_each_mut_outer(|source| {
			*source = remap_inner_head_link(head_offset, repetition_results, *source);
		});
	}
}

fn build_repeat_condition(
	nodes: &mut Vec<Node>,
	selector: Link,
	correlation: RepeatMatchCorrelation,
) -> Link {
	match correlation {
		RepeatMatchCorrelation::FalseBranchRepeats { .. } => {
			let zero = Node::add_i32_into(nodes, 0);

			CompareOperation::add_into(nodes, selector, zero, Type::I32, CompareOperator::Equal)
		}
		RepeatMatchCorrelation::TrueBranchRepeats { .. } => selector,
	}
}

fn configure_inner_repeat(
	repeat: &mut Repeat,
	matcher: &Match,
	correlation: RepeatMatchCorrelation,
	repetition_branch: &mut Branch,
) -> Vec<Link> {
	let (repetition_results, head_offset) =
		prepend_repetition_body(&mut repeat.nodes, repetition_branch);
	let head_start = usize::try_from(head_offset).unwrap();
	let results_index = repeat.results_index();
	let sources = matcher
		.arguments
		.iter()
		.map(|&source| remap_inner_head_link(head_offset, &repetition_results, source))
		.collect();
	let selector = remap_inner_head_link(head_offset, &repetition_results, matcher.condition);

	remap_inner_head(
		&mut repeat.nodes[head_start..results_index],
		head_offset,
		&repetition_results,
	);
	let Some(Node::RepeatResults(mut results)) = repeat.nodes.pop() else {
		unreachable!()
	};

	let condition = build_repeat_condition(&mut repeat.nodes, selector, correlation);
	results.sources = sources;
	results.condition = condition;
	repeat.nodes.push(Node::RepeatResults(results));

	repetition_results
}

fn route_inner_arguments(repeat: &mut Repeat, argument_count: u16) {
	repeat.arguments.clear();
	repeat
		.arguments
		.extend((0..argument_count).map(|port| Link(Branch::ARGUMENTS_ID, port)));
	repeat.set_argument_count(argument_count);
}

fn route_repetition_branch(
	branch: &mut Branch,
	argument_count: u16,
	repeat: Arc<Mutex<Repeat>>,
	mut reusable_sources: Vec<Link>,
) {
	let repeat_identifier = Branch::ARGUMENTS_ID + 1;

	branch.nodes.insert(
		usize::try_from(repeat_identifier).unwrap(),
		Node::Repeat(repeat),
	);
	reusable_sources.clear();
	reusable_sources.extend((0..argument_count).map(|port| Link(repeat_identifier, port)));
	branch.results_mut().sources = reusable_sources;
}

fn remap_outer_match(matcher: &mut Match, head_offset: u32, repeat_arguments: &[Link]) {
	for source in &mut matcher.arguments {
		*source = remap_region_link(head_offset, repeat_arguments, *source);
	}

	matcher.condition = remap_region_link(head_offset, repeat_arguments, matcher.condition);
}

const fn remap_moved_link(offset: u32, argument_identifier: u32, source: Link) -> Link {
	if source.0 == Branch::ARGUMENTS_ID {
		return Link(argument_identifier, source.1);
	}

	remap_non_argument_link(offset, source)
}

fn move_exit_branch(
	branch: &mut Branch,
	target_nodes: &mut Vec<Node>,
	argument_identifier: u32,
) -> Identity {
	let Some(Node::BranchResults(mut results)) = branch.nodes.pop() else {
		unreachable!()
	};
	let offset = u32::try_from(target_nodes.len()).unwrap();

	for mut node in branch.nodes.drain(1..) {
		node.for_each_mut_outer(|source| {
			*source = remap_moved_link(offset, argument_identifier, *source);
		});

		target_nodes.push(node);
	}

	let replacement = Identity {
		sources: results
			.sources
			.iter()
			.map(|&source| remap_moved_link(offset, argument_identifier, source))
			.collect(),
	};

	let argument_count = branch.argument_count();

	results.sources.clear();
	results
		.sources
		.extend((0..argument_count).map(|port| Link(Branch::ARGUMENTS_ID, port)));
	branch.nodes.push(Node::BranchResults(results));

	replacement
}

fn candidate_branches(
	matcher: &Match,
	correlation: RepeatMatchCorrelation,
) -> (Arc<Mutex<Branch>>, Arc<Mutex<Branch>>) {
	match correlation {
		RepeatMatchCorrelation::FalseBranchRepeats { .. } => (
			Arc::clone(&matcher.branches[1]),
			Arc::clone(&matcher.branches[0]),
		),
		RepeatMatchCorrelation::TrueBranchRepeats { .. } => (
			Arc::clone(&matcher.branches[0]),
			Arc::clone(&matcher.branches[1]),
		),
	}
}

fn configure_inverted_repeat(
	target_nodes: &mut Vec<Node>,
	repeat: &Arc<Mutex<Repeat>>,
	repeat_guard: &mut Repeat,
	matcher: &Arc<Mutex<Match>>,
	correlation: RepeatMatchCorrelation,
) -> Arc<Mutex<Branch>> {
	let outer_head_offset = copy_head(repeat_guard, target_nodes);
	let mut matcher_guard = matcher.lock();
	let (exit_branch, repetition_branch) = candidate_branches(&matcher_guard, correlation);
	let mut repetition_guard = repetition_branch.lock();
	let repetition_output_storage = configure_inner_repeat(
		repeat_guard,
		&matcher_guard,
		correlation,
		&mut repetition_guard,
	);
	remap_outer_match(
		&mut matcher_guard,
		outer_head_offset,
		&repeat_guard.arguments,
	);
	route_inner_arguments(repeat_guard, matcher_guard.argument_count());
	route_repetition_branch(
		&mut repetition_guard,
		matcher_guard.argument_count(),
		Arc::clone(repeat),
		repetition_output_storage,
	);

	drop(repetition_guard);
	drop(matcher_guard);

	exit_branch
}

fn invert_repeat(
	target_nodes: &mut Vec<Node>,
	position: usize,
	repeat: &Arc<Mutex<Repeat>>,
	correlation: RepeatMatchCorrelation,
	inverter: &mut HeadControlledLoopInverter,
) -> bool {
	let matcher_identifier = correlation.match_identifier();
	let mut repeat_guard = repeat.lock();

	let Some(matcher) = try_pull_suffix(&mut repeat_guard, matcher_identifier, inverter) else {
		return false;
	};
	let exit_branch = configure_inverted_repeat(
		target_nodes,
		repeat,
		&mut repeat_guard,
		&matcher,
		correlation,
	);

	drop(repeat_guard);

	replace_repeat(target_nodes, position, matcher, &exit_branch);

	true
}

fn replace_repeat(
	target_nodes: &mut Vec<Node>,
	position: usize,
	matcher: Arc<Mutex<Match>>,
	exit_branch: &Arc<Mutex<Branch>>,
) {
	let outer_matcher_identifier = u32::try_from(target_nodes.len()).unwrap();

	target_nodes.push(Node::Match(matcher));

	let replacement = move_exit_branch(
		&mut exit_branch.lock(),
		target_nodes,
		outer_matcher_identifier,
	);

	target_nodes[position] = Node::Identity(replacement);
}

fn try_invert(
	nodes: &mut Vec<Node>,
	position: usize,
	inverter: &mut HeadControlledLoopInverter,
) -> bool {
	let correlation = {
		let Node::Repeat(repeat) = &nodes[position] else {
			return false;
		};
		let repeat = repeat.lock();
		let Some(correlation) = analyze_repeat_match(&repeat) else {
			return false;
		};

		correlation
	};

	inverter.invert(nodes, position, correlation)
}
