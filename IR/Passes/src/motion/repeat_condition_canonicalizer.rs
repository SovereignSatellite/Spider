//! Repeat condition canonicalization.

use ir_graph::region::Repeat;

use crate::analysis::correlated_repeat_condition;

/// Redirect a Repeat condition through correlated Matches.
#[must_use = "propagate whether this pass changed the graph"]
pub fn canonicalize_repeat_condition(repeat: &mut Repeat) -> bool {
	let condition = correlated_repeat_condition(repeat);

	if condition == repeat.results().condition {
		return false;
	}

	repeat.results_mut().condition = condition;
	true
}
