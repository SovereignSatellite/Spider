//! Region output origin tracing.

use alloc::sync::Arc;

use parking_lot::Mutex;
use set::Set;

use crate::{
	Link, Node,
	node::{
		operation::Identity,
		region::{Branch, Match, Repeat},
	},
};

/// Follows identity pass-throughs to the underlying source link.
///
/// # Panics
///
/// Panics if `link` refers to a node index outside `nodes`.
#[must_use]
pub fn identity_source(nodes: &[Node], mut link: Link) -> Link {
	while let Node::Identity(Identity { sources }) = &nodes[usize::try_from(link.0).unwrap()] {
		link = sources[usize::from(link.1)];
	}

	link
}

fn resolve_argument(source: Link, arguments: &[Link]) -> Option<Link> {
	(source.0 == 0).then(|| arguments[usize::from(source.1)])
}

fn branch_result_source(branch: &Arc<Mutex<Branch>>, port: usize) -> Link {
	let guard = branch.lock();

	identity_source(&guard.nodes, guard.results().sources[port])
}

/// Returns the parent link every branch forwards to one match output, following identities.
/// Returns `None` when any branch computes the value or the forwarded links disagree.
#[must_use]
pub fn trace_match(matcher: &Match, port: u16) -> Option<Link> {
	let port = usize::from(port);
	let mut branches = matcher.branches.iter();

	let origin = resolve_argument(
		branch_result_source(branches.next()?, port),
		&matcher.arguments,
	)?;

	for branch in branches {
		if resolve_argument(branch_result_source(branch, port), &matcher.arguments) != Some(origin)
		{
			return None;
		}
	}

	Some(origin)
}

/// Returns an invariant repeat carry's identity-resolved origin, or `None` when it disagrees.
/// On success, `visited` contains every port in the rotation cycle.
#[must_use]
pub fn trace_repeat(visited: &mut Set, repeat: &Repeat, port: u16) -> Option<Link> {
	let results = &repeat.results().sources;
	let origin = repeat.arguments[usize::from(port)];
	let mut current = usize::from(port);

	visited.clear();

	loop {
		if visited.grow_insert(current) {
			return Some(origin);
		}

		let source = identity_source(&repeat.nodes, results[current]);

		if resolve_argument(source, &repeat.arguments) != Some(origin) {
			return None;
		}

		current = usize::from(source.1);
	}
}
