//! Recursive, post-order region traversal driver.

use alloc::sync::Arc;

use parking_lot::Mutex;

use crate::{
	Node, Region, Shape,
	node::region::{Branch, Function, Match, Repeat},
};

fn run_region<H>(region: Region, handler: &mut H)
where
	H: FnMut(Region),
{
	stacker::maybe_grow(0x1_0000, 0x10_0000, || {
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
