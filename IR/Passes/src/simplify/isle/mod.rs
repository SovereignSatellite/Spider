//! ISLE-based peephole optimizations.

use ir_graph::{Link, Node, operation::Identity};

use self::{
	context::RegionContext,
	internal::{
		LinkPair, constructor_SimplifyAggregate, constructor_SimplifyConvert,
		constructor_SimplifyFloat, constructor_SimplifyInteger, constructor_SimplifyLuauArithmetic,
		constructor_SimplifyLuauBit32, constructor_SimplifyLuauCompare,
		constructor_SimplifyLuauMath, constructor_SimplifyLuauTransmute,
		constructor_SimplifyLuauWide, constructor_SimplifyMemory, constructor_SimplifyMutable,
		constructor_SimplifyReference, constructor_SimplifyTable,
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

fn simplify_single<Constructor>(
	nodes: &mut Vec<Node>,
	identifier: u32,
	constructor: Constructor,
) -> bool
where
	Constructor: FnOnce(&mut RegionContext<'_>, Link) -> Option<Link>,
{
	constructor(&mut RegionContext(nodes), Link(identifier, 0)).is_some_and(|source| {
		replace_node(nodes, identifier, &[source]);

		true
	})
}

fn simplify_pair<Constructor>(
	nodes: &mut Vec<Node>,
	identifier: u32,
	constructor: Constructor,
) -> bool
where
	Constructor: FnOnce(&mut RegionContext<'_>, Link) -> Option<LinkPair>,
{
	constructor(&mut RegionContext(nodes), Link(identifier, 0)).is_some_and(|sources| {
		replace_node(nodes, identifier, &sources.as_fixed());

		true
	})
}

fn simplify_foreign(nodes: &mut Vec<Node>, identifier: u32) -> bool {
	simplify_single(nodes, identifier, |context, link| {
		constructor_SimplifyLuauBit32(context, link)
	}) || simplify_single(nodes, identifier, |context, link| {
		constructor_SimplifyLuauArithmetic(context, link)
	}) || simplify_single(nodes, identifier, |context, link| {
		constructor_SimplifyLuauCompare(context, link)
	}) || simplify_single(nodes, identifier, |context, link| {
		constructor_SimplifyLuauMath(context, link)
	}) || simplify_single(nodes, identifier, |context, link| {
		constructor_SimplifyLuauTransmute(context, link)
	}) || simplify_pair(nodes, identifier, |context, link| {
		constructor_SimplifyLuauWide(context, link)
	})
}

#[expect(
	clippy::match_same_arms,
	clippy::too_many_lines,
	reason = "the exhaustive Node router stays in declaration order and owns one complete dispatch"
)]
fn simplify(nodes: &mut Vec<Node>, identifier: u32) -> bool {
	match &nodes[usize::try_from(identifier).unwrap()] {
		Node::Function(_)
		| Node::Match(_)
		| Node::Repeat(_)
		| Node::FunctionArguments(_)
		| Node::FunctionResults(_)
		| Node::BranchArguments(_)
		| Node::BranchResults(_)
		| Node::RepeatArguments(_)
		| Node::RepeatResults(_)
		| Node::Import(_)
		| Node::Export(_) => false,
		Node::Foreign(_) => simplify_foreign(nodes, identifier),
		Node::Trap
		| Node::Null
		| Node::I32(_)
		| Node::I64(_)
		| Node::F32(_)
		| Node::F64(_)
		| Node::Identity(_)
		| Node::Fence(_)
		| Node::Apply(_) => false,
		Node::RefIsNull(_) => simplify_single(nodes, identifier, |context, link| {
			constructor_SimplifyReference(context, link)
		}),
		Node::IntegerUnaryOperation(_)
		| Node::IntegerBinaryOperation(_)
		| Node::IntegerCompareOperation(_) => simplify_single(nodes, identifier, |context, link| {
			constructor_SimplifyInteger(context, link)
		}),
		Node::IntegerNarrow(_) => simplify_single(nodes, identifier, |context, link| {
			constructor_SimplifyConvert(context, link)
		}),
		Node::IntegerWiden(_) => false,
		Node::IntegerSignExtend(_) => simplify_single(nodes, identifier, |context, link| {
			constructor_SimplifyConvert(context, link)
		}),
		Node::IntegerConvertToNumber(_) => false,
		Node::IntegerTransmuteToNumber(_) => simplify_single(nodes, identifier, |context, link| {
			constructor_SimplifyConvert(context, link)
		}),
		Node::NumberUnaryOperation(_) => simplify_single(nodes, identifier, |context, link| {
			constructor_SimplifyFloat(context, link)
		}),
		Node::NumberBinaryOperation(_) => false,
		Node::NumberCompareOperation(_) => simplify_single(nodes, identifier, |context, link| {
			constructor_SimplifyInteger(context, link)
		}),
		Node::NumberNarrow(_) | Node::NumberWiden(_) | Node::NumberTruncateToInteger(_) => false,
		Node::NumberTransmuteToInteger(_) => simplify_single(nodes, identifier, |context, link| {
			constructor_SimplifyConvert(context, link)
		}),
		Node::MutableNew(_) => false,
		Node::MutableGet(_) => simplify_pair(nodes, identifier, |context, link| {
			constructor_SimplifyMutable(context, link)
		}),
		Node::MutableSet(_) | Node::Aggregate(_) => false,
		Node::Extract(_) => simplify_single(nodes, identifier, |context, link| {
			constructor_SimplifyAggregate(context, link)
		}),
		Node::TableNew(_) => false,
		Node::TableGet(_) => simplify_pair(nodes, identifier, |context, link| {
			constructor_SimplifyTable(context, link)
		}),
		Node::TableSet(_)
		| Node::TableSize(_)
		| Node::TableGrow(_)
		| Node::TableFill(_)
		| Node::TableCopy(_)
		| Node::TableDrop(_)
		| Node::MemoryNew(_) => false,
		Node::MemoryLoad(_) => simplify_pair(nodes, identifier, |context, link| {
			constructor_SimplifyMemory(context, link)
		}),
		Node::MemoryStore(_) | Node::MemoryFill(_) | Node::MemoryCopy(_) | Node::MemoryDrop(_) => {
			false
		}
	}
}

/// Sweeps every node once, reporting whether any rule fired.
#[must_use = "propagate whether this pass changed the graph"]
pub fn run(nodes: &mut Vec<Node>) -> bool {
	let mut applied = false;

	let Ok(last) = u32::try_from(nodes.len()) else {
		unreachable!()
	};

	for identifier in (0..last).rev() {
		applied |= simplify(nodes, identifier);
	}

	applied
}
