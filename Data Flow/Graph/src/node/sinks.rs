use alloc::{boxed::Box, sync::Arc, vec::Vec};
use control_flow_graph::instruction::{
	ExtendType, IntegerBinaryOperator, IntegerCompareOperator, IntegerType, IntegerUnaryOperator,
	LoadType, NumberBinaryOperator, NumberCompareOperator, NumberType, NumberUnaryOperator,
	StoreType,
};
use list::resizable::Resizable;

use crate::{
	DataFlowGraph, Link, Node,
	node::{
		control::{
			Export, FunctionType, GammaIn, GammaOut, Import, LambdaIn, LambdaOut, OmegaIn,
			OmegaOut, RegionIn, RegionOut, ThetaIn, ThetaOut,
		},
		simple::{
			Apply, GlobalGet, GlobalNew, GlobalSet, Identity, IntegerBinaryOperation,
			IntegerCompareOperation, IntegerConvertToNumber, IntegerExtend, IntegerNarrow,
			IntegerTransmuteToNumber, IntegerUnaryOperation, IntegerWiden, Location, MemoryCopy,
			MemoryDrop, MemoryFill, MemoryGrow, MemoryLoad, MemoryNew, MemorySize, MemoryStore,
			Merge, NumberBinaryOperation, NumberCompareOperation, NumberNarrow,
			NumberTransmuteToInteger, NumberTruncateToInteger, NumberUnaryOperation, NumberWiden,
			RefIsNull, TableCopy, TableDrop, TableFill, TableGet, TableGrow, TableNew, TableSet,
			TableSize,
		},
	},
};

impl Identity {
	pub fn add_into(graph: &mut DataFlowGraph, sources: Resizable<Link, 4>) -> Link {
		let node = Node::Identity(Self { sources });

		Link(graph.add_node(node), 0)
	}
}

impl Merge {
	pub fn add_into(graph: &mut DataFlowGraph, sources: Resizable<Link, 4>) -> Link {
		let node = Node::Merge(Self { sources });

		Link(graph.add_node(node), 0)
	}
}

impl Apply {
	pub fn add_into(
		graph: &mut DataFlowGraph,
		function: Link,
		arguments: Vec<Link>,
		results: u16,
		states: u16,
	) -> u32 {
		let node = Node::Apply(Self {
			function,
			arguments,
			results,
			states,
		});

		graph.add_node(node)
	}
}

impl RefIsNull {
	pub fn add_into(graph: &mut DataFlowGraph, source: Link) -> Link {
		let node = Node::RefIsNull(Self { source });

		Link(graph.add_node(node), 0)
	}
}

impl IntegerUnaryOperation {
	pub fn add_into(
		graph: &mut DataFlowGraph,
		source: Link,
		r#type: IntegerType,
		operator: IntegerUnaryOperator,
	) -> Link {
		let node = Node::IntegerUnaryOperation(Self {
			source,
			r#type,
			operator,
		});

		Link(graph.add_node(node), 0)
	}
}

impl IntegerBinaryOperation {
	pub fn add_into(
		graph: &mut DataFlowGraph,
		lhs: Link,
		rhs: Link,
		r#type: IntegerType,
		operator: IntegerBinaryOperator,
	) -> Link {
		let node = Node::IntegerBinaryOperation(Self {
			lhs,
			rhs,
			r#type,
			operator,
		});

		Link(graph.add_node(node), 0)
	}
}

impl IntegerCompareOperation {
	pub fn add_into(
		graph: &mut DataFlowGraph,
		lhs: Link,
		rhs: Link,
		r#type: IntegerType,
		operator: IntegerCompareOperator,
	) -> Link {
		let node = Node::IntegerCompareOperation(Self {
			lhs,
			rhs,
			r#type,
			operator,
		});

		Link(graph.add_node(node), 0)
	}
}

impl IntegerNarrow {
	pub fn add_into(graph: &mut DataFlowGraph, source: Link) -> Link {
		let node = Node::IntegerNarrow(Self { source });

		Link(graph.add_node(node), 0)
	}
}

impl IntegerWiden {
	pub fn add_into(graph: &mut DataFlowGraph, source: Link) -> Link {
		let node = Node::IntegerWiden(Self { source });

		Link(graph.add_node(node), 0)
	}
}

impl IntegerExtend {
	pub fn add_into(graph: &mut DataFlowGraph, source: Link, r#type: ExtendType) -> Link {
		let node = Node::IntegerExtend(Self { source, r#type });

		Link(graph.add_node(node), 0)
	}
}

impl IntegerConvertToNumber {
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
	pub fn add_into(graph: &mut DataFlowGraph, source: Link, from: IntegerType) -> Link {
		let node = Node::IntegerTransmuteToNumber(Self { source, from });

		Link(graph.add_node(node), 0)
	}
}

impl NumberUnaryOperation {
	pub fn add_into(
		graph: &mut DataFlowGraph,
		source: Link,
		r#type: NumberType,
		operator: NumberUnaryOperator,
	) -> Link {
		let node = Node::NumberUnaryOperation(Self {
			source,
			r#type,
			operator,
		});

		Link(graph.add_node(node), 0)
	}
}

impl NumberBinaryOperation {
	pub fn add_into(
		graph: &mut DataFlowGraph,
		lhs: Link,
		rhs: Link,
		r#type: NumberType,
		operator: NumberBinaryOperator,
	) -> Link {
		let node = Node::NumberBinaryOperation(Self {
			lhs,
			rhs,
			r#type,
			operator,
		});

		Link(graph.add_node(node), 0)
	}
}

impl NumberCompareOperation {
	pub fn add_into(
		graph: &mut DataFlowGraph,
		lhs: Link,
		rhs: Link,
		r#type: NumberType,
		operator: NumberCompareOperator,
	) -> Link {
		let node = Node::NumberCompareOperation(Self {
			lhs,
			rhs,
			r#type,
			operator,
		});

		Link(graph.add_node(node), 0)
	}
}

impl NumberTruncateToInteger {
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
	pub fn add_into(graph: &mut DataFlowGraph, source: Link, from: NumberType) -> Link {
		let node = Node::NumberTransmuteToInteger(Self { source, from });

		Link(graph.add_node(node), 0)
	}
}

impl NumberNarrow {
	pub fn add_into(graph: &mut DataFlowGraph, source: Link) -> Link {
		let node = Node::NumberNarrow(Self { source });

		Link(graph.add_node(node), 0)
	}
}

impl NumberWiden {
	pub fn add_into(graph: &mut DataFlowGraph, source: Link) -> Link {
		let node = Node::NumberWiden(Self { source });

		Link(graph.add_node(node), 0)
	}
}

impl GlobalNew {
	pub fn add_into(graph: &mut DataFlowGraph, initializer: Link) -> Link {
		let node = Node::GlobalNew(Self { initializer });

		Link(graph.add_node(node), 0)
	}
}

impl GlobalGet {
	pub const RESULT_PORT: u16 = 0;
	pub const STATE_PORT: u16 = 1;

	pub fn add_into(graph: &mut DataFlowGraph, source: Link) -> (Link, Link) {
		let node = Node::GlobalGet(Self { source });
		let id = graph.add_node(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl GlobalSet {
	pub const STATE_PORT: u16 = 0;

	pub fn add_into(graph: &mut DataFlowGraph, destination: Link, source: Link) -> Link {
		let node = Node::GlobalSet(Self {
			destination,
			source,
		});

		Link(graph.add_node(node), Self::STATE_PORT)
	}
}

impl TableNew {
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
	pub const RESULT_PORT: u16 = 0;
	pub const STATE_PORT: u16 = 1;

	pub fn add_into(graph: &mut DataFlowGraph, source: Location) -> (Link, Link) {
		let node = Node::TableGet(Self { source });
		let id = graph.add_node(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl TableSet {
	pub const STATE_PORT: u16 = 0;

	pub fn add_into(graph: &mut DataFlowGraph, destination: Location, source: Link) -> Link {
		let node = Node::TableSet(Self {
			destination,
			source,
		});

		Link(graph.add_node(node), Self::STATE_PORT)
	}
}

impl TableSize {
	pub const RESULT_PORT: u16 = 0;
	pub const STATE_PORT: u16 = 1;

	pub fn add_into(graph: &mut DataFlowGraph, source: Link) -> (Link, Link) {
		let node = Node::TableSize(Self { source });
		let id = graph.add_node(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl TableGrow {
	pub const RESULT_PORT: u16 = 0;
	pub const STATE_PORT: u16 = 1;

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
	pub const STATE_PORT: u16 = 0;

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
	pub const DESTINATION_STATE_PORT: u16 = 0;
	pub const SOURCE_STATE_PORT: u16 = 1;

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
	pub const STATE_PORT: u16 = 0;

	pub fn add_into(graph: &mut DataFlowGraph, source: Link) -> Link {
		let node = Node::TableDrop(Self { source });

		Link(graph.add_node(node), Self::STATE_PORT)
	}
}

impl MemoryNew {
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
	pub const RESULT_PORT: u16 = 0;
	pub const STATE_PORT: u16 = 1;

	pub fn add_into(graph: &mut DataFlowGraph, source: Location, r#type: LoadType) -> (Link, Link) {
		let node = Node::MemoryLoad(Self { source, r#type });
		let id = graph.add_node(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl MemoryStore {
	pub const STATE_PORT: u16 = 0;

	pub fn add_into(
		graph: &mut DataFlowGraph,
		destination: Location,
		source: Link,
		r#type: StoreType,
	) -> Link {
		let node = Node::MemoryStore(Self {
			destination,
			source,
			r#type,
		});

		Link(graph.add_node(node), Self::STATE_PORT)
	}
}

impl MemorySize {
	pub const RESULT_PORT: u16 = 0;
	pub const STATE_PORT: u16 = 1;

	pub fn add_into(graph: &mut DataFlowGraph, source: Link) -> (Link, Link) {
		let node = Node::MemorySize(Self { source });
		let id = graph.add_node(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl MemoryGrow {
	pub const RESULT_PORT: u16 = 0;
	pub const STATE_PORT: u16 = 1;

	pub fn add_into(graph: &mut DataFlowGraph, destination: Link, size: Link) -> (Link, Link) {
		let node = Node::MemoryGrow(Self { destination, size });
		let id = graph.add_node(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl MemoryFill {
	pub const STATE_PORT: u16 = 0;

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
	pub const DESTINATION_STATE_PORT: u16 = 0;
	pub const SOURCE_STATE_PORT: u16 = 1;

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
	pub const STATE_PORT: u16 = 0;

	pub fn add_into(graph: &mut DataFlowGraph, source: Link) -> Link {
		let node = Node::MemoryDrop(Self { source });

		Link(graph.add_node(node), Self::STATE_PORT)
	}
}

impl LambdaIn {
	#[must_use]
	pub fn dependency_ports(&self) -> core::ops::Range<u16> {
		0..self.dependencies.len().try_into().unwrap()
	}

	#[must_use]
	pub fn argument_ports(&self) -> core::ops::Range<u16> {
		let dependencies: u16 = self.dependencies.len().try_into().unwrap();
		let arguments: u16 = self.r#type.arguments.len().try_into().unwrap();

		dependencies..dependencies + arguments
	}

	#[must_use]
	pub fn output_ports(&self) -> core::ops::Range<u16> {
		let dependencies: u16 = self.dependencies.len().try_into().unwrap();
		let arguments: u16 = self.r#type.arguments.len().try_into().unwrap();

		0..dependencies + arguments
	}

	fn set_output_indirectly(graph: &mut DataFlowGraph, id: u32, to: u32) {
		let Self { output, .. } = graph.get_mut(id).as_mut_lambda_in().unwrap();

		*output = to;
	}

	pub fn add_into(
		graph: &mut DataFlowGraph,
		r#type: Box<FunctionType>,
		dependencies: Vec<Link>,
	) -> u32 {
		let node = Node::LambdaIn(Self {
			output: u32::MAX,
			r#type,
			dependencies,
		});

		graph.add_node(node)
	}
}

impl LambdaOut {
	pub fn add_into(graph: &mut DataFlowGraph, input: u32, results: Vec<Link>) -> u32 {
		let node = Node::LambdaOut(Self { input, results });
		let id = graph.add_node(node);

		LambdaIn::set_output_indirectly(graph, input, id);

		id
	}
}

impl RegionIn {
	fn ports_output(&self, graph: &DataFlowGraph) -> usize {
		graph.get(self.input).as_gamma_in().unwrap().ports_output()
	}

	fn set_output_indirectly(graph: &mut DataFlowGraph, id: u32, to: u32) {
		let Self { output, .. } = graph.get_mut(id).as_mut_region_in().unwrap();

		*output = to;
	}

	pub fn add_into(graph: &mut DataFlowGraph, input: u32) -> u32 {
		let node = Node::RegionIn(Self {
			output: u32::MAX,
			input,
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

	pub fn add_into(graph: &mut DataFlowGraph, input: u32, results: Vec<Link>) -> u32 {
		let node = Node::RegionOut(Self {
			output: u32::MAX,
			input,
			results,
		});
		let id = graph.add_node(node);

		RegionIn::set_output_indirectly(graph, input, id);

		id
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
	#[must_use]
	pub fn ports_output(&self, graph: &DataFlowGraph) -> usize {
		let first = *self.regions.first().unwrap();

		graph.get(first).as_region_out().unwrap().ports_output()
	}

	pub fn add_into(graph: &mut DataFlowGraph, input: u32, regions: Vec<u32>) -> u32 {
		let id = graph.add_node(Node::Trap);

		GammaIn::set_output_indirectly(graph, input, id);

		for &region in &regions {
			RegionOut::set_output_indirectly(graph, region, id);
		}

		*graph.get_mut(id) = Node::GammaOut(Self { input, regions });

		id
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
	pub const ENVIRONMENT_PORT: u16 = 0;
	pub const STATE_PORT: u16 = 1;

	fn set_output_indirectly(graph: &mut DataFlowGraph, id: u32, to: u32) {
		let Self { output, .. } = graph.get_mut(id).as_mut_omega_in().unwrap();

		*output = to;
	}

	pub fn add_into(graph: &mut DataFlowGraph) -> u32 {
		let node = Node::OmegaIn(Self { output: u32::MAX });

		graph.add_node(node)
	}
}

impl OmegaOut {
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
	#[must_use]
	pub fn ports_output(&self, graph: &DataFlowGraph) -> Option<usize> {
		let result = match self {
			Self::RegionIn(node) => node.ports_output(graph),
			Self::GammaOut(node) => node.ports_output(graph),
			Self::ThetaIn(node) => node.ports_output(),
			Self::ThetaOut(node) => node.ports_output(),

			_ => return None,
		};

		Some(result)
	}

	#[must_use]
	pub const fn as_ports(&self) -> Option<&Vec<Link>> {
		let ports = match self {
			Self::LambdaIn(node) => &node.dependencies,
			Self::LambdaOut(node) => &node.results,
			Self::RegionOut(node) => &node.results,
			Self::GammaIn(node) => &node.arguments,
			Self::ThetaIn(node) => &node.arguments,
			Self::ThetaOut(node) => &node.results,

			_ => return None,
		};

		Some(ports)
	}

	#[must_use]
	pub const fn as_mut_ports(&mut self) -> Option<&mut Vec<Link>> {
		let ports = match self {
			Self::LambdaIn(node) => &mut node.dependencies,
			Self::LambdaOut(node) => &mut node.results,
			Self::RegionOut(node) => &mut node.results,
			Self::GammaIn(node) => &mut node.arguments,
			Self::ThetaIn(node) => &mut node.arguments,
			Self::ThetaOut(node) => &mut node.results,

			_ => return None,
		};

		Some(ports)
	}

	pub fn add_trap_into(graph: &mut DataFlowGraph) -> Link {
		Link(graph.add_node(Self::Trap), 0)
	}

	pub fn add_null_into(graph: &mut DataFlowGraph) -> Link {
		Link(graph.add_node(Self::Null), 0)
	}

	pub fn add_i32_into(graph: &mut DataFlowGraph, source: i32) -> Link {
		let node = Self::I32(source);

		Link(graph.add_node(node), 0)
	}

	pub fn add_i64_into(graph: &mut DataFlowGraph, source: i64) -> Link {
		let node = Self::I64(source);

		Link(graph.add_node(node), 0)
	}

	pub fn add_f32_into(graph: &mut DataFlowGraph, source: f32) -> Link {
		let node = Self::F32(source);

		Link(graph.add_node(node), 0)
	}

	pub fn add_f64_into(graph: &mut DataFlowGraph, source: f64) -> Link {
		let node = Self::F64(source);

		Link(graph.add_node(node), 0)
	}
}
