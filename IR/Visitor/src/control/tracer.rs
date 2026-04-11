//! Region output origin tracing.

use set::Set;

use ir_graph::{
	Link,
	control::{Match, Repeat},
};

fn find_argument(source: Link, arguments: &[Link]) -> Option<Link> {
	(source.0 == 0).then(|| arguments[usize::from(source.1)])
}

/// Traces the origin of an output port on a match node.
///
/// Returns the parent-scope `Link` if all branches produce the same
/// argument for this port, or `None` if the value is computed or the
/// branches disagree.
///
/// Assumes identity nodes have been removed from all regions.
#[must_use]
pub fn trace_match(matcher: &Match, port: u16) -> Option<Link> {
	let port = usize::from(port);
	let mut branches = matcher.branches.iter();

	let origin = {
		let guard = branches.next()?.lock();

		find_argument(guard.results().sources[port], &matcher.arguments)?
	};

	for branch in branches {
		let guard = branch.lock();

		if find_argument(guard.results().sources[port], &matcher.arguments) != Some(origin) {
			return None;
		}
	}

	Some(origin)
}

/// Traces the origin of an output port on a repeat node.
///
/// Returns the origin if the port is invariant, or `None` if the value is
/// computed or the cycle arguments disagree. After a successful call,
/// `visited` contains all ports in the cycle.
///
/// Assumes identity nodes have been removed from all regions.
pub fn trace_repeat(visited: &mut Set, repeat: &Repeat, port: u16) -> Option<Link> {
	visited.clear();

	let port = usize::from(port);
	let results = &repeat.results().sources;
	let origin = repeat.arguments[port];
	let mut current = port;

	loop {
		if visited.grow_insert(current) {
			return Some(origin);
		}

		if find_argument(results[current], &repeat.arguments) != Some(origin) {
			return None;
		}

		current = usize::from(results[current].1);
	}
}
