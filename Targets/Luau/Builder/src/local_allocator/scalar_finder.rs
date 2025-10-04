use data_flow_graph::{DataFlowGraph, Link, Node};
use hashbrown::{HashMap, hash_map::Entry};

pub fn result_count_of(node: &Node) -> u16 {
	match node {
		Node::LambdaIn(_)
		| Node::RegionIn(_)
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

		Node::Host(_node) => 0,

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

		Node::Call(node) => node.results,
	}
}

fn add_value_assignments(assignments: &mut HashMap<Link, Link>, graph: &DataFlowGraph, id: u32) {
	let results = result_count_of(graph.get(id));

	for link in (0..results).map(|port| Link(id, port)) {
		let _ = assignments.try_insert(link, Link::DANGLING);
	}
}

pub struct ScalarFinder {
	handled: HashMap<u32, bool>,
}

impl ScalarFinder {
	pub fn new() -> Self {
		Self {
			handled: HashMap::new(),
		}
	}

	fn handle_effects(
		&mut self,
		assignments: &mut HashMap<Link, Link>,
		graph: &DataFlowGraph,
		id: u32,
		node: &Node,
	) {
		// Without at least one value reference we might discard the side effects
		// of these expressions.
		if !matches!(
			node,
			Node::Call(_) | Node::TableGrow(_) | Node::MemoryGrow(_)
		) {
			return;
		}

		if let Entry::Vacant(entry) = self.handled.entry(id) {
			entry.insert(true);

			add_value_assignments(assignments, graph, id);
		}
	}

	fn handle_repeat(
		&mut self,
		assignments: &mut HashMap<Link, Link>,
		graph: &DataFlowGraph,
		id: u32,
	) {
		match self.handled.entry(id) {
			Entry::Occupied(mut entry) => {
				if entry.insert(true) {
					return;
				}

				add_value_assignments(assignments, graph, id);
			}
			Entry::Vacant(entry) => {
				entry.insert(false);
			}
		}
	}

	fn handle_excess(
		&mut self,
		assignments: &mut HashMap<Link, Link>,
		graph: &DataFlowGraph,
		id: u32,
	) {
		if self.handled.insert(id, true).unwrap_or_default() {
			return;
		}

		add_value_assignments(assignments, graph, id);
	}

	fn handle_uses(
		&mut self,
		assignments: &mut HashMap<Link, Link>,
		graph: &DataFlowGraph,
		node: &Node,
	) {
		node.for_each_argument(|Link(id, port)| {
			if port == 0 {
				self.handle_repeat(assignments, graph, id);
			} else {
				self.handle_excess(assignments, graph, id);
			}
		});
	}

	// We assign locals to all value ports in a node if...
	//   * Any value port is used out of local order.
	//   * Any value port has more than one use.
	//   * Any value port other than the first is in use.
	//   * No value port is used but it has side effects.
	pub fn run(&mut self, assignments: &mut HashMap<Link, Link>, graph: &DataFlowGraph) {
		self.handled.clear();

		// All uses are handled first since that contains all base assignments.
		for node in graph.nodes() {
			self.handle_uses(assignments, graph, node);
		}

		// Then, effects are handled from the missing assignments.
		for (node, id) in graph.nodes().zip(0..) {
			self.handle_effects(assignments, graph, id, node);
		}
	}
}
