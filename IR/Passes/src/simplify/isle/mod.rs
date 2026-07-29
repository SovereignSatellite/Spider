//! ISLE-based peephole optimizations.

use ir_graph::{Link, Node, operation::Identity};

use self::{
	context::RegionContext,
	internal::{
		Links, constructor_SimplifyAggregate, constructor_SimplifyConvert,
		constructor_SimplifyFloat, constructor_SimplifyI32, constructor_SimplifyI64,
		constructor_SimplifyLuauArithmetic, constructor_SimplifyLuauBit32,
		constructor_SimplifyLuauCompare, constructor_SimplifyLuauMath,
		constructor_SimplifyLuauTransmute, constructor_SimplifyLuauWide,
		constructor_SimplifyMemory, constructor_SimplifyMutable, constructor_SimplifyReference,
		constructor_SimplifyTable,
	},
};

pub use self::match_output_reducer::reduce_match_outputs;

mod context;
mod internal;
mod luau;
mod match_output_reducer;

fn replace_node(nodes: &mut [Node], destination: u32, sources: &[Link]) {
	let sources = sources.iter().copied().collect();

	nodes[usize::try_from(destination).unwrap()] = Node::Identity(Identity { sources });
}

fn simplify_single<Constructor>(nodes: &mut Vec<Node>, id: u32, constructor: Constructor) -> bool
where
	Constructor: FnOnce(&mut RegionContext<'_>, Link) -> Option<Link>,
{
	constructor(&mut RegionContext(nodes), Link(id, 0)).is_some_and(|source| {
		replace_node(nodes, id, &[source]);

		true
	})
}

fn simplify_multi<Constructor>(nodes: &mut Vec<Node>, id: u32, constructor: Constructor) -> bool
where
	Constructor: FnOnce(&mut RegionContext<'_>, Link) -> Option<Links>,
{
	constructor(&mut RegionContext(nodes), Link(id, 0)).is_some_and(|sources| {
		replace_node(nodes, id, &sources.as_fixed());

		true
	})
}

fn simplify(nodes: &mut Vec<Node>, id: u32) -> bool {
	simplify_single(nodes, id, |context, link| {
		constructor_SimplifyI32(context, link)
	}) || simplify_single(nodes, id, |context, link| {
		constructor_SimplifyI64(context, link)
	}) || simplify_single(nodes, id, |context, link| {
		constructor_SimplifyConvert(context, link)
	}) || simplify_single(nodes, id, |context, link| {
		constructor_SimplifyFloat(context, link)
	}) || simplify_single(nodes, id, |context, link| {
		constructor_SimplifyAggregate(context, link)
	}) || simplify_single(nodes, id, |context, link| {
		constructor_SimplifyReference(context, link)
	}) || simplify_multi(nodes, id, |context, link| {
		constructor_SimplifyMutable(context, link)
	}) || simplify_multi(nodes, id, |context, link| {
		constructor_SimplifyTable(context, link)
	}) || simplify_multi(nodes, id, |context, link| {
		constructor_SimplifyMemory(context, link)
	}) || simplify_single(nodes, id, |context, link| {
		constructor_SimplifyLuauBit32(context, link)
	}) || simplify_single(nodes, id, |context, link| {
		constructor_SimplifyLuauArithmetic(context, link)
	}) || simplify_single(nodes, id, |context, link| {
		constructor_SimplifyLuauCompare(context, link)
	}) || simplify_single(nodes, id, |context, link| {
		constructor_SimplifyLuauMath(context, link)
	}) || simplify_single(nodes, id, |context, link| {
		constructor_SimplifyLuauTransmute(context, link)
	}) || simplify_multi(nodes, id, |context, link| {
		constructor_SimplifyLuauWide(context, link)
	})
}

/// Sweeps every node once, reporting whether any rule fired.
#[must_use = "propagate whether this pass changed the graph"]
pub fn run(nodes: &mut Vec<Node>) -> bool {
	let mut applied = false;

	let Ok(last) = u32::try_from(nodes.len()) else {
		unreachable!()
	};

	for id in (0..last).rev() {
		applied |= simplify(nodes, id);
	}

	applied
}
