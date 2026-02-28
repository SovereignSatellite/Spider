use hashbrown::HashMap;
use ir_graph::{
	DataFlowGraph, Link, Node,
	control::{GammaIn, GammaOut, LambdaIn, OmegaIn, OmegaOut, RegionOut, ThetaIn, ThetaOut},
	simple::{
		Fence, GlobalGet, GlobalSet, Identity, MemoryCopy, MemoryDrop, MemoryFill, MemoryGrow,
		MemoryLoad, MemorySize, MemoryStore, TableCopy, TableDrop, TableFill, TableGet, TableGrow,
		TableSet, TableSize,
	},
};

use super::scalar_finder::result_count_of;

fn add_state_assignment(assignments: &mut HashMap<Link, Link>, graph: &DataFlowGraph, link: Link) {
	let Link(id, port) = link;

	if port >= result_count_of(graph.get(id)) {
		let _ = assignments.try_insert(link, Link::DANGLING);
	}
}

fn handle_lambda_in(assignments: &mut HashMap<Link, Link>, id: u32, node: &LambdaIn) {
	for port in node.output_ports() {
		let _ = assignments.try_insert(Link(id, port), Link::DANGLING);
	}
}

fn handle_region_out(assignments: &mut HashMap<Link, Link>, node: &RegionOut) {
	let RegionOut {
		output, results, ..
	} = node;

	let outputs = (0..).map(|port| Link(*output, port));

	assignments.extend(results.iter().copied().zip(outputs));
}

fn handle_region_post(
	assignments: &mut HashMap<Link, Link>,
	graph: &DataFlowGraph,
	regions: &[u32],
	arguments: &[Link],
) {
	let mut regions = regions.iter();
	let last = *regions.next_back().unwrap();

	let RegionOut { input: last, .. } = *graph.get(last).as_region_out().unwrap();

	let len = arguments.len().try_into().unwrap();

	// We ensure that all arguments of the last region get their local, even if not used.
	for argument in (0..len).map(|port| Link(last, port)) {
		let _ = assignments.try_insert(argument, Link::DANGLING);
	}

	// Then we make all other regions reuse the locals of the last.
	for &region in regions {
		let RegionOut { input, .. } = *graph.get(region).as_region_out().unwrap();

		assignments.extend((0..len).map(|port| (Link(input, port), Link(last, port))));
	}

	// Then we make all arguments of the block reuse the locals of the last.
	let inputs = (0..).map(|port| Link(last, port));

	assignments.extend(arguments.iter().copied().zip(inputs));
}

fn handle_gamma_post(assignments: &mut HashMap<Link, Link>, graph: &DataFlowGraph, region: u32) {
	let RegionOut {
		output, results, ..
	} = graph.get(region).as_region_out().unwrap();

	let len = results.len().try_into().unwrap();

	// We ensure that all results get their local, even if not used.
	for result in (0..len).map(|port| Link(*output, port)) {
		let _ = assignments.try_insert(result, Link::DANGLING);
	}
}

fn handle_gamma_out(assignments: &mut HashMap<Link, Link>, graph: &DataFlowGraph, node: &GammaOut) {
	let GammaOut { input, regions } = node;
	let GammaIn {
		arguments,
		condition,
		..
	} = graph.get(*input).as_gamma_in().unwrap();

	handle_region_post(assignments, graph, regions, arguments);
	handle_gamma_post(assignments, graph, *regions.last().unwrap());

	if regions.len() != 2 {
		let _ = assignments.try_insert(*condition, Link::DANGLING);
	}
}

fn handle_theta_out(assignments: &mut HashMap<Link, Link>, graph: &DataFlowGraph, node: &ThetaOut) {
	let ThetaOut { input, results, .. } = node;
	let ThetaIn { output, arguments } = graph.get(*input).as_theta_in().unwrap();

	let outputs = (0..).map(|port| Link(*output, port));

	assignments.extend(arguments.iter().copied().zip(outputs.clone()));
	assignments.extend(results.iter().copied().zip(outputs.clone()));

	let len = results.len();
	let inputs = (0..).map(|port| Link(*input, port));

	assignments.extend(inputs.zip(outputs.clone()).take(len));

	for output in outputs.take(len) {
		let _ = assignments.try_insert(output, Link::DANGLING);
	}
}

fn handle_omega_in(assignments: &mut HashMap<Link, Link>, graph: &DataFlowGraph, node: &OmegaIn) {
	let OmegaIn { output } = node;
	let OmegaOut { input, state, .. } = graph.get(*output).as_omega_out().unwrap();

	let _ = assignments.try_insert(Link(*input, OmegaIn::ENVIRONMENT_PORT), Link::DANGLING);
	let _ = assignments.try_insert(Link(*input, OmegaIn::STATE_PORT), Link::DANGLING);

	let _ = assignments.try_insert(*state, Link::DANGLING);
}

fn handle_trap(assignments: &mut HashMap<Link, Link>, id: u32) {
	let _ = assignments.try_insert(Link(id, 0), Link::DANGLING);
}

fn handle_identity(assignments: &mut HashMap<Link, Link>, id: u32, node: &Identity) {
	let Identity { sources } = node;

	let len = sources.len();
	let outputs = (0..).map(|port| Link(id, port));

	assignments.extend(sources.iter().copied().zip(outputs.clone()));

	for output in outputs.take(len) {
		let _ = assignments.try_insert(output, Link::DANGLING);
	}
}

fn handle_fence(assignments: &mut HashMap<Link, Link>, id: u32, node: &Fence) {
	let Fence { sources } = node;

	let len = sources.len();
	let outputs = (0..).map(|port| Link(id, port));

	assignments.extend(sources.iter().copied().zip(outputs.clone()));

	for output in outputs.take(len) {
		let _ = assignments.try_insert(output, Link::DANGLING);
	}
}

fn handle_global_get(assignments: &mut HashMap<Link, Link>, id: u32, node: GlobalGet) {
	let GlobalGet { source } = node;

	assignments.insert(source, Link(id, GlobalGet::STATE_PORT));
}

fn handle_global_set(
	assignments: &mut HashMap<Link, Link>,
	graph: &DataFlowGraph,
	id: u32,
	node: GlobalSet,
) {
	let GlobalSet {
		destination,
		source,
	} = node;

	assignments.insert(destination, Link(id, GlobalSet::STATE_PORT));

	add_state_assignment(assignments, graph, source);
}

fn handle_table_get(assignments: &mut HashMap<Link, Link>, id: u32, node: TableGet) {
	let TableGet { source } = node;

	assignments.insert(source.reference, Link(id, TableGet::STATE_PORT));
}

fn handle_table_set(assignments: &mut HashMap<Link, Link>, id: u32, node: TableSet) {
	let TableSet { destination, .. } = node;

	assignments.insert(destination.reference, Link(id, TableSet::STATE_PORT));
}

fn handle_table_size(assignments: &mut HashMap<Link, Link>, id: u32, node: TableSize) {
	let TableSize { source } = node;

	assignments.insert(source, Link(id, TableSize::STATE_PORT));
}

fn handle_table_grow(assignments: &mut HashMap<Link, Link>, id: u32, node: TableGrow) {
	let TableGrow { destination, .. } = node;

	assignments.insert(destination, Link(id, TableGrow::STATE_PORT));
}

fn handle_table_fill(assignments: &mut HashMap<Link, Link>, id: u32, node: TableFill) {
	let TableFill { destination, .. } = node;

	assignments.insert(destination.reference, Link(id, TableFill::STATE_PORT));
}

fn handle_table_copy(assignments: &mut HashMap<Link, Link>, id: u32, node: TableCopy) {
	let TableCopy {
		destination,
		source,
		..
	} = node;

	assignments.insert(
		destination.reference,
		Link(id, TableCopy::DESTINATION_STATE_PORT),
	);
	assignments.insert(source.reference, Link(id, TableCopy::SOURCE_STATE_PORT));
}

fn handle_table_drop(assignments: &mut HashMap<Link, Link>, id: u32, node: TableDrop) {
	let TableDrop { source } = node;

	assignments.insert(source, Link(id, TableDrop::STATE_PORT));
}

fn handle_memory_load(assignments: &mut HashMap<Link, Link>, id: u32, node: MemoryLoad) {
	let MemoryLoad { source, .. } = node;

	assignments.insert(source.reference, Link(id, MemoryLoad::STATE_PORT));
}

fn handle_memory_store(assignments: &mut HashMap<Link, Link>, id: u32, node: MemoryStore) {
	let MemoryStore { destination, .. } = node;

	assignments.insert(destination.reference, Link(id, MemoryStore::STATE_PORT));
}

fn handle_memory_size(assignments: &mut HashMap<Link, Link>, id: u32, node: MemorySize) {
	let MemorySize { source } = node;

	assignments.insert(source, Link(id, MemorySize::STATE_PORT));
}

fn handle_memory_grow(assignments: &mut HashMap<Link, Link>, id: u32, node: MemoryGrow) {
	let MemoryGrow { destination, .. } = node;

	assignments.insert(destination, Link(id, MemoryGrow::STATE_PORT));
}

fn handle_memory_fill(assignments: &mut HashMap<Link, Link>, id: u32, node: MemoryFill) {
	let MemoryFill { destination, .. } = node;

	assignments.insert(destination.reference, Link(id, MemoryFill::STATE_PORT));
}

fn handle_memory_copy(assignments: &mut HashMap<Link, Link>, id: u32, node: MemoryCopy) {
	let MemoryCopy {
		destination,
		source,
		..
	} = node;

	assignments.insert(
		destination.reference,
		Link(id, MemoryCopy::DESTINATION_STATE_PORT),
	);
	assignments.insert(source.reference, Link(id, MemoryCopy::SOURCE_STATE_PORT));
}

fn handle_memory_drop(assignments: &mut HashMap<Link, Link>, id: u32, node: MemoryDrop) {
	let MemoryDrop { source } = node;

	assignments.insert(source, Link(id, MemoryDrop::STATE_PORT));
}

fn handle_node(assignments: &mut HashMap<Link, Link>, graph: &DataFlowGraph, id: u32, node: &Node) {
	match *node {
		Node::LambdaOut(_)
		| Node::RegionIn(_)
		| Node::GammaIn(_)
		| Node::ThetaIn(_)
		| Node::OmegaOut(_)
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

		Node::LambdaIn(ref node) => handle_lambda_in(assignments, id, node),
		Node::RegionOut(ref node) => handle_region_out(assignments, node),
		Node::GammaOut(ref node) => handle_gamma_out(assignments, graph, node),
		Node::ThetaOut(ref node) => handle_theta_out(assignments, graph, node),
		Node::OmegaIn(ref node) => handle_omega_in(assignments, graph, node),

		Node::Trap => handle_trap(assignments, id),

		Node::Identity(ref node) => handle_identity(assignments, id, node),
		Node::Fence(ref node) => handle_fence(assignments, id, node),
		Node::GlobalGet(node) => handle_global_get(assignments, id, node),
		Node::GlobalSet(node) => handle_global_set(assignments, graph, id, node),
		Node::TableGet(node) => handle_table_get(assignments, id, node),
		Node::TableSet(node) => handle_table_set(assignments, id, node),
		Node::TableSize(node) => handle_table_size(assignments, id, node),
		Node::TableGrow(node) => handle_table_grow(assignments, id, node),
		Node::TableFill(node) => handle_table_fill(assignments, id, node),
		Node::TableCopy(node) => handle_table_copy(assignments, id, node),
		Node::TableDrop(node) => handle_table_drop(assignments, id, node),
		Node::MemoryLoad(node) => handle_memory_load(assignments, id, node),
		Node::MemoryStore(node) => handle_memory_store(assignments, id, node),
		Node::MemorySize(node) => handle_memory_size(assignments, id, node),
		Node::MemoryGrow(node) => handle_memory_grow(assignments, id, node),
		Node::MemoryFill(node) => handle_memory_fill(assignments, id, node),
		Node::MemoryCopy(node) => handle_memory_copy(assignments, id, node),
		Node::MemoryDrop(node) => handle_memory_drop(assignments, id, node),
	}
}

pub fn run(assignments: &mut HashMap<Link, Link>, graph: &DataFlowGraph) {
	for (node, id) in graph.nodes().zip(0..) {
		handle_node(assignments, graph, id, node);
	}
}
