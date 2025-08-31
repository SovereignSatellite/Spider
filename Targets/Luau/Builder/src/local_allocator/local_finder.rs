use alloc::vec::Vec;
use data_flow_graph::{
	mvp::{
		Call, DataDrop, ElementsDrop, GlobalGet, GlobalSet, MemoryCopy, MemoryFill, MemoryGrow,
		MemoryInit, MemoryLoad, MemorySize, MemoryStore, Merge, TableCopy, TableFill, TableGet,
		TableGrow, TableInit, TableSet, TableSize,
	},
	nested::{GammaIn, GammaOut, LambdaOut, RegionOut, ThetaIn, ThetaOut},
	DataFlowGraph, Link, Node,
};

use crate::reference_finder::{result_count_of, ReferenceFinder};

// We localize any predecessors that appear out of order relative
// to their position in the topological order.
fn localize_sequences(locals: &mut Vec<u32>, graph: &DataFlowGraph, node: &Node) {
	let mut parameter = 0;

	node.for_each_argument(|Link(id, port)| {
		if port >= result_count_of(graph.get(id)) {
			return;
		}

		if id >= parameter {
			parameter = id;
		} else {
			locals.push(id);
		}
	});
}

fn handle_link_list(locals: &mut Vec<u32>, list: &[Link]) {
	locals.extend(list.iter().map(|link| link.0));
}

fn handle_region_out(locals: &mut Vec<u32>, region_out: &RegionOut) {
	let RegionOut { results, .. } = region_out;

	handle_link_list(locals, results);
}

fn handle_gamma_in(locals: &mut Vec<u32>, gamma_in: &GammaIn) {
	let GammaIn { arguments, .. } = gamma_in;

	handle_link_list(locals, arguments);
}

fn handle_gamma_out(locals: &mut Vec<u32>, graph: &DataFlowGraph, gamma_out: &GammaOut) {
	let GammaOut { input, regions } = gamma_out;

	if regions.len() == 2 {
		return;
	}

	let GammaIn { condition, .. } = graph.get(*input).as_gamma_in().unwrap();

	locals.push(condition.0);
}

fn handle_theta_in(locals: &mut Vec<u32>, theta_in: &ThetaIn) {
	let ThetaIn { arguments, .. } = theta_in;

	handle_link_list(locals, arguments);
}

fn handle_theta_out(locals: &mut Vec<u32>, theta_out: &ThetaOut) {
	let ThetaOut {
		condition, results, ..
	} = theta_out;

	locals.push(condition.0);

	handle_link_list(locals, results);
}

fn handle_lambda_out(locals: &mut Vec<u32>, lambda_out: &LambdaOut) {
	let LambdaOut { input: _, results } = lambda_out;

	handle_link_list(locals, results);
}

fn localize_regions(locals: &mut Vec<u32>, graph: &DataFlowGraph, node: &Node) {
	match node {
		Node::RegionOut(region_out) => handle_region_out(locals, region_out),
		Node::GammaIn(gamma_in) => handle_gamma_in(locals, gamma_in),
		Node::GammaOut(gamma_out) => handle_gamma_out(locals, graph, gamma_out),
		Node::ThetaIn(theta_in) => handle_theta_in(locals, theta_in),
		Node::ThetaOut(theta_out) => handle_theta_out(locals, theta_out),
		Node::LambdaOut(lambda_out) => handle_lambda_out(locals, lambda_out),

		_ => {}
	}
}

// If a state input takes from a value output then we should localize the value.
fn handle_state_producer(locals: &mut Vec<u32>, graph: &DataFlowGraph, link: Link) {
	let node = graph.get(link.0);

	if result_count_of(node) > link.1 {
		locals.push(link.0);
	}
}

fn localize_state_producer(locals: &mut Vec<u32>, graph: &DataFlowGraph, node: &Node) {
	match *node {
		Node::LambdaIn(_)
		| Node::LambdaOut(_)
		| Node::RegionIn(_)
		| Node::RegionOut(_)
		| Node::GammaIn(_)
		| Node::GammaOut(_)
		| Node::ThetaIn(_)
		| Node::ThetaOut(_)
		| Node::OmegaIn(_)
		| Node::OmegaOut(_)
		| Node::Import(_)
		| Node::Host(_)
		| Node::Trap
		| Node::Null
		| Node::Identity(_)
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

		Node::Call(Call {
			ref arguments,
			states,
			..
		}) => {
			for &argument in &arguments[arguments.len() - usize::from(states)..] {
				handle_state_producer(locals, graph, argument);
			}
		}
		Node::Merge(Merge { ref states }) => {
			for &state in states {
				handle_state_producer(locals, graph, state);
			}
		}
		Node::GlobalGet(GlobalGet { source })
		| Node::TableSize(TableSize { source })
		| Node::ElementsDrop(ElementsDrop { source })
		| Node::MemorySize(MemorySize { source })
		| Node::DataDrop(DataDrop { source }) => handle_state_producer(locals, graph, source),

		Node::GlobalSet(GlobalSet { destination, .. })
		| Node::TableGrow(TableGrow { destination, .. })
		| Node::MemoryGrow(MemoryGrow { destination, .. }) => {
			handle_state_producer(locals, graph, destination);
		}

		Node::TableGet(TableGet { source }) | Node::MemoryLoad(MemoryLoad { source, .. }) => {
			handle_state_producer(locals, graph, source.reference);
		}

		Node::TableSet(TableSet { destination, .. })
		| Node::TableFill(TableFill { destination, .. })
		| Node::MemoryStore(MemoryStore { destination, .. })
		| Node::MemoryFill(MemoryFill { destination, .. }) => {
			handle_state_producer(locals, graph, destination.reference);
		}

		Node::TableCopy(TableCopy {
			destination,
			source,
			..
		})
		| Node::TableInit(TableInit {
			destination,
			source,
			..
		})
		| Node::MemoryCopy(MemoryCopy {
			destination,
			source,
			..
		})
		| Node::MemoryInit(MemoryInit {
			destination,
			source,
			..
		}) => {
			handle_state_producer(locals, graph, destination.reference);
			handle_state_producer(locals, graph, source.reference);
		}
	}
}

fn should_localize(reference_finder: &ReferenceFinder, id: u32, node: &Node) -> bool {
	let local = match *node {
		Node::Trap => true,

		Node::Call(Call { results, .. }) => results
			.checked_sub(1)
			.is_some_and(|port| !reference_finder.has_result_at(id, port)),

		Node::TableGrow(_) => !reference_finder.has_result_at(id, TableGrow::RESULT_PORT),
		Node::MemoryGrow(_) => !reference_finder.has_result_at(id, MemoryGrow::RESULT_PORT),

		_ => false,
	};

	local || reference_finder.has_many_uses(id, node) || reference_finder.has_many_results(id, node)
}

pub fn run(locals: &mut Vec<u32>, graph: &DataFlowGraph, reference_finder: &ReferenceFinder) {
	locals.clear();

	for (node, id) in graph.nodes().zip(0..) {
		localize_sequences(locals, graph, node);
		localize_regions(locals, graph, node);
		localize_state_producer(locals, graph, node);

		if should_localize(reference_finder, id, node) {
			locals.push(id);
		}
	}

	locals.sort_unstable();
	locals.dedup();
}
