use alloc::sync::{Arc, Weak};

use list::resizable::Resizable;
use parking_lot::Mutex;

use crate::Link;

use super::{
	Node,
	control::{
		Branch, BranchArguments, BranchResults, Export, Function, FunctionArguments,
		FunctionCaptures, FunctionResults, Import, Match, Module, ModuleArguments, ModuleResults,
		Repeat, RepeatArguments, RepeatResults, ValueType,
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
		NumberUnaryOperator, NumberWiden, RefIsNull, StoreType, TableCopy, TableDrop, TableFill,
		TableGet, TableGrow, TableNew, TableSet, TableSize,
	},
};

impl Identity {
	/// Adds an identity node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, sources: Resizable<Link, 4>) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::Identity(Self { sources });

		nodes.push(node);

		id
	}

	/// Returns the number of output ports.
	#[must_use]
	pub fn result_count(&self) -> u16 {
		self.sources
			.len()
			.try_into()
			.unwrap_or_else(|_| unreachable!())
	}
}

impl Fence {
	/// Adds a fence node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, sources: Resizable<Link, 4>) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::Fence(Self { sources });

		nodes.push(node);

		id
	}

	/// Returns the number of output ports.
	#[must_use]
	pub fn result_count(&self) -> u16 {
		self.sources
			.len()
			.try_into()
			.unwrap_or_else(|_| unreachable!())
	}
}

impl Apply {
	/// Adds a function application node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		function: Link,
		arguments: Vec<Link>,
		results: u16,
	) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::Apply(Self {
			function,
			arguments,
			results,
		});

		nodes.push(node);

		id
	}

	/// Returns the number of output ports.
	#[must_use]
	pub const fn result_count(&self) -> u16 {
		self.results
	}
}

impl RefIsNull {
	/// Adds a reference null check node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::RefIsNull(Self { source });

		nodes.push(node);

		Link(id, 0)
	}
}

impl IntegerUnaryOperation {
	/// Adds an integer unary operation node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		source: Link,
		kind: IntegerType,
		operator: IntegerUnaryOperator,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::IntegerUnaryOperation(Self {
			source,
			kind,
			operator,
		});

		nodes.push(node);

		Link(id, 0)
	}
}

impl IntegerBinaryOperation {
	/// Adds an integer binary operation node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		lhs: Link,
		rhs: Link,
		kind: IntegerType,
		operator: IntegerBinaryOperator,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::IntegerBinaryOperation(Self {
			lhs,
			rhs,
			kind,
			operator,
		});

		nodes.push(node);

		Link(id, 0)
	}
}

impl IntegerCompareOperation {
	/// Adds an integer comparison node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		lhs: Link,
		rhs: Link,
		kind: IntegerType,
		operator: IntegerCompareOperator,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::IntegerCompareOperation(Self {
			lhs,
			rhs,
			kind,
			operator,
		});

		nodes.push(node);

		Link(id, 0)
	}
}

impl IntegerNarrow {
	/// Adds an integer narrowing node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::IntegerNarrow(Self { source });

		nodes.push(node);

		Link(id, 0)
	}
}

impl IntegerWiden {
	/// Adds an integer widening node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::IntegerWiden(Self { source });

		nodes.push(node);

		Link(id, 0)
	}
}

impl IntegerExtend {
	/// Adds an integer sign-extension node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, source: Link, kind: ExtendType) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::IntegerExtend(Self { source, kind });

		nodes.push(node);

		Link(id, 0)
	}
}

impl IntegerConvertToNumber {
	/// Adds an integer-to-floating-point conversion node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		source: Link,
		signed: bool,
		to: NumberType,
		from: IntegerType,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::IntegerConvertToNumber(Self {
			source,
			signed,
			to,
			from,
		});

		nodes.push(node);

		Link(id, 0)
	}
}

impl IntegerTransmuteToNumber {
	/// Adds an integer-to-floating-point reinterpretation node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, source: Link, from: IntegerType) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::IntegerTransmuteToNumber(Self { source, from });

		nodes.push(node);

		Link(id, 0)
	}
}

impl NumberUnaryOperation {
	/// Adds a floating-point unary operation node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		source: Link,
		kind: NumberType,
		operator: NumberUnaryOperator,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::NumberUnaryOperation(Self {
			source,
			kind,
			operator,
		});

		nodes.push(node);

		Link(id, 0)
	}
}

impl NumberBinaryOperation {
	/// Adds a floating-point binary operation node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		lhs: Link,
		rhs: Link,
		kind: NumberType,
		operator: NumberBinaryOperator,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::NumberBinaryOperation(Self {
			lhs,
			rhs,
			kind,
			operator,
		});

		nodes.push(node);

		Link(id, 0)
	}
}

impl NumberCompareOperation {
	/// Adds a floating-point comparison node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		lhs: Link,
		rhs: Link,
		kind: NumberType,
		operator: NumberCompareOperator,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::NumberCompareOperation(Self {
			lhs,
			rhs,
			kind,
			operator,
		});

		nodes.push(node);

		Link(id, 0)
	}
}

impl NumberTruncateToInteger {
	/// Adds a floating-point-to-integer truncation node to the graph.
	#[expect(
		clippy::too_many_arguments,
		reason = "conversion constructor requires source, signedness, saturation, and type pair"
	)]
	pub fn add_into(
		nodes: &mut Vec<Node>,
		source: Link,
		signed: bool,
		saturate: bool,
		to: IntegerType,
		from: NumberType,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::NumberTruncateToInteger(Self {
			source,
			signed,
			saturate,
			to,
			from,
		});

		nodes.push(node);

		Link(id, 0)
	}
}

impl NumberTransmuteToInteger {
	/// Adds a floating-point-to-integer reinterpretation node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, source: Link, from: NumberType) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::NumberTransmuteToInteger(Self { source, from });

		nodes.push(node);

		Link(id, 0)
	}
}

impl NumberNarrow {
	/// Adds a floating-point narrowing node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::NumberNarrow(Self { source });

		nodes.push(node);

		Link(id, 0)
	}
}

impl NumberWiden {
	/// Adds a floating-point widening node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::NumberWiden(Self { source });

		nodes.push(node);

		Link(id, 0)
	}
}

impl GlobalNew {
	/// Adds a global creation node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, initializer: Link) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::GlobalNew(Self { initializer });

		nodes.push(node);

		Link(id, 0)
	}
}

impl GlobalGet {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a global read node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> (Link, Link) {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::GlobalGet(Self { source });

		nodes.push(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl GlobalSet {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 1;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a global write node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, destination: Link, source: Link) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::GlobalSet(Self {
			destination,
			source,
		});

		nodes.push(node);

		Link(id, Self::STATE_PORT)
	}
}

impl TableNew {
	/// Adds a table creation node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		initializer: Vec<(Link, u32)>,
		minimum: u32,
		maximum: u32,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::TableNew(Self {
			initializer,
			minimum,
			maximum,
		});

		nodes.push(node);

		Link(id, 0)
	}
}

impl TableGet {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a table read node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, source: Location) -> (Link, Link) {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::TableGet(Self { source });

		nodes.push(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl TableSet {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 1;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a table write node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, destination: Location, source: Link) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::TableSet(Self {
			destination,
			source,
		});

		nodes.push(node);

		Link(id, Self::STATE_PORT)
	}
}

impl TableSize {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a table size query node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> (Link, Link) {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::TableSize(Self { source });

		nodes.push(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl TableGrow {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a table grow node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		destination: Link,
		initializer: Link,
		size: Link,
	) -> (Link, Link) {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::TableGrow(Self {
			destination,
			initializer,
			size,
		});

		nodes.push(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl TableFill {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 1;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a table fill node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		destination: Location,
		source: Link,
		size: Link,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::TableFill(Self {
			destination,
			source,
			size,
		});

		nodes.push(node);

		Link(id, Self::STATE_PORT)
	}
}

impl TableCopy {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the destination state token.
	pub const DESTINATION_STATE_PORT: u16 = 0;
	/// The port index for the source state token.
	pub const SOURCE_STATE_PORT: u16 = 1;

	/// Adds a table copy node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		destination: Location,
		source: Location,
		size: Link,
	) -> (Link, Link) {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::TableCopy(Self {
			destination,
			source,
			size,
		});

		nodes.push(node);

		(
			Link(id, Self::DESTINATION_STATE_PORT),
			Link(id, Self::SOURCE_STATE_PORT),
		)
	}
}

impl TableDrop {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 1;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a table drop node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::TableDrop(Self { source });

		nodes.push(node);

		Link(id, Self::STATE_PORT)
	}
}

impl MemoryNew {
	/// Adds a memory creation node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		initializer: Vec<(Arc<[u8]>, u32)>,
		minimum: u32,
		maximum: u32,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::MemoryNew(Self {
			initializer,
			minimum,
			maximum,
		});

		nodes.push(node);

		Link(id, 0)
	}
}

impl MemoryLoad {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a memory load node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, source: Location, kind: LoadType) -> (Link, Link) {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::MemoryLoad(Self { source, kind });

		nodes.push(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl MemoryStore {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 1;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a memory store node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		destination: Location,
		source: Link,
		kind: StoreType,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::MemoryStore(Self {
			destination,
			source,
			kind,
		});

		nodes.push(node);

		Link(id, Self::STATE_PORT)
	}
}

impl MemorySize {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a memory size query node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> (Link, Link) {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::MemorySize(Self { source });

		nodes.push(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl MemoryGrow {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the result value.
	pub const RESULT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a memory grow node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, destination: Link, size: Link) -> (Link, Link) {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::MemoryGrow(Self { destination, size });

		nodes.push(node);

		(Link(id, Self::RESULT_PORT), Link(id, Self::STATE_PORT))
	}
}

impl MemoryFill {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 1;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a memory fill node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, destination: Location, byte: Link, size: Link) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::MemoryFill(Self {
			destination,
			byte,
			size,
		});

		nodes.push(node);

		Link(id, Self::STATE_PORT)
	}
}

impl MemoryCopy {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the destination state token.
	pub const DESTINATION_STATE_PORT: u16 = 0;
	/// The port index for the source state token.
	pub const SOURCE_STATE_PORT: u16 = 1;

	/// Adds a memory copy node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		destination: Location,
		source: Location,
		size: Link,
	) -> (Link, Link) {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::MemoryCopy(Self {
			destination,
			source,
			size,
		});

		nodes.push(node);

		(
			Link(id, Self::DESTINATION_STATE_PORT),
			Link(id, Self::SOURCE_STATE_PORT),
		)
	}
}

impl MemoryDrop {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 1;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 0;

	/// Adds a memory drop node to the graph.
	pub fn add_into(nodes: &mut Vec<Node>, source: Link) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::MemoryDrop(Self { source });

		nodes.push(node);

		Link(id, Self::STATE_PORT)
	}
}

impl Import {
	/// Adds an import node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		environment: Link,
		namespace: Arc<str>,
		identifier: Arc<str>,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::Import(
			Self {
				environment,
				namespace,
				identifier,
			}
			.into(),
		);

		nodes.push(node);

		Link(id, 0)
	}
}

impl Module {
	/// Node index of the arguments boundary node.
	pub const ARGUMENTS_ID: u32 = 0;

	/// Creates a new module region.
	pub fn create<F>(initializer: F) -> Arc<Mutex<Self>>
	where
		F: FnOnce(&mut Vec<Node>, u32) -> (Link, Vec<Export>),
	{
		let create = |weak: &Weak<Mutex<Self>>| {
			let mut nodes = Vec::new();

			let module_arguments = ModuleArguments::add_into(&mut nodes, Weak::clone(weak));
			let (state, exports) = initializer(&mut nodes, module_arguments);

			ModuleResults::add_into(&mut nodes, Weak::clone(weak), state, exports);

			Mutex::new(Self { nodes })
		};

		Arc::new_cyclic(create)
	}
}

impl ModuleArguments {
	/// The number of output ports.
	pub const RESULT_COUNT: u16 = 2;
	/// The port index for the environment.
	pub const ENVIRONMENT_PORT: u16 = 0;
	/// The port index for the state token.
	pub const STATE_PORT: u16 = 1;

	/// Adds a module arguments boundary node to the region.
	pub fn add_into(nodes: &mut Vec<Node>, parent: Weak<Mutex<Module>>) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::ModuleArguments(Self { parent });

		nodes.push(node);

		id
	}
}

impl ModuleResults {
	/// Adds a module results boundary node to the region.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		parent: Weak<Mutex<Module>>,
		state: Link,
		exports: Vec<Export>,
	) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::ModuleResults(Self {
			parent,
			state,
			exports,
		});

		nodes.push(node);

		id
	}
}

impl Function {
	/// Node index of the captures boundary node.
	pub const CAPTURES_ID: u32 = 0;
	/// Node index of the arguments boundary node.
	pub const ARGUMENTS_ID: u32 = 1;

	/// Creates a new function region.
	pub fn create<F>(
		argument_types: Resizable<ValueType, 15>,
		result_types: Resizable<ValueType, 15>,
		captures: Vec<Link>,
		initializer: F,
	) -> Arc<Mutex<Self>>
	where
		F: FnOnce(&mut Vec<Node>, u32, u32) -> Vec<Link>,
	{
		let create = |weak: &Weak<Mutex<Self>>| {
			let mut nodes = Vec::new();

			let function_captures = FunctionCaptures::add_into(&mut nodes, Weak::clone(weak));
			let function_arguments = FunctionArguments::add_into(&mut nodes, Weak::clone(weak));
			let sources = initializer(&mut nodes, function_captures, function_arguments);

			FunctionResults::add_into(&mut nodes, Weak::clone(weak), sources);

			Mutex::new(Self {
				argument_types,
				result_types,
				captures,
				nodes,
			})
		};

		Arc::new_cyclic(create)
	}

	/// Adds a function region node to the graph.
	pub fn add_into<F>(
		nodes: &mut Vec<Node>,
		argument_types: Resizable<ValueType, 15>,
		result_types: Resizable<ValueType, 15>,
		captures: Vec<Link>,
		initializer: F,
	) -> Link
	where
		F: FnOnce(&mut Vec<Node>, u32, u32) -> Vec<Link>,
	{
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::Function(Self::create(
			argument_types,
			result_types,
			captures,
			initializer,
		));

		nodes.push(node);

		Link(id, 0)
	}

	/// Returns the number of captures.
	#[must_use]
	pub fn capture_count(&self) -> u16 {
		self.captures
			.len()
			.try_into()
			.unwrap_or_else(|_| unreachable!())
	}

	/// Returns the number of arguments.
	#[must_use]
	pub fn argument_count(&self) -> u16 {
		self.argument_types
			.len()
			.try_into()
			.unwrap_or_else(|_| unreachable!())
	}
}

impl FunctionCaptures {
	/// Adds a function captures boundary node to the region.
	pub fn add_into(nodes: &mut Vec<Node>, parent: Weak<Mutex<Function>>) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::FunctionCaptures(Self { parent });

		nodes.push(node);

		id
	}

	/// Returns the number of output ports.
	#[must_use]
	pub fn result_count(&self) -> u16 {
		let function = self.parent.upgrade().unwrap_or_else(|| unreachable!());

		function.lock().capture_count()
	}
}

impl FunctionArguments {
	/// Adds a function arguments boundary node to the region.
	pub fn add_into(nodes: &mut Vec<Node>, parent: Weak<Mutex<Function>>) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::FunctionArguments(Self { parent });

		nodes.push(node);

		id
	}

	/// Returns the number of output ports.
	#[must_use]
	pub fn result_count(&self) -> u16 {
		let function = self.parent.upgrade().unwrap_or_else(|| unreachable!());

		function.lock().argument_count()
	}
}

impl FunctionResults {
	/// Adds a function results boundary node to the region.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		parent: Weak<Mutex<Function>>,
		sources: Vec<Link>,
	) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::FunctionResults(Self { parent, sources });

		nodes.push(node);

		id
	}

	/// Returns the number of result values.
	#[must_use]
	pub fn argument_count(&self) -> u16 {
		self.sources
			.len()
			.try_into()
			.unwrap_or_else(|_| unreachable!())
	}
}

impl Match {
	/// Creates a new match region.
	pub fn create<F>(arguments: Vec<Link>, condition: Link, initializer: F) -> Arc<Mutex<Self>>
	where
		F: FnOnce(&Weak<Mutex<Self>>) -> Vec<Arc<Mutex<Branch>>>,
	{
		let create = |weak: &Weak<Mutex<Self>>| {
			let branches = initializer(weak);

			Mutex::new(Self {
				arguments,
				condition,
				branches,
			})
		};

		Arc::new_cyclic(create)
	}

	/// Adds a match region node to the graph.
	pub fn add_into<F>(
		nodes: &mut Vec<Node>,
		arguments: Vec<Link>,
		condition: Link,
		initializer: F,
	) -> u32
	where
		F: FnOnce(&Weak<Mutex<Self>>) -> Vec<Arc<Mutex<Branch>>>,
	{
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::Match(Self::create(arguments, condition, initializer));

		nodes.push(node);

		id
	}

	/// Creates a new if-else match region.
	pub fn create_if<F, T>(
		arguments: Vec<Link>,
		condition: Link,
		on_false: F,
		on_true: T,
	) -> Arc<Mutex<Self>>
	where
		F: FnOnce(&mut Vec<Node>, u32) -> Vec<Link>,
		T: FnOnce(&mut Vec<Node>, u32) -> Vec<Link>,
	{
		Self::create(arguments, condition, |parent| {
			vec![
				Branch::create(Weak::clone(parent), on_false),
				Branch::create(Weak::clone(parent), on_true),
			]
		})
	}

	/// Adds an if-else structure to the graph.
	pub fn add_if_into<F, T>(
		nodes: &mut Vec<Node>,
		arguments: Vec<Link>,
		condition: Link,
		on_false: F,
		on_true: T,
	) -> u32
	where
		F: FnOnce(&mut Vec<Node>, u32) -> Vec<Link>,
		T: FnOnce(&mut Vec<Node>, u32) -> Vec<Link>,
	{
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::Match(Self::create_if(arguments, condition, on_false, on_true));

		nodes.push(node);

		id
	}

	/// Returns the number of arguments.
	#[must_use]
	pub fn argument_count(&self) -> u16 {
		self.arguments
			.len()
			.try_into()
			.unwrap_or_else(|_| unreachable!())
	}

	/// Returns the number of output ports.
	#[must_use]
	pub fn result_count(&self) -> u16 {
		self.branches
			.first()
			.map_or(0, |branch| branch.lock().result_count())
	}
}

impl Branch {
	/// Node index of the arguments boundary node.
	pub const ARGUMENTS_ID: u32 = 0;

	/// Creates a new branch region.
	pub fn create<F>(parent: Weak<Mutex<Match>>, initializer: F) -> Arc<Mutex<Self>>
	where
		F: FnOnce(&mut Vec<Node>, u32) -> Vec<Link>,
	{
		let create = |weak: &Weak<Mutex<Self>>| {
			let mut nodes = Vec::new();

			let branch_arguments = BranchArguments::add_into(&mut nodes, Weak::clone(weak));
			let sources = initializer(&mut nodes, branch_arguments);

			BranchResults::add_into(&mut nodes, Weak::clone(weak), sources);

			Mutex::new(Self { nodes, parent })
		};

		Arc::new_cyclic(create)
	}

	/// Returns the number of arguments.
	#[must_use]
	pub fn argument_count(&self) -> u16 {
		let matcher = self.parent.upgrade().unwrap_or_else(|| unreachable!());

		matcher.lock().argument_count()
	}

	/// Returns the number of output ports.
	#[must_use]
	pub fn result_count(&self) -> u16 {
		self.results().argument_count()
	}
}

impl BranchArguments {
	/// Adds a branch arguments boundary node to the region.
	pub fn add_into(nodes: &mut Vec<Node>, parent: Weak<Mutex<Branch>>) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::BranchArguments(Self { parent });

		nodes.push(node);

		id
	}

	/// Returns the number of output ports.
	#[must_use]
	pub fn result_count(&self) -> u16 {
		let branch = self.parent.upgrade().unwrap_or_else(|| unreachable!());

		branch.lock().argument_count()
	}
}

impl BranchResults {
	/// Adds a branch results boundary node to the region.
	pub fn add_into(nodes: &mut Vec<Node>, parent: Weak<Mutex<Branch>>, sources: Vec<Link>) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::BranchResults(Self { parent, sources });

		nodes.push(node);

		id
	}

	/// Returns the number of result values.
	#[must_use]
	pub fn argument_count(&self) -> u16 {
		self.sources
			.len()
			.try_into()
			.unwrap_or_else(|_| unreachable!())
	}
}

impl Repeat {
	/// Node index of the arguments boundary node.
	pub const ARGUMENTS_ID: u32 = 0;

	/// Creates a new repeat region.
	pub fn create<F>(arguments: Vec<Link>, initializer: F) -> Arc<Mutex<Self>>
	where
		F: FnOnce(&mut Vec<Node>, u32) -> (Vec<Link>, Link),
	{
		let create = |weak: &Weak<Mutex<Self>>| {
			let mut nodes = Vec::new();

			let repeat_arguments = RepeatArguments::add_into(&mut nodes, Weak::clone(weak));
			let (sources, condition) = initializer(&mut nodes, repeat_arguments);

			RepeatResults::add_into(&mut nodes, Weak::clone(weak), sources, condition);

			Mutex::new(Self { arguments, nodes })
		};

		Arc::new_cyclic(create)
	}

	/// Adds a repeat region node to the graph.
	pub fn add_into<F>(nodes: &mut Vec<Node>, arguments: Vec<Link>, initializer: F) -> u32
	where
		F: FnOnce(&mut Vec<Node>, u32) -> (Vec<Link>, Link),
	{
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::Repeat(Self::create(arguments, initializer));

		nodes.push(node);

		id
	}

	/// Returns the number of arguments.
	#[must_use]
	pub fn argument_count(&self) -> u16 {
		self.arguments
			.len()
			.try_into()
			.unwrap_or_else(|_| unreachable!())
	}

	/// Returns the number of output ports.
	#[must_use]
	pub fn result_count(&self) -> u16 {
		self.results().argument_count()
	}
}

impl RepeatArguments {
	/// Adds a repeat arguments boundary node to the region.
	pub fn add_into(nodes: &mut Vec<Node>, parent: Weak<Mutex<Repeat>>) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::RepeatArguments(Self { parent });

		nodes.push(node);

		id
	}

	/// Returns the number of output ports.
	#[must_use]
	pub fn result_count(&self) -> u16 {
		let repeat = self.parent.upgrade().unwrap_or_else(|| unreachable!());

		repeat.lock().argument_count()
	}
}

impl RepeatResults {
	/// Adds a repeat results boundary node to the region.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		parent: Weak<Mutex<Repeat>>,
		sources: Vec<Link>,
		condition: Link,
	) -> u32 {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::RepeatResults(Self {
			parent,
			sources,
			condition,
		});

		nodes.push(node);

		id
	}

	/// Returns the number of result values.
	#[must_use]
	pub fn argument_count(&self) -> u16 {
		self.sources
			.len()
			.try_into()
			.unwrap_or_else(|_| unreachable!())
	}
}

impl Node {
	/// Adds a trap node to the graph.
	pub fn add_trap_into(nodes: &mut Vec<Self>) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());

		nodes.push(Self::Trap);

		Link(id, 0)
	}

	/// Adds a null reference constant node to the graph.
	pub fn add_null_into(nodes: &mut Vec<Self>) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());

		nodes.push(Self::Null);

		Link(id, 0)
	}

	/// Adds a 32-bit integer constant node to the graph.
	pub fn add_i32_into(nodes: &mut Vec<Self>, source: i32) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Self::I32(source);

		nodes.push(node);

		Link(id, 0)
	}

	/// Adds a 64-bit integer constant node to the graph.
	pub fn add_i64_into(nodes: &mut Vec<Self>, source: i64) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Self::I64(source);

		nodes.push(node);

		Link(id, 0)
	}

	/// Adds a 32-bit float constant node to the graph.
	pub fn add_f32_into(nodes: &mut Vec<Self>, source: f32) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Self::F32(source);

		nodes.push(node);

		Link(id, 0)
	}

	/// Adds a 64-bit float constant node to the graph.
	pub fn add_f64_into(nodes: &mut Vec<Self>, source: f64) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Self::F64(source);

		nodes.push(node);

		Link(id, 0)
	}
}

impl Node {
	/// Returns the number of output ports for this node.
	#[must_use]
	#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
	pub fn result_count(&self) -> u16 {
		match self {
			Self::Function(_)
			| Self::Import(_)
			| Self::Trap
			| Self::Null
			| Self::I32(_)
			| Self::I64(_)
			| Self::F32(_)
			| Self::F64(_)
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
			| Self::TableNew(_)
			| Self::MemoryNew(_) => 1,

			Self::Match(arc) => arc.lock().result_count(),
			Self::Repeat(arc) => arc.lock().result_count(),

			Self::ModuleArguments(_) => ModuleArguments::RESULT_COUNT,

			Self::ModuleResults(_)
			| Self::FunctionResults(_)
			| Self::BranchResults(_)
			| Self::RepeatResults(_) => 0,

			Self::FunctionCaptures(node) => node.result_count(),
			Self::FunctionArguments(node) => node.result_count(),

			Self::BranchArguments(node) => node.result_count(),

			Self::RepeatArguments(node) => node.result_count(),

			Self::Host(host) => host.result_count(),

			Self::Identity(node) => node.result_count(),
			Self::Fence(node) => node.result_count(),
			Self::Apply(node) => node.result_count(),

			Self::GlobalGet(_) => GlobalGet::RESULT_COUNT,
			Self::GlobalSet(_) => GlobalSet::RESULT_COUNT,

			Self::TableGet(_) => TableGet::RESULT_COUNT,
			Self::TableSet(_) => TableSet::RESULT_COUNT,
			Self::TableSize(_) => TableSize::RESULT_COUNT,
			Self::TableGrow(_) => TableGrow::RESULT_COUNT,
			Self::TableFill(_) => TableFill::RESULT_COUNT,
			Self::TableCopy(_) => TableCopy::RESULT_COUNT,
			Self::TableDrop(_) => TableDrop::RESULT_COUNT,

			Self::MemoryLoad(_) => MemoryLoad::RESULT_COUNT,
			Self::MemoryStore(_) => MemoryStore::RESULT_COUNT,
			Self::MemorySize(_) => MemorySize::RESULT_COUNT,
			Self::MemoryGrow(_) => MemoryGrow::RESULT_COUNT,
			Self::MemoryFill(_) => MemoryFill::RESULT_COUNT,
			Self::MemoryCopy(_) => MemoryCopy::RESULT_COUNT,
			Self::MemoryDrop(_) => MemoryDrop::RESULT_COUNT,
		}
	}
}
