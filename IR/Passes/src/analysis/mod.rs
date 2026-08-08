//! Read-only graph queries.

pub use self::{
	repeat_match_correlation::{
		RepeatMatchCorrelation, analyze_repeat_match, correlated_repeat_condition,
	},
	successor_finder::{Successor, SuccessorFinder},
};

mod repeat_match_correlation;
mod successor_finder;
