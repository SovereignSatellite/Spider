//! Repeat and Match predicate correlation.

use alloc::sync::Arc;

use parking_lot::Mutex;

use ir_graph::{
	Link, Node,
	region::{Branch, Match, Repeat},
	tracer,
};

/// Certify which branch of a binary Match repeats its enclosing Repeat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepeatMatchCorrelation {
	/// Repeat through the Match's false branch.
	FalseBranchRepeats {
		/// Identify the correlated Match within the Repeat.
		match_identifier: u32,
	},
	/// Repeat through the Match's true branch.
	TrueBranchRepeats {
		/// Identify the correlated Match within the Repeat.
		match_identifier: u32,
	},
}

impl RepeatMatchCorrelation {
	/// Return the correlated Match's identifier in the analyzed graph state.
	#[must_use]
	pub const fn match_identifier(self) -> u32 {
		match self {
			Self::FalseBranchRepeats { match_identifier }
			| Self::TrueBranchRepeats { match_identifier } => match_identifier,
		}
	}
}

enum PredicateRelation {
	Direct(Link),
	Inverted,
}

impl PredicateRelation {
	const fn redirected_condition(self) -> Option<Link> {
		match self {
			Self::Direct(condition) => Some(condition),
			Self::Inverted => None,
		}
	}
}

fn match_at(repeat: &Repeat, identifier: u32) -> Option<&Arc<Mutex<Match>>> {
	let Node::Match(matcher) = &repeat.nodes[usize::try_from(identifier).unwrap()] else {
		return None;
	};

	Some(matcher)
}

fn branch_alternative(branch: &Arc<Mutex<Branch>>, output_port: u16) -> Option<i32> {
	let branch = branch.lock();
	let source = branch.results().sources[usize::from(output_port)];
	let source = tracer::identity_source(&branch.nodes, source);
	let Node::I32(alternative) = branch.nodes[usize::try_from(source.0).unwrap()] else {
		return None;
	};

	drop(branch);

	Some(alternative)
}

fn constant_alternatives(matcher: &Match, output_port: u16) -> Option<[i32; 2]> {
	let [on_false, on_true] = matcher.branches.as_slice() else {
		return None;
	};

	Some([
		branch_alternative(on_false, output_port)?,
		branch_alternative(on_true, output_port)?,
	])
}

fn selected_branch_alternative(matcher: &Match, selector: i32, output_port: u16) -> Option<i32> {
	let branch_index = usize::try_from(selector).ok()?;
	let branch = matcher.branches.get(branch_index)?;

	branch_alternative(branch, output_port)
}

fn condition_alternatives(
	repeat: &Repeat,
	condition: Link,
	match_identifier: u32,
	matcher: &Match,
	match_condition: Link,
) -> Option<[i32; 2]> {
	if condition == match_condition {
		return Some([0_i32, 1_i32]);
	}

	if condition.0 == match_identifier {
		return constant_alternatives(matcher, condition.1);
	}

	let mapping_match = match_at(repeat, condition.0)?;
	let selector_output = tracer::identity_source(&repeat.nodes, mapping_match.lock().condition);

	if selector_output.0 != match_identifier {
		return None;
	}

	let selectors = constant_alternatives(matcher, selector_output.1)?;
	let mapping_matcher = mapping_match.lock();

	Some([
		selected_branch_alternative(&mapping_matcher, selectors[0], condition.1)?,
		selected_branch_alternative(&mapping_matcher, selectors[1], condition.1)?,
	])
}

fn analyze_at(
	repeat: &Repeat,
	repeat_condition: Link,
	match_identifier: u32,
) -> Option<PredicateRelation> {
	let matcher = match_at(repeat, match_identifier)?.lock();
	let match_condition = tracer::identity_source(&repeat.nodes, matcher.condition);
	let alternatives = condition_alternatives(
		repeat,
		repeat_condition,
		match_identifier,
		&matcher,
		match_condition,
	)?;
	drop(matcher);

	match alternatives {
		[0_i32, 1_i32] => Some(PredicateRelation::Direct(match_condition)),
		[1_i32, 0_i32] => Some(PredicateRelation::Inverted),
		_ => None,
	}
}

fn find_redirect(repeat: &Repeat, repeat_condition: Link) -> Option<Link> {
	if let Some(condition) = analyze_at(repeat, repeat_condition, repeat_condition.0)
		.and_then(PredicateRelation::redirected_condition)
	{
		return Some(condition);
	}

	let mapping_match = match_at(repeat, repeat_condition.0)?;
	let output = tracer::identity_source(&repeat.nodes, mapping_match.lock().condition);

	analyze_at(repeat, repeat_condition, output.0)?.redirected_condition()
}

/// Find the condition reached by repeat-match correlation.
#[must_use]
pub fn correlated_repeat_condition(repeat: &Repeat) -> Link {
	let mut condition = tracer::identity_source(&repeat.nodes, repeat.results().condition);

	while let Some(redirected) = find_redirect(repeat, condition) {
		condition = redirected;
	}

	condition
}

/// Find the binary Match that controls a Repeat.
#[must_use]
pub fn analyze_repeat_match(repeat: &Repeat) -> Option<RepeatMatchCorrelation> {
	let repeat_condition = correlated_repeat_condition(repeat);

	repeat.nodes[..repeat.results_index()]
		.iter()
		.enumerate()
		.rev()
		.find_map(|(index, node)| {
			if !matches!(node, Node::Match(_)) {
				return None;
			}

			let match_identifier = u32::try_from(index).ok()?;
			let correlation = analyze_at(repeat, repeat_condition, match_identifier)?;

			Some(match correlation {
				PredicateRelation::Direct(_) => {
					RepeatMatchCorrelation::TrueBranchRepeats { match_identifier }
				}
				PredicateRelation::Inverted => {
					RepeatMatchCorrelation::FalseBranchRepeats { match_identifier }
				}
			})
		})
}
