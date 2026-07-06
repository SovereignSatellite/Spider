//! Recursive, post-order region traversal driver.

use alloc::sync::Arc;

use parking_lot::Mutex;

use crate::{
	Node, Region, Shape,
	node::region::{Branch, Function, Match, Repeat},
};

const STACK_RED_ZONE: usize = 64 * 1024;
const STACK_SEGMENT: usize = 1024 * 1024;

fn run_region<H>(region: Region, handler: &mut H)
where
	H: FnMut(Region),
{
	stacker::maybe_grow(STACK_RED_ZONE, STACK_SEGMENT, || {
		for node in region.nodes() {
			run_node(node, handler);
		}

		handler(region);
	});
}

/// Visits every region in the function, deepest first.
pub fn run_function<H>(function: &Arc<Mutex<Function>>, handler: &mut H)
where
	H: FnMut(Region),
{
	let guard = Mutex::lock_arc(function);

	run_region(Region::Function(guard), handler);
}

fn run_branch<H>(region: &Arc<Mutex<Branch>>, handler: &mut H)
where
	H: FnMut(Region),
{
	let guard = Mutex::lock_arc(region);

	run_region(Region::Branch(guard), handler);
}

fn run_match<H>(region: &Arc<Mutex<Match>>, handler: &mut H)
where
	H: FnMut(Region),
{
	let guard = region.lock();

	for branch in &guard.branches {
		run_branch(branch, handler);
	}
}

fn run_repeat<H>(region: &Arc<Mutex<Repeat>>, handler: &mut H)
where
	H: FnMut(Region),
{
	let guard = Mutex::lock_arc(region);

	run_region(Region::Repeat(guard), handler);
}

/// Visits every region reachable from `node`, deepest first.
pub fn run_node<H>(node: &Node, handler: &mut H)
where
	H: FnMut(Region),
{
	match node.shape() {
		Shape::Plain | Shape::BranchResults(_) | Shape::RepeatResults(_) => {}
		Shape::Function(region) => run_function(region, handler),
		Shape::Match(region) => run_match(region, handler),
		Shape::Repeat(region) => run_repeat(region, handler),
	}
}
