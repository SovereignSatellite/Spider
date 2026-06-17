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

/// Traces the origin of an output port on a match node.
///
/// Returns the parent-scope `Link` every branch forwards to this result port,
/// or `None` if a branch computes the value or the branches disagree.
/// Identity pass-throughs are followed, so the boundary identities resolve to
/// their origin.
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

/// Traces the origin of an output port on a repeat node.
///
/// Returns the origin if the carry is invariant around its rotation cycle, or
/// `None` if it is computed or the cycle disagrees. After a successful call,
/// `visited` holds every port in the cycle. Identity pass-throughs are followed.
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
