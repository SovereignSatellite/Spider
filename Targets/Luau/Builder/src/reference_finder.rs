use data_flow_graph::{DataFlowGraph, Node};
use data_flow_visitor::successor_finder::SuccessorFinder;

pub fn result_count_of(node: &Node) -> u16 {
	match node {
		Node::LambdaIn(lambda_in) => lambda_in.r#type.arguments.len().try_into().unwrap(),

		Node::RegionIn(_)
		| Node::RegionOut(_)
		| Node::GammaIn(_)
		| Node::GammaOut(_)
		| Node::ThetaIn(_)
		| Node::ThetaOut(_)
		| Node::OmegaIn(_)
		| Node::Merge(_)
		| Node::GlobalSet(_)
		| Node::TableSet(_)
		| Node::TableFill(_)
		| Node::TableCopy(_)
		| Node::TableInit(_)
		| Node::ElementsDrop(_)
		| Node::MemoryStore(_)
		| Node::MemoryFill(_)
		| Node::MemoryCopy(_)
		| Node::MemoryInit(_)
		| Node::DataDrop(_) => 0,

		Node::Host(_host) => 0,

		Node::LambdaOut(_)
		| Node::OmegaOut(_)
		| Node::Import(_)
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
		| Node::GlobalGet(_)
		| Node::TableNew(_)
		| Node::TableGet(_)
		| Node::TableSize(_)
		| Node::TableGrow(_)
		| Node::ElementsNew(_)
		| Node::MemoryNew(_)
		| Node::MemoryLoad(_)
		| Node::MemorySize(_)
		| Node::MemoryGrow(_)
		| Node::DataNew(_) => 1,

		Node::Call(call) => call.results,
	}
}

pub struct ReferenceFinder {
	successor_finder: SuccessorFinder,
}

impl ReferenceFinder {
	pub const fn new() -> Self {
		Self {
			successor_finder: SuccessorFinder::new(),
		}
	}

	pub fn has_result_at(&self, id: u32, port: u16) -> bool {
		self.successor_finder.at(id).any(|&item| item.port <= port)
	}

	pub fn has_many_uses(&self, id: u32, node: &Node) -> bool {
		let mut successors = self
			.successor_finder
			.at(id)
			.filter(|item| item.port < result_count_of(node));

		successors.next().is_some() && successors.next().is_some()
	}

	pub fn has_many_results(&self, id: u32, node: &Node) -> bool {
		self.successor_finder
			.at(id)
			.any(|item| item.port > 0 && item.port < result_count_of(node))
	}

	pub fn run(&mut self, graph: &DataFlowGraph) {
		self.successor_finder.run(graph);
	}
}
