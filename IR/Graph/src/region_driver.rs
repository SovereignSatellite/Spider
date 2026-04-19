//! Bottom-up recursive driver for region-local passes.

use alloc::sync::Arc;

use parking_lot::Mutex;

use crate::{
	Node, Region,
	node::region::{Branch, Function, Match, Module, Repeat},
};

fn run_region<H>(region: Region, handler: &mut H)
where
	H: FnMut(Region),
{
	for node in region.nodes() {
		run_node(node, handler);
	}

	handler(region);
}

/// Applies `handle` to every region in the module, deepest first.
pub fn run_module<H>(module: &Arc<Mutex<Module>>, handler: &mut H)
where
	H: FnMut(Region),
{
	let guard = Mutex::lock_arc(module);

	run_region(Region::Module(guard), handler);
}

fn run_function<H>(region: &Arc<Mutex<Function>>, handler: &mut H)
where
	H: FnMut(Region),
{
	let guard = Mutex::lock_arc(region);

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

/// Recursively applies `handle` to any region nested inside `node`.
#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
pub fn run_node<H>(node: &Node, handler: &mut H)
where
	H: FnMut(Region),
{
	match node {
		Node::Function(region) => run_function(region, handler),
		Node::Match(region) => run_match(region, handler),
		Node::Repeat(region) => run_repeat(region, handler),

		Node::ModuleArguments(_)
		| Node::ModuleResults(_)
		| Node::FunctionCaptures(_)
		| Node::FunctionArguments(_)
		| Node::FunctionResults(_)
		| Node::BranchArguments(_)
		| Node::BranchResults(_)
		| Node::RepeatArguments(_)
		| Node::RepeatResults(_)
		| Node::Import(_)
		| Node::Foreign(_)
		| Node::Trap
		| Node::Null
		| Node::I32(_)
		| Node::I64(_)
		| Node::F32(_)
		| Node::F64(_)
		| Node::Identity(_)
		| Node::Fence(_)
		| Node::Apply(_)
		| Node::RefIsNull(_)
		| Node::IntegerUnaryOperation(_)
		| Node::IntegerBinaryOperation(_)
		| Node::IntegerCompareOperation(_)
		| Node::IntegerNarrow(_)
		| Node::IntegerWiden(_)
		| Node::IntegerSignExtend(_)
		| Node::IntegerConvertToNumber(_)
		| Node::IntegerTransmuteToNumber(_)
		| Node::NumberUnaryOperation(_)
		| Node::NumberBinaryOperation(_)
		| Node::NumberCompareOperation(_)
		| Node::NumberNarrow(_)
		| Node::NumberWiden(_)
		| Node::NumberTruncateToInteger(_)
		| Node::NumberTransmuteToInteger(_)
		| Node::MutableNew(_)
		| Node::MutableGet(_)
		| Node::MutableSet(_)
		| Node::TableNew(_)
		| Node::TableGet(_)
		| Node::TableSet(_)
		| Node::TableSize(_)
		| Node::TableGrow(_)
		| Node::TableFill(_)
		| Node::TableCopy(_)
		| Node::TableDrop(_)
		| Node::MemoryNew(_)
		| Node::MemoryLoad(_)
		| Node::MemoryStore(_)
		| Node::MemorySize(_)
		| Node::MemoryGrow(_)
		| Node::MemoryFill(_)
		| Node::MemoryCopy(_)
		| Node::MemoryDrop(_) => {}
	}
}
