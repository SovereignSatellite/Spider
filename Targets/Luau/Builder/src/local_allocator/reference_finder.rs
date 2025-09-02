use data_flow_graph::{
	DataFlowGraph, Link, Node,
	mvp::{
		Call, DataDrop, ElementsDrop, GlobalGet, GlobalSet, Identity, MemoryCopy, MemoryFill,
		MemoryGrow, MemoryInit, MemoryLoad, MemorySize, MemoryStore, Merge, TableCopy, TableFill,
		TableGet, TableGrow, TableInit, TableSet, TableSize,
	},
	nested::{
		GammaIn, GammaOut, LambdaIn, LambdaOut, OmegaIn, OmegaOut, RegionOut, ThetaIn, ThetaOut,
	},
};
use hashbrown::HashMap;

use super::scalar_finder::{add_value_assignments, result_count_of};

fn add_state_assignment(assignments: &mut HashMap<Link, Link>, graph: &DataFlowGraph, link: Link) {
	let Link(id, port) = link;

	if port >= result_count_of(graph.get(id)) {
		let _ = assignments.try_insert(link, link);
	}
}

fn handle_lambda_in(assignments: &mut HashMap<Link, Link>, id: u32, lambda_in: &LambdaIn) {
	for port in lambda_in.output_ports() {
		let argument = Link(id, port);

		let _ = assignments.try_insert(argument, argument);
	}
}

fn handle_lambda_out(
	assignments: &mut HashMap<Link, Link>,
	graph: &DataFlowGraph,
	lambda_out: &LambdaOut,
) {
	let LambdaOut { results, .. } = lambda_out;

	for &result in results {
		add_value_assignments(assignments, graph, result.0);
		add_state_assignment(assignments, graph, result);
	}
}

fn handle_region_out(assignments: &mut HashMap<Link, Link>, region_out: &RegionOut) {
	let RegionOut {
		output, results, ..
	} = region_out;

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
		let _ = assignments.try_insert(argument, argument);
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
		let _ = assignments.try_insert(result, result);
	}
}

fn handle_gamma_out(
	assignments: &mut HashMap<Link, Link>,
	graph: &DataFlowGraph,
	gamma_out: &GammaOut,
) {
	let GammaOut { input, regions } = gamma_out;
	let GammaIn {
		arguments,
		condition,
		..
	} = graph.get(*input).as_gamma_in().unwrap();

	handle_region_post(assignments, graph, regions, arguments);
	handle_gamma_post(assignments, graph, *regions.last().unwrap());

	if regions.len() != 2 {
		let _ = assignments.try_insert(*condition, *condition);
	}
}

fn handle_theta_out(
	assignments: &mut HashMap<Link, Link>,
	graph: &DataFlowGraph,
	theta_out: &ThetaOut,
) {
	let ThetaOut { input, results, .. } = theta_out;
	let ThetaIn { output, arguments } = graph.get(*input).as_theta_in().unwrap();

	let outputs = (0..).map(|port| Link(*output, port));

	assignments.extend(arguments.iter().copied().zip(outputs.clone()));
	assignments.extend(results.iter().copied().zip(outputs.clone()));

	let len = results.len();
	let inputs = (0..).map(|port| Link(*input, port));

	assignments.extend(inputs.zip(outputs.clone()).take(len));

	for output in outputs.take(len) {
		let _ = assignments.try_insert(output, output);
	}
}

fn handle_omega_in(
	assignments: &mut HashMap<Link, Link>,
	graph: &DataFlowGraph,
	omega_in: &OmegaIn,
) {
	let OmegaIn { output } = omega_in;
	let OmegaOut { input, state, .. } = graph.get(*output).as_omega_out().unwrap();

	let _ = assignments.try_insert(*state, *state);

	let environment = Link(*input, OmegaIn::ENVIRONMENT_PORT);
	let state = Link(*input, OmegaIn::STATE_PORT);

	let _ = assignments.try_insert(environment, environment);
	let _ = assignments.try_insert(state, state);
}

fn handle_trap(assignments: &mut HashMap<Link, Link>, id: u32) {
	let trap = Link(id, 0);

	let _ = assignments.try_insert(trap, trap);
}

fn handle_identity(assignments: &mut HashMap<Link, Link>, id: u32, identity: Identity) {
	let Identity { source } = identity;

	assignments.insert(source, Link(id, 0));
}

fn handle_call(assignments: &mut HashMap<Link, Link>, id: u32, call: &Call) {
	let Call {
		ref arguments,
		results,
		states,
		..
	} = *call;

	let arguments = arguments[arguments.len() - usize::from(states)..].iter();

	let len = arguments.len();
	let states = (results..).map(|port| Link(id, port));

	for state in states.clone().take(len) {
		let _ = assignments.try_insert(state, state);
	}

	assignments.extend(arguments.copied().zip(states));
}

fn handle_merge(assignments: &mut HashMap<Link, Link>, merge: &Merge) {
	let Merge { states } = merge;

	for &state in states {
		let _ = assignments.try_insert(state, state);
	}
}

fn handle_global_get(assignments: &mut HashMap<Link, Link>, id: u32, global_get: GlobalGet) {
	let GlobalGet { source } = global_get;

	// TODO: Fix `get` states
	if false {
		assignments.insert(source, Link(id, GlobalGet::STATE_PORT));
	}
}

fn handle_global_set(
	assignments: &mut HashMap<Link, Link>,
	graph: &DataFlowGraph,
	id: u32,
	global_set: GlobalSet,
) {
	let GlobalSet {
		destination,
		source,
	} = global_set;

	assignments.insert(destination, Link(id, GlobalSet::STATE_PORT));

	add_state_assignment(assignments, graph, source);
}

fn handle_table_get(assignments: &mut HashMap<Link, Link>, id: u32, table_get: TableGet) {
	let TableGet { source } = table_get;

	// TODO: Fix `get` states
	if false {
		assignments.insert(source.reference, Link(id, TableGet::STATE_PORT));
	}
}

fn handle_table_set(assignments: &mut HashMap<Link, Link>, id: u32, table_set: TableSet) {
	let TableSet { destination, .. } = table_set;

	assignments.insert(destination.reference, Link(id, TableSet::STATE_PORT));
}

fn handle_table_size(assignments: &mut HashMap<Link, Link>, id: u32, table_size: TableSize) {
	let TableSize { source } = table_size;

	// TODO: Fix `get` states
	if false {
		assignments.insert(source, Link(id, TableSize::STATE_PORT));
	}
}

fn handle_table_grow(assignments: &mut HashMap<Link, Link>, id: u32, table_grow: TableGrow) {
	let TableGrow { destination, .. } = table_grow;

	assignments.insert(destination, Link(id, TableGrow::STATE_PORT));
}

fn handle_table_fill(assignments: &mut HashMap<Link, Link>, id: u32, table_fill: TableFill) {
	let TableFill { destination, .. } = table_fill;

	assignments.insert(destination.reference, Link(id, TableFill::STATE_PORT));
}

fn handle_table_copy(assignments: &mut HashMap<Link, Link>, id: u32, table_copy: TableCopy) {
	let TableCopy {
		destination,
		source,
		..
	} = table_copy;

	assignments.insert(
		destination.reference,
		Link(id, TableCopy::DESTINATION_STATE_PORT),
	);
	assignments.insert(source.reference, Link(id, TableCopy::SOURCE_STATE_PORT));
}

fn handle_table_init(assignments: &mut HashMap<Link, Link>, id: u32, table_init: TableInit) {
	let TableInit {
		destination,
		source,
		..
	} = table_init;

	assignments.insert(
		destination.reference,
		Link(id, TableInit::DESTINATION_STATE_PORT),
	);
	assignments.insert(source.reference, Link(id, TableInit::SOURCE_STATE_PORT));
}

fn handle_elements_drop(
	assignments: &mut HashMap<Link, Link>,
	id: u32,
	elements_drop: ElementsDrop,
) {
	let ElementsDrop { source } = elements_drop;

	assignments.insert(source, Link(id, ElementsDrop::STATE_PORT));
}

fn handle_memory_load(assignments: &mut HashMap<Link, Link>, id: u32, memory_load: MemoryLoad) {
	let MemoryLoad { source, .. } = memory_load;

	// TODO: Fix `get` states
	if false {
		assignments.insert(source.reference, Link(id, MemoryLoad::STATE_PORT));
	}
}

fn handle_memory_store(assignments: &mut HashMap<Link, Link>, id: u32, memory_store: MemoryStore) {
	let MemoryStore { destination, .. } = memory_store;

	assignments.insert(destination.reference, Link(id, MemoryStore::STATE_PORT));
}

fn handle_memory_size(assignments: &mut HashMap<Link, Link>, id: u32, memory_size: MemorySize) {
	let MemorySize { source } = memory_size;

	// TODO: Fix `get` states
	if false {
		assignments.insert(source, Link(id, MemorySize::STATE_PORT));
	}
}

fn handle_memory_grow(assignments: &mut HashMap<Link, Link>, id: u32, memory_grow: MemoryGrow) {
	let MemoryGrow { destination, .. } = memory_grow;

	assignments.insert(destination, Link(id, MemoryGrow::STATE_PORT));
}

fn handle_memory_fill(assignments: &mut HashMap<Link, Link>, id: u32, memory_fill: MemoryFill) {
	let MemoryFill { destination, .. } = memory_fill;

	assignments.insert(destination.reference, Link(id, MemoryFill::STATE_PORT));
}

fn handle_memory_copy(assignments: &mut HashMap<Link, Link>, id: u32, memory_copy: MemoryCopy) {
	let MemoryCopy {
		destination,
		source,
		..
	} = memory_copy;

	assignments.insert(
		destination.reference,
		Link(id, MemoryCopy::DESTINATION_STATE_PORT),
	);
	assignments.insert(source.reference, Link(id, MemoryCopy::SOURCE_STATE_PORT));
}

fn handle_memory_init(assignments: &mut HashMap<Link, Link>, id: u32, memory_init: MemoryInit) {
	let MemoryInit {
		destination,
		source,
		..
	} = memory_init;

	assignments.insert(
		destination.reference,
		Link(id, MemoryInit::DESTINATION_STATE_PORT),
	);
	assignments.insert(source.reference, Link(id, MemoryInit::SOURCE_STATE_PORT));
}

fn handle_data_drop(assignments: &mut HashMap<Link, Link>, id: u32, data_drop: DataDrop) {
	let DataDrop { source } = data_drop;

	assignments.insert(source, Link(id, DataDrop::STATE_PORT));
}

fn handle_node(assignments: &mut HashMap<Link, Link>, graph: &DataFlowGraph, id: u32, node: &Node) {
	match *node {
		Node::RegionIn(_)
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
		| Node::ElementsNew(_)
		| Node::MemoryNew(_)
		| Node::DataNew(_) => {}

		Node::LambdaIn(ref lambda_in) => handle_lambda_in(assignments, id, lambda_in),
		Node::LambdaOut(ref lambda_out) => handle_lambda_out(assignments, graph, lambda_out),
		Node::RegionOut(ref region_out) => handle_region_out(assignments, region_out),
		Node::GammaOut(ref gamma_out) => handle_gamma_out(assignments, graph, gamma_out),
		Node::ThetaOut(ref theta_out) => handle_theta_out(assignments, graph, theta_out),
		Node::OmegaIn(ref omega_in) => handle_omega_in(assignments, graph, omega_in),

		Node::Trap => handle_trap(assignments, id),

		Node::Identity(identity) => handle_identity(assignments, id, identity),
		Node::Call(ref call) => handle_call(assignments, id, call),
		Node::Merge(ref merge) => handle_merge(assignments, merge),
		Node::GlobalGet(global_get) => handle_global_get(assignments, id, global_get),
		Node::GlobalSet(global_set) => handle_global_set(assignments, graph, id, global_set),
		Node::TableGet(table_get) => handle_table_get(assignments, id, table_get),
		Node::TableSet(table_set) => handle_table_set(assignments, id, table_set),
		Node::TableSize(table_size) => handle_table_size(assignments, id, table_size),
		Node::TableGrow(table_grow) => handle_table_grow(assignments, id, table_grow),
		Node::TableFill(table_fill) => handle_table_fill(assignments, id, table_fill),
		Node::TableCopy(table_copy) => handle_table_copy(assignments, id, table_copy),
		Node::TableInit(table_init) => handle_table_init(assignments, id, table_init),
		Node::ElementsDrop(elements_drop) => {
			handle_elements_drop(assignments, id, elements_drop);
		}
		Node::MemoryLoad(memory_load) => handle_memory_load(assignments, id, memory_load),
		Node::MemoryStore(memory_store) => handle_memory_store(assignments, id, memory_store),
		Node::MemorySize(memory_size) => handle_memory_size(assignments, id, memory_size),
		Node::MemoryGrow(memory_grow) => handle_memory_grow(assignments, id, memory_grow),
		Node::MemoryFill(memory_fill) => handle_memory_fill(assignments, id, memory_fill),
		Node::MemoryCopy(memory_copy) => handle_memory_copy(assignments, id, memory_copy),
		Node::MemoryInit(memory_init) => handle_memory_init(assignments, id, memory_init),
		Node::DataDrop(data_drop) => handle_data_drop(assignments, id, data_drop),
	}
}

pub fn run(assignments: &mut HashMap<Link, Link>, graph: &DataFlowGraph) {
	for (node, id) in graph.nodes().zip(0..) {
		handle_node(assignments, graph, id, node);
	}
}
