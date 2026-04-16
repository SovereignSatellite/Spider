use alloc::sync::Arc;

use hashbrown::HashMap;
use parking_lot::Mutex;

use ir_graph::{
	Link, Node,
	control::{Match, ModuleArguments, Repeat},
	simple::{
		Fence, GlobalGet, GlobalSet, Identity, MemoryCopy, MemoryDrop, MemoryFill, MemoryGrow,
		MemoryLoad, MemorySize, MemoryStore, TableCopy, TableDrop, TableFill, TableGet, TableGrow,
		TableSet, TableSize,
	},
};

use super::scalar_finder::value_port_count_of;

type ScopedLink = (Link, usize);

fn add_state_assignment(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	nodes: &[Node],
	scope: usize,
	link: Link,
) {
	let Link(id, port) = link;

	if port >= value_port_count_of(&nodes[id as usize]) {
		let _ = assignments.try_insert((link, scope), (Link::DANGLING, 0));
	}
}

fn handle_repeat(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	parent_scope: usize,
	repeat_node_id: u32,
	arc: &Arc<Mutex<Repeat>>,
) {
	let repeat = arc.lock();
	let repeat_scope = Arc::as_ptr(arc) as usize;
	let argument_count = repeat.argument_count();

	for port in 0..argument_count {
		let _ = assignments.try_insert((Link(0, port), repeat_scope), (Link::DANGLING, 0));
	}

	for (offset, &argument) in repeat.arguments.iter().enumerate() {
		let port = u16::try_from(offset).unwrap();

		assignments.insert((argument, parent_scope), (Link(0, port), repeat_scope));
	}

	for (offset, &result) in repeat.results().sources.iter().enumerate() {
		let port = u16::try_from(offset).unwrap();

		assignments.insert((result, repeat_scope), (Link(0, port), repeat_scope));
	}

	drop(repeat);

	for port in 0..argument_count {
		assignments.insert(
			(Link(repeat_node_id, port), parent_scope),
			(Link(0, port), repeat_scope),
		);
	}
}

fn handle_match(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	parent_scope: usize,
	match_node_id: u32,
	arc: &Arc<Mutex<Match>>,
) {
	let matcher = arc.lock();

	let Some(last_branch_arc) = matcher.branches.last() else {
		return;
	};

	let last_branch = last_branch_arc.lock();
	let last_branch_scope = Arc::as_ptr(last_branch_arc) as usize;
	let argument_count = matcher.argument_count();

	for port in 0..argument_count {
		let _ = assignments.try_insert((Link(0, port), last_branch_scope), (Link::DANGLING, 0));
	}

	for branch_arc in &matcher.branches[..matcher.branches.len() - 1] {
		let branch_scope = Arc::as_ptr(branch_arc) as usize;

		for port in 0..argument_count {
			assignments.insert(
				(Link(0, port), branch_scope),
				(Link(0, port), last_branch_scope),
			);
		}
	}

	for (offset, &argument) in matcher.arguments.iter().enumerate() {
		let port = u16::try_from(offset).unwrap();

		assignments.insert((argument, parent_scope), (Link(0, port), last_branch_scope));
	}

	// All branches write to the last branch's canonical result locals, so
	// the Match output reads from those shared locals.
	for (offset, &result) in last_branch.results().sources.iter().enumerate() {
		let port = u16::try_from(offset).unwrap();

		assignments.insert(
			(Link(match_node_id, port), parent_scope),
			(result, last_branch_scope),
		);
	}

	for &result in &last_branch.results().sources {
		let _ = assignments.try_insert((result, last_branch_scope), (Link::DANGLING, 0));
	}

	for branch_arc in &matcher.branches[..matcher.branches.len() - 1] {
		let branch = branch_arc.lock();
		let branch_scope = Arc::as_ptr(branch_arc) as usize;

		for (&result, &last_result) in branch
			.results()
			.sources
			.iter()
			.zip(&last_branch.results().sources)
		{
			assignments.insert((result, branch_scope), (last_result, last_branch_scope));
		}
	}

	// A two-branch match is an if/else whose condition is used directly; any
	// other shape needs a local to dispatch through.
	if matcher.branches.len() != 2 {
		let _ = assignments.try_insert((matcher.condition, parent_scope), (Link::DANGLING, 0));
	}
}

fn handle_identity(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	scope: usize,
	id: u32,
	node: &Identity,
) {
	let Identity { sources } = node;

	let len = sources.len();
	let outputs = (0..).map(|port| Link(id, port));

	assignments.extend(
		sources
			.iter()
			.copied()
			.zip(outputs.clone())
			.map(|(src, dst)| ((src, scope), (dst, scope))),
	);

	for output in outputs.take(len) {
		let _ = assignments.try_insert((output, scope), (Link::DANGLING, 0));
	}
}

fn handle_fence(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	scope: usize,
	id: u32,
	node: &Fence,
) {
	let Fence { sources } = node;

	let len = sources.len();
	let outputs = (0..).map(|port| Link(id, port));

	assignments.extend(
		sources
			.iter()
			.copied()
			.zip(outputs.clone())
			.map(|(src, dst)| ((src, scope), (dst, scope))),
	);

	for output in outputs.take(len) {
		let _ = assignments.try_insert((output, scope), (Link::DANGLING, 0));
	}
}

fn handle_global_get(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	scope: usize,
	id: u32,
	node: GlobalGet,
) {
	let GlobalGet { source } = node;

	let _ = assignments.insert((source, scope), (Link(id, GlobalGet::STATE_PORT), scope));
}

fn handle_global_set(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	nodes: &[Node],
	scope: usize,
	id: u32,
	node: GlobalSet,
) {
	let GlobalSet {
		destination,
		source,
	} = node;

	assignments.insert(
		(destination, scope),
		(Link(id, GlobalSet::STATE_PORT), scope),
	);

	add_state_assignment(assignments, nodes, scope, source);
}

fn handle_table_get(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	scope: usize,
	id: u32,
	node: TableGet,
) {
	let TableGet { source } = node;

	assignments.insert(
		(source.reference, scope),
		(Link(id, TableGet::STATE_PORT), scope),
	);
}

fn handle_table_set(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	scope: usize,
	id: u32,
	node: TableSet,
) {
	let TableSet { destination, .. } = node;

	assignments.insert(
		(destination.reference, scope),
		(Link(id, TableSet::STATE_PORT), scope),
	);
}

fn handle_table_size(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	scope: usize,
	id: u32,
	node: TableSize,
) {
	let TableSize { source } = node;

	assignments.insert((source, scope), (Link(id, TableSize::STATE_PORT), scope));
}

fn handle_table_grow(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	scope: usize,
	id: u32,
	node: TableGrow,
) {
	let TableGrow { destination, .. } = node;

	assignments.insert(
		(destination, scope),
		(Link(id, TableGrow::STATE_PORT), scope),
	);
}

fn handle_table_fill(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	scope: usize,
	id: u32,
	node: TableFill,
) {
	let TableFill { destination, .. } = node;

	assignments.insert(
		(destination.reference, scope),
		(Link(id, TableFill::STATE_PORT), scope),
	);
}

fn handle_table_copy(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	scope: usize,
	id: u32,
	node: TableCopy,
) {
	let TableCopy {
		destination,
		source,
		..
	} = node;

	assignments.insert(
		(destination.reference, scope),
		(Link(id, TableCopy::DESTINATION_STATE_PORT), scope),
	);
	assignments.insert(
		(source.reference, scope),
		(Link(id, TableCopy::SOURCE_STATE_PORT), scope),
	);
}

fn handle_table_drop(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	scope: usize,
	id: u32,
	node: TableDrop,
) {
	let TableDrop { source } = node;

	assignments.insert((source, scope), (Link(id, TableDrop::STATE_PORT), scope));
}

fn handle_memory_load(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	scope: usize,
	id: u32,
	node: MemoryLoad,
) {
	let MemoryLoad { source, .. } = node;

	assignments.insert(
		(source.reference, scope),
		(Link(id, MemoryLoad::STATE_PORT), scope),
	);
}

fn handle_memory_store(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	scope: usize,
	id: u32,
	node: MemoryStore,
) {
	let MemoryStore { destination, .. } = node;

	assignments.insert(
		(destination.reference, scope),
		(Link(id, MemoryStore::STATE_PORT), scope),
	);
}

fn handle_memory_size(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	scope: usize,
	id: u32,
	node: MemorySize,
) {
	let MemorySize { source } = node;

	assignments.insert((source, scope), (Link(id, MemorySize::STATE_PORT), scope));
}

fn handle_memory_grow(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	scope: usize,
	id: u32,
	node: MemoryGrow,
) {
	let MemoryGrow { destination, .. } = node;

	assignments.insert(
		(destination, scope),
		(Link(id, MemoryGrow::STATE_PORT), scope),
	);
}

fn handle_memory_fill(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	scope: usize,
	id: u32,
	node: MemoryFill,
) {
	let MemoryFill { destination, .. } = node;

	assignments.insert(
		(destination.reference, scope),
		(Link(id, MemoryFill::STATE_PORT), scope),
	);
}

fn handle_memory_copy(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	scope: usize,
	id: u32,
	node: MemoryCopy,
) {
	let MemoryCopy {
		destination,
		source,
		..
	} = node;

	assignments.insert(
		(destination.reference, scope),
		(Link(id, MemoryCopy::DESTINATION_STATE_PORT), scope),
	);
	assignments.insert(
		(source.reference, scope),
		(Link(id, MemoryCopy::SOURCE_STATE_PORT), scope),
	);
}

fn handle_memory_drop(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	scope: usize,
	id: u32,
	node: MemoryDrop,
) {
	let MemoryDrop { source } = node;

	assignments.insert((source, scope), (Link(id, MemoryDrop::STATE_PORT), scope));
}

#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
fn handle_node(
	assignments: &mut HashMap<ScopedLink, ScopedLink>,
	nodes: &[Node],
	scope: usize,
	id: u32,
	node: &Node,
) {
	match *node {
		Node::Function(_)
		| Node::ModuleResults(_)
		| Node::FunctionCaptures(_)
		| Node::FunctionArguments(_)
		| Node::FunctionResults(_)
		| Node::BranchArguments(_)
		| Node::BranchResults(_)
		| Node::RepeatArguments(_)
		| Node::RepeatResults(_)
		| Node::Import(_)
		| Node::Host(_)
		| Node::Null
		| Node::I32(_)
		| Node::I64(_)
		| Node::F32(_)
		| Node::F64(_)
		| Node::Apply(_)
		| Node::RefIsNull(_)
		| Node::IntegerUnaryOperation(_)
		| Node::IntegerBinaryOperation(_)
		| Node::IntegerCompareOperation(_)
		| Node::IntegerNarrow(_)
		| Node::IntegerWiden(_)
		| Node::IntegerExtend(_)
		| Node::IntegerConvertToNumber(_)
		| Node::IntegerTransmuteToNumber(_)
		| Node::NumberUnaryOperation(_)
		| Node::NumberBinaryOperation(_)
		| Node::NumberCompareOperation(_)
		| Node::NumberNarrow(_)
		| Node::NumberWiden(_)
		| Node::NumberTruncateToInteger(_)
		| Node::NumberTransmuteToInteger(_)
		| Node::GlobalNew(_)
		| Node::TableNew(_)
		| Node::MemoryNew(_) => {}

		Node::Match(ref arc) => handle_match(assignments, scope, id, arc),
		Node::Repeat(ref arc) => handle_repeat(assignments, scope, id, arc),

		Node::ModuleArguments(_) => {
			for port in 0..ModuleArguments::RESULT_COUNT {
				let _ = assignments.try_insert((Link(id, port), scope), (Link::DANGLING, 0));
			}
		}

		Node::Trap => {
			let _ = assignments.try_insert((Link(id, 0), scope), (Link::DANGLING, 0));
		}

		Node::Identity(ref node) => handle_identity(assignments, scope, id, node),
		Node::Fence(ref node) => handle_fence(assignments, scope, id, node),
		Node::GlobalGet(node) => handle_global_get(assignments, scope, id, node),
		Node::GlobalSet(node) => handle_global_set(assignments, nodes, scope, id, node),
		Node::TableGet(node) => handle_table_get(assignments, scope, id, node),
		Node::TableSet(node) => handle_table_set(assignments, scope, id, node),
		Node::TableSize(node) => handle_table_size(assignments, scope, id, node),
		Node::TableGrow(node) => handle_table_grow(assignments, scope, id, node),
		Node::TableFill(node) => handle_table_fill(assignments, scope, id, node),
		Node::TableCopy(node) => handle_table_copy(assignments, scope, id, node),
		Node::TableDrop(node) => handle_table_drop(assignments, scope, id, node),
		Node::MemoryLoad(node) => handle_memory_load(assignments, scope, id, node),
		Node::MemoryStore(node) => handle_memory_store(assignments, scope, id, node),
		Node::MemorySize(node) => handle_memory_size(assignments, scope, id, node),
		Node::MemoryGrow(node) => handle_memory_grow(assignments, scope, id, node),
		Node::MemoryFill(node) => handle_memory_fill(assignments, scope, id, node),
		Node::MemoryCopy(node) => handle_memory_copy(assignments, scope, id, node),
		Node::MemoryDrop(node) => handle_memory_drop(assignments, scope, id, node),
	}
}

#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
fn run_region(assignments: &mut HashMap<ScopedLink, ScopedLink>, nodes: &[Node], scope: usize) {
	for (id, node) in nodes.iter().enumerate() {
		let id = id.try_into().unwrap();

		handle_node(assignments, nodes, scope, id, node);
	}

	// Recurse into nested control regions (not Functions).
	for node in nodes {
		match node {
			Node::Function(_)
			| Node::ModuleArguments(_)
			| Node::ModuleResults(_)
			| Node::FunctionCaptures(_)
			| Node::FunctionArguments(_)
			| Node::FunctionResults(_)
			| Node::BranchArguments(_)
			| Node::BranchResults(_)
			| Node::RepeatArguments(_)
			| Node::RepeatResults(_)
			| Node::Import(_)
			| Node::Host(_)
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
			| Node::IntegerExtend(_)
			| Node::IntegerConvertToNumber(_)
			| Node::IntegerTransmuteToNumber(_)
			| Node::NumberUnaryOperation(_)
			| Node::NumberBinaryOperation(_)
			| Node::NumberCompareOperation(_)
			| Node::NumberNarrow(_)
			| Node::NumberWiden(_)
			| Node::NumberTruncateToInteger(_)
			| Node::NumberTransmuteToInteger(_)
			| Node::GlobalNew(_)
			| Node::GlobalGet(_)
			| Node::GlobalSet(_)
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

			Node::Match(arc) => {
				let matcher = arc.lock();

				for branch_arc in &matcher.branches {
					let branch = branch_arc.lock();
					let branch_scope = Arc::as_ptr(branch_arc) as usize;

					run_region(assignments, &branch.nodes, branch_scope);
				}
			}
			Node::Repeat(arc) => {
				let repeat = arc.lock();
				let repeat_scope = Arc::as_ptr(arc) as usize;

				run_region(assignments, &repeat.nodes, repeat_scope);
			}
		}
	}
}

pub fn run(assignments: &mut HashMap<ScopedLink, ScopedLink>, nodes: &[Node], scope: usize) {
	run_region(assignments, nodes, scope);
}
