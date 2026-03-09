use alloc::{boxed::Box, sync::Arc, vec::Vec};
use list::resizable::Resizable;

use crate::{
	DataFlowGraph, Link, Node,
	node::{
		control::{
			Export, FunctionType, GammaIn, GammaOut, Import, LambdaIn, LambdaOut, OmegaIn,
			OmegaOut, RegionIn, RegionOut, ThetaIn, ThetaOut,
		},
		simple::{
			Apply, ExtendType, Fence, GlobalGet, GlobalNew, GlobalSet, Identity,
			IntegerBinaryOperation, IntegerBinaryOperator, IntegerCompareOperation,
			IntegerCompareOperator, IntegerConvertToNumber, IntegerExtend, IntegerNarrow,
			IntegerTransmuteToNumber, IntegerType, IntegerUnaryOperation, IntegerUnaryOperator,
			IntegerWiden, LoadType, Location, MemoryCopy, MemoryDrop, MemoryFill, MemoryGrow,
			MemoryLoad, MemoryNew, MemorySize, MemoryStore, NumberBinaryOperation,
			NumberBinaryOperator, NumberCompareOperation, NumberCompareOperator, NumberNarrow,
			NumberTransmuteToInteger, NumberTruncateToInteger, NumberType, NumberUnaryOperation,
			NumberUnaryOperator, NumberWiden, RefIsNull, StoreType, TableCopy, TableDrop,
			TableFill, TableGet, TableGrow, TableNew, TableSet, TableSize,
		},
	},
};

impl Identity {
	/// Adds an identity node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, sources: Resizable<Link, 4>) -> u32 {
		let node = Node::Identity(Self { sources });

		graph.add_node(node)
	}
}

impl Fence {
	/// Adds a fence node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, sources: Resizable<Link, 4>) -> u32 {
		let node = Node::Fence(Self { sources });

		graph.add_node(node)
	}
}

impl Apply {
	/// Adds a function application node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		function: Link,
		arguments: Vec<Link>,
		results: u16,
	) -> u32 {
		let node = Node::Apply(Self {
			function,
			arguments,
			results,
		});

		graph.add_node(node)
	}
}

impl RefIsNull {
	/// Adds a reference null check node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, source: Link) -> Link {
		let node = Node::RefIsNull(Self { source });

		Link(graph.add_node(node), 0)
	}
}

impl IntegerUnaryOperation {
	/// Adds an integer unary operation node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		source: Link,
		kind: IntegerType,
		operator: IntegerUnaryOperator,
	) -> Link {
		let node = Node::IntegerUnaryOperation(Self {
			source,
			kind,
			operator,
		});

		Link(graph.add_node(node), 0)
	}
}

impl IntegerBinaryOperation {
	/// Adds an integer binary operation node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		lhs: Link,
		rhs: Link,
		kind: IntegerType,
		operator: IntegerBinaryOperator,
	) -> Link {
		let node = Node::IntegerBinaryOperation(Self {
			lhs,
			rhs,
			kind,
			operator,
		});

		Link(graph.add_node(node), 0)
	}
}

impl IntegerCompareOperation {
	/// Adds an integer comparison node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		lhs: Link,
		rhs: Link,
		kind: IntegerType,
		operator: IntegerCompareOperator,
	) -> Link {
		let node = Node::IntegerCompareOperation(Self {
			lhs,
			rhs,
			kind,
			operator,
		});

		Link(graph.add_node(node), 0)
	}
}

impl IntegerNarrow {
	/// Adds an integer narrowing node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, source: Link) -> Link {
		let node = Node::IntegerNarrow(Self { source });

		Link(graph.add_node(node), 0)
	}
}

impl IntegerWiden {
	/// Adds an integer widening node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, source: Link) -> Link {
		let node = Node::IntegerWiden(Self { source });

		Link(graph.add_node(node), 0)
	}
}

impl IntegerExtend {
	/// Adds an integer sign-extension node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, source: Link, kind: ExtendType) -> Link {
		let node = Node::IntegerExtend(Self { source, kind });

		Link(graph.add_node(node), 0)
	}
}

impl IntegerConvertToNumber {
	/// Adds an integer-to-floating-point conversion node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		source: Link,
		signed: bool,
		to: NumberType,
		from: IntegerType,
	) -> Link {
		let node = Node::IntegerConvertToNumber(Self {
			source,
			signed,
			to,
			from,
		});

		Link(graph.add_node(node), 0)
	}
}

impl IntegerTransmuteToNumber {
	/// Adds an integer-to-floating-point reinterpretation node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, source: Link, from: IntegerType) -> Link {
		let node = Node::IntegerTransmuteToNumber(Self { source, from });

		Link(graph.add_node(node), 0)
	}
}

impl NumberUnaryOperation {
	/// Adds a floating-point unary operation node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		source: Link,
		kind: NumberType,
		operator: NumberUnaryOperator,
	) -> Link {
		let node = Node::NumberUnaryOperation(Self {
			source,
			kind,
			operator,
		});

		Link(graph.add_node(node), 0)
	}
}

impl NumberBinaryOperation {
	/// Adds a floating-point binary operation node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		lhs: Link,
		rhs: Link,
		kind: NumberType,
		operator: NumberBinaryOperator,
	) -> Link {
		let node = Node::NumberBinaryOperation(Self {
			lhs,
			rhs,
			kind,
			operator,
		});

		Link(graph.add_node(node), 0)
	}
}

impl NumberCompareOperation {
	/// Adds a floating-point comparison node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		lhs: Link,
		rhs: Link,
		kind: NumberType,
		operator: NumberCompareOperator,
	) -> Link {
		let node = Node::NumberCompareOperation(Self {
			lhs,
			rhs,
			kind,
			operator,
		});

		Link(graph.add_node(node), 0)
	}
}

impl NumberTruncateToInteger {
	/// Adds a floating-point-to-integer truncation node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		source: Link,
		signed: bool,
		saturate: bool,
		to: IntegerType,
		from: NumberType,
	) -> Link {
		let node = Node::NumberTruncateToInteger(Self {
			source,
			signed,
			saturate,
			to,
			from,
		});

		Link(graph.add_node(node), 0)
	}
}

impl NumberTransmuteToInteger {
	/// Adds a floating-point-to-integer reinterpretation node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, source: Link, from: NumberType) -> Link {
		let node = Node::NumberTransmuteToInteger(Self { source, from });

		Link(graph.add_node(node), 0)
	}
}

impl NumberNarrow {
	/// Adds a floating-point narrowing node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, source: Link) -> Link {
		let node = Node::NumberNarrow(Self { source });

		Link(graph.add_node(node), 0)
	}
}

impl NumberWiden {
	/// Adds a floating-point widening node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, source: Link) -> Link {
		let node = Node::NumberWiden(Self { source });

		Link(graph.add_node(node), 0)
	}
}

impl GlobalNew {
	/// Adds a global creation node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, initializer: Link) -> Link {
		let node = Node::GlobalNew(Self { initializer });

		Link(graph.add_node(node), 0)
	}
}

impl GlobalGet {
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a global read node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, source: Link) -> (Link, Link) {
		let node = Node::GlobalGet(Self { source });
		let id = graph.add_node(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl GlobalSet {
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a global write node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, destination: Link, source: Link) -> Link {
		let node = Node::GlobalSet(Self {
			destination,
			source,
		});

		Link(graph.add_node(node), Self::STATE_PORT)
	}
}

impl TableNew {
	/// Adds a table creation node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		initializer: Vec<(Link, u32)>,
		minimum: u32,
		maximum: u32,
	) -> Link {
		let node = Node::TableNew(Self {
			initializer,
			minimum,
			maximum,
		});

		Link(graph.add_node(node), 0)
	}
}

impl TableGet {
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a table read node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, source: Location) -> (Link, Link) {
		let node = Node::TableGet(Self { source });
		let id = graph.add_node(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl TableSet {
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a table write node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, destination: Location, source: Link) -> Link {
		let node = Node::TableSet(Self {
			destination,
			source,
		});

		Link(graph.add_node(node), Self::STATE_PORT)
	}
}

impl TableSize {
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a table size query node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, source: Link) -> (Link, Link) {
		let node = Node::TableSize(Self { source });
		let id = graph.add_node(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl TableGrow {
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a table grow node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		destination: Link,
		initializer: Link,
		size: Link,
	) -> (Link, Link) {
		let node = Node::TableGrow(Self {
			destination,
			initializer,
			size,
		});
		let id = graph.add_node(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl TableFill {
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a table fill node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		destination: Location,
		source: Link,
		size: Link,
	) -> Link {
		let node = Node::TableFill(Self {
			destination,
			source,
			size,
		});

		Link(graph.add_node(node), Self::STATE_PORT)
	}
}

impl TableCopy {
	/// The port index for the destination state token.
	pub const DESTINATION_STATE_PORT: u16 = 0;
	/// The port index for the source state token.
	pub const SOURCE_STATE_PORT: u16 = 1;

	/// Adds a table copy node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		destination: Location,
		source: Location,
		size: Link,
	) -> (Link, Link) {
		let node = Node::TableCopy(Self {
			destination,
			source,
			size,
		});
		let id = graph.add_node(node);

		(
			Link(id, Self::DESTINATION_STATE_PORT),
			Link(id, Self::SOURCE_STATE_PORT),
		)
	}
}

impl TableDrop {
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a table drop node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, source: Link) -> Link {
		let node = Node::TableDrop(Self { source });

		Link(graph.add_node(node), Self::STATE_PORT)
	}
}

impl MemoryNew {
	/// Adds a memory creation node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		initializer: Vec<(Arc<[u8]>, u32)>,
		minimum: u32,
		maximum: u32,
	) -> Link {
		let node = Node::MemoryNew(Self {
			initializer,
			minimum,
			maximum,
		});

		Link(graph.add_node(node), 0)
	}
}

impl MemoryLoad {
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a memory load node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, source: Location, kind: LoadType) -> (Link, Link) {
		let node = Node::MemoryLoad(Self { source, kind });
		let id = graph.add_node(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl MemoryStore {
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a memory store node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		destination: Location,
		source: Link,
		kind: StoreType,
	) -> Link {
		let node = Node::MemoryStore(Self {
			destination,
			source,
			kind,
		});

		Link(graph.add_node(node), Self::STATE_PORT)
	}
}

impl MemorySize {
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a memory size query node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, source: Link) -> (Link, Link) {
		let node = Node::MemorySize(Self { source });
		let id = graph.add_node(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl MemoryGrow {
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a memory grow node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, destination: Link, size: Link) -> (Link, Link) {
		let node = Node::MemoryGrow(Self { destination, size });
		let id = graph.add_node(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl MemoryFill {
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a memory fill node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		destination: Location,
		byte: Link,
		size: Link,
	) -> Link {
		let node = Node::MemoryFill(Self {
			destination,
			byte,
			size,
		});

		Link(graph.add_node(node), Self::STATE_PORT)
	}
}

impl MemoryCopy {
	/// The port index for the destination state token.
	pub const DESTINATION_STATE_PORT: u16 = 0;
	/// The port index for the source state token.
	pub const SOURCE_STATE_PORT: u16 = 1;

	/// Adds a memory copy node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		destination: Location,
		source: Location,
		size: Link,
	) -> (Link, Link) {
		let node = Node::MemoryCopy(Self {
			destination,
			source,
			size,
		});
		let id = graph.add_node(node);

		(
			Link(id, Self::DESTINATION_STATE_PORT),
			Link(id, Self::SOURCE_STATE_PORT),
		)
	}
}

impl MemoryDrop {
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a memory drop node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, source: Link) -> Link {
		let node = Node::MemoryDrop(Self { source });

		Link(graph.add_node(node), Self::STATE_PORT)
	}
}

impl LambdaIn {
	/// Returns the port range for the closure dependencies.
	///
	/// # Panics
	///
	/// Panics if the port count exceeds `u16::MAX`; if this happens, it is a bug.
	#[must_use]
	pub fn dependency_ports(&self) -> core::ops::Range<u16> {
		0..self.dependencies.len().try_into().unwrap()
	}

	/// Returns the port range for the function arguments.
	///
	/// # Panics
	///
	/// Panics if the port count exceeds `u16::MAX`; if this happens, it is a bug.
	#[must_use]
	pub fn argument_ports(&self) -> core::ops::Range<u16> {
		let dependencies: u16 = self.dependencies.len().try_into().unwrap();
		let arguments: u16 = self.kind.arguments.len().try_into().unwrap();

		dependencies..dependencies + arguments
	}

	/// Returns the port range for all output ports.
	///
	/// # Panics
	///
	/// Panics if the port count exceeds `u16::MAX`; if this happens, it is a bug.
	#[must_use]
	pub fn output_ports(&self) -> core::ops::Range<u16> {
		let dependencies: u16 = self.dependencies.len().try_into().unwrap();
		let arguments: u16 = self.kind.arguments.len().try_into().unwrap();

		0..dependencies + arguments
	}

	fn set_output_indirectly(graph: &mut DataFlowGraph, id: u32, to: u32) {
		let Self { output, .. } = graph.get_mut(id).as_mut_lambda_in().unwrap();

		*output = to;
	}

	/// Adds a lambda input node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		kind: Box<FunctionType>,
		dependencies: Vec<Link>,
	) -> u32 {
		let node = Node::LambdaIn(Self {
			output: u32::MAX,
			kind,
			dependencies,
		});

		graph.add_node(node)
	}
}

impl LambdaOut {
	/// Adds a lambda output node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, input: u32, results: Vec<Link>) -> u32 {
		let node = Node::LambdaOut(Self { input, results });
		let id = graph.add_node(node);

		LambdaIn::set_output_indirectly(graph, input, id);

		id
	}
}

impl RegionIn {
	fn ports_output(self, graph: &DataFlowGraph) -> usize {
		graph.get(self.input).as_gamma_in().unwrap().ports_output()
	}

	fn set_output_indirectly(graph: &mut DataFlowGraph, id: u32, to: u32) {
		let Self { output, .. } = graph.get_mut(id).as_mut_region_in().unwrap();

		*output = to;
	}

	/// Adds a region input node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, input: u32) -> u32 {
		let node = Node::RegionIn(Self {
			input,
			output: u32::MAX,
		});

		graph.add_node(node)
	}
}

impl RegionOut {
	const fn ports_output(&self) -> usize {
		self.results.len()
	}

	fn set_output_indirectly(graph: &mut DataFlowGraph, id: u32, to: u32) {
		let Self { output, .. } = graph.get_mut(id).as_mut_region_out().unwrap();

		*output = to;
	}

	/// Adds a region output node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, input: u32, results: Vec<Link>) -> u32 {
		let node = Node::RegionOut(Self {
			input,
			output: u32::MAX,
			results,
		});
		let id = graph.add_node(node);

		RegionIn::set_output_indirectly(graph, input, id);

		id
	}

	/// Adds a scoped region with input and output nodes to the graph.
	pub fn add_scoped_into<H>(graph: &mut DataFlowGraph, input: u32, handler: H) -> u32
	where
		H: FnOnce(&mut DataFlowGraph, u32) -> Vec<Link>,
	{
		let arguments = RegionIn::add_into(graph, input);
		let results = handler(graph, arguments);

		Self::add_into(graph, arguments, results)
	}
}

impl GammaIn {
	const fn ports_output(&self) -> usize {
		self.arguments.len()
	}

	fn set_output_indirectly(graph: &mut DataFlowGraph, id: u32, to: u32) {
		let Self { output, .. } = graph.get_mut(id).as_mut_gamma_in().unwrap();

		*output = to;
	}

	/// Adds a gamma input node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, arguments: Vec<Link>, condition: Link) -> u32 {
		let node = Node::GammaIn(Self {
			output: u32::MAX,
			arguments,
			condition,
		});

		graph.add_node(node)
	}
}

impl GammaOut {
	/// Returns the number of output ports.
	///
	/// # Panics
	///
	/// Panics if the graph structure is inconsistent; if this happens, it is a bug.
	#[must_use]
	pub fn ports_output(&self, graph: &DataFlowGraph) -> usize {
		let first = *self.regions.first().unwrap();

		graph.get(first).as_region_out().unwrap().ports_output()
	}

	/// Adds a gamma output node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, input: u32, regions: Vec<u32>) -> u32 {
		let id = graph.add_node(Node::Trap);

		GammaIn::set_output_indirectly(graph, input, id);

		for &region in &regions {
			RegionOut::set_output_indirectly(graph, region, id);
		}

		*graph.get_mut(id) = Node::GammaOut(Self { input, regions });

		id
	}

	/// Adds an if-else structure to the graph.
	pub fn add_if_into<F, T>(
		graph: &mut DataFlowGraph,
		arguments: Vec<Link>,
		condition: Link,
		on_false: F,
		on_true: T,
	) -> u32
	where
		F: FnOnce(&mut DataFlowGraph, u32) -> Vec<Link>,
		T: FnOnce(&mut DataFlowGraph, u32) -> Vec<Link>,
	{
		let arguments = GammaIn::add_into(graph, arguments, condition);
		let regions = alloc::vec![
			RegionOut::add_scoped_into(graph, arguments, on_false),
			RegionOut::add_scoped_into(graph, arguments, on_true)
		];

		Self::add_into(graph, arguments, regions)
	}
}

impl ThetaIn {
	const fn ports_output(&self) -> usize {
		self.arguments.len()
	}

	fn set_output_indirectly(graph: &mut DataFlowGraph, id: u32, to: u32) {
		let Self { output, .. } = graph.get_mut(id).as_mut_theta_in().unwrap();

		*output = to;
	}

	/// Adds a theta input node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph, arguments: Vec<Link>) -> u32 {
		let node = Node::ThetaIn(Self {
			output: u32::MAX,
			arguments,
		});

		graph.add_node(node)
	}
}

impl ThetaOut {
	const fn ports_output(&self) -> usize {
		self.results.len()
	}

	/// Adds a theta output node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		input: u32,
		results: Vec<Link>,
		condition: Link,
	) -> u32 {
		let node = Node::ThetaOut(Self {
			input,
			results,
			condition,
		});
		let id = graph.add_node(node);

		ThetaIn::set_output_indirectly(graph, input, id);

		id
	}
}

impl Import {
	/// Adds an import node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		environment: Link,
		namespace: Arc<str>,
		identifier: Arc<str>,
	) -> Link {
		let node = Node::Import(
			Self {
				environment,
				namespace,
				identifier,
			}
			.into(),
		);

		Link(graph.add_node(node), 0)
	}
}

impl OmegaIn {
	/// The port index for the environment.
	pub const ENVIRONMENT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	fn set_output_indirectly(graph: &mut DataFlowGraph, id: u32, to: u32) {
		let Self { output, .. } = graph.get_mut(id).as_mut_omega_in().unwrap();

		*output = to;
	}

	/// Adds an omega input node to the graph.
	pub fn add_into(graph: &mut DataFlowGraph) -> u32 {
		let node = Node::OmegaIn(Self { output: u32::MAX });

		graph.add_node(node)
	}
}

impl OmegaOut {
	/// Adds an omega output node to the graph.
	pub fn add_into(
		graph: &mut DataFlowGraph,
		input: u32,
		state: Link,
		exports: Vec<Export>,
	) -> u32 {
		let node = Node::OmegaOut(Self {
			input,
			state,
			exports,
		});
		let id = graph.add_node(node);

		OmegaIn::set_output_indirectly(graph, input, id);

		id
	}
}

impl Node {
	/// Returns the number of output ports, if applicable.
	#[must_use]
	pub fn ports_output(&self, graph: &DataFlowGraph) -> Option<usize> {
		let result = match self {
			Self::RegionIn(node) => (*node).ports_output(graph),
			Self::GammaOut(node) => node.ports_output(graph),
			Self::ThetaIn(node) => node.ports_output(),
			Self::ThetaOut(node) => node.ports_output(),

			Self::LambdaIn(_)
			| Self::LambdaOut(_)
			| Self::RegionOut(_)
			| Self::GammaIn(_)
			| Self::OmegaIn(_)
			| Self::OmegaOut(_)
			| Self::Import(_)
			| Self::Host(_)
			| Self::Trap
			| Self::Null
			| Self::I32(_)
			| Self::I64(_)
			| Self::F32(_)
			| Self::F64(_)
			| Self::Identity(_)
			| Self::Fence(_)
			| Self::Apply(_)
			| Self::RefIsNull(_)
			| Self::IntegerUnaryOperation(_)
			| Self::IntegerBinaryOperation(_)
			| Self::IntegerCompareOperation(_)
			| Self::IntegerNarrow(_)
			| Self::IntegerWiden(_)
			| Self::IntegerExtend(_)
			| Self::IntegerConvertToNumber(_)
			| Self::IntegerTransmuteToNumber(_)
			| Self::NumberUnaryOperation(_)
			| Self::NumberBinaryOperation(_)
			| Self::NumberCompareOperation(_)
			| Self::NumberNarrow(_)
			| Self::NumberWiden(_)
			| Self::NumberTruncateToInteger(_)
			| Self::NumberTransmuteToInteger(_)
			| Self::GlobalNew(_)
			| Self::GlobalGet(_)
			| Self::GlobalSet(_)
			| Self::TableNew(_)
			| Self::TableGet(_)
			| Self::TableSet(_)
			| Self::TableSize(_)
			| Self::TableGrow(_)
			| Self::TableFill(_)
			| Self::TableCopy(_)
			| Self::TableDrop(_)
			| Self::MemoryNew(_)
			| Self::MemoryLoad(_)
			| Self::MemoryStore(_)
			| Self::MemorySize(_)
			| Self::MemoryGrow(_)
			| Self::MemoryFill(_)
			| Self::MemoryCopy(_)
			| Self::MemoryDrop(_) => return None,
		};

		Some(result)
	}

	/// Returns a reference to this node's port list, if it has one.
	#[must_use]
	pub const fn as_ports(&self) -> Option<&Vec<Link>> {
		let ports = match self {
			Self::LambdaIn(node) => &node.dependencies,
			Self::LambdaOut(node) => &node.results,
			Self::RegionOut(node) => &node.results,
			Self::GammaIn(node) => &node.arguments,
			Self::ThetaIn(node) => &node.arguments,
			Self::ThetaOut(node) => &node.results,

			Self::RegionIn(_)
			| Self::GammaOut(_)
			| Self::OmegaIn(_)
			| Self::OmegaOut(_)
			| Self::Import(_)
			| Self::Host(_)
			| Self::Trap
			| Self::Null
			| Self::I32(_)
			| Self::I64(_)
			| Self::F32(_)
			| Self::F64(_)
			| Self::Identity(_)
			| Self::Fence(_)
			| Self::Apply(_)
			| Self::RefIsNull(_)
			| Self::IntegerUnaryOperation(_)
			| Self::IntegerBinaryOperation(_)
			| Self::IntegerCompareOperation(_)
			| Self::IntegerNarrow(_)
			| Self::IntegerWiden(_)
			| Self::IntegerExtend(_)
			| Self::IntegerConvertToNumber(_)
			| Self::IntegerTransmuteToNumber(_)
			| Self::NumberUnaryOperation(_)
			| Self::NumberBinaryOperation(_)
			| Self::NumberCompareOperation(_)
			| Self::NumberNarrow(_)
			| Self::NumberWiden(_)
			| Self::NumberTruncateToInteger(_)
			| Self::NumberTransmuteToInteger(_)
			| Self::GlobalNew(_)
			| Self::GlobalGet(_)
			| Self::GlobalSet(_)
			| Self::TableNew(_)
			| Self::TableGet(_)
			| Self::TableSet(_)
			| Self::TableSize(_)
			| Self::TableGrow(_)
			| Self::TableFill(_)
			| Self::TableCopy(_)
			| Self::TableDrop(_)
			| Self::MemoryNew(_)
			| Self::MemoryLoad(_)
			| Self::MemoryStore(_)
			| Self::MemorySize(_)
			| Self::MemoryGrow(_)
			| Self::MemoryFill(_)
			| Self::MemoryCopy(_)
			| Self::MemoryDrop(_) => return None,
		};

		Some(ports)
	}

	/// Returns mutable access to this node's port list, if it has one.
	#[must_use]
	pub const fn as_mut_ports(&mut self) -> Option<&mut Vec<Link>> {
		let ports = match self {
			Self::LambdaIn(node) => &mut node.dependencies,
			Self::LambdaOut(node) => &mut node.results,
			Self::RegionOut(node) => &mut node.results,
			Self::GammaIn(node) => &mut node.arguments,
			Self::ThetaIn(node) => &mut node.arguments,
			Self::ThetaOut(node) => &mut node.results,

			Self::RegionIn(_)
			| Self::GammaOut(_)
			| Self::OmegaIn(_)
			| Self::OmegaOut(_)
			| Self::Import(_)
			| Self::Host(_)
			| Self::Trap
			| Self::Null
			| Self::I32(_)
			| Self::I64(_)
			| Self::F32(_)
			| Self::F64(_)
			| Self::Identity(_)
			| Self::Fence(_)
			| Self::Apply(_)
			| Self::RefIsNull(_)
			| Self::IntegerUnaryOperation(_)
			| Self::IntegerBinaryOperation(_)
			| Self::IntegerCompareOperation(_)
			| Self::IntegerNarrow(_)
			| Self::IntegerWiden(_)
			| Self::IntegerExtend(_)
			| Self::IntegerConvertToNumber(_)
			| Self::IntegerTransmuteToNumber(_)
			| Self::NumberUnaryOperation(_)
			| Self::NumberBinaryOperation(_)
			| Self::NumberCompareOperation(_)
			| Self::NumberNarrow(_)
			| Self::NumberWiden(_)
			| Self::NumberTruncateToInteger(_)
			| Self::NumberTransmuteToInteger(_)
			| Self::GlobalNew(_)
			| Self::GlobalGet(_)
			| Self::GlobalSet(_)
			| Self::TableNew(_)
			| Self::TableGet(_)
			| Self::TableSet(_)
			| Self::TableSize(_)
			| Self::TableGrow(_)
			| Self::TableFill(_)
			| Self::TableCopy(_)
			| Self::TableDrop(_)
			| Self::MemoryNew(_)
			| Self::MemoryLoad(_)
			| Self::MemoryStore(_)
			| Self::MemorySize(_)
			| Self::MemoryGrow(_)
			| Self::MemoryFill(_)
			| Self::MemoryCopy(_)
			| Self::MemoryDrop(_) => return None,
		};

		Some(ports)
	}

	/// Adds a trap node to the graph.
	pub fn add_trap_into(graph: &mut DataFlowGraph) -> Link {
		Link(graph.add_node(Self::Trap), 0)
	}

	/// Adds a null reference constant node to the graph.
	pub fn add_null_into(graph: &mut DataFlowGraph) -> Link {
		Link(graph.add_node(Self::Null), 0)
	}

	/// Adds a 32-bit integer constant node to the graph.
	pub fn add_i32_into(graph: &mut DataFlowGraph, source: i32) -> Link {
		let node = Self::I32(source);

		Link(graph.add_node(node), 0)
	}

	/// Adds a 64-bit integer constant node to the graph.
	pub fn add_i64_into(graph: &mut DataFlowGraph, source: i64) -> Link {
		let node = Self::I64(source);

		Link(graph.add_node(node), 0)
	}

	/// Adds a 32-bit float constant node to the graph.
	pub fn add_f32_into(graph: &mut DataFlowGraph, source: f32) -> Link {
		let node = Self::F32(source);

		Link(graph.add_node(node), 0)
	}

	/// Adds a 64-bit float constant node to the graph.
	pub fn add_f64_into(graph: &mut DataFlowGraph, source: f64) -> Link {
		let node = Self::F64(source);

		Link(graph.add_node(node), 0)
	}
}
