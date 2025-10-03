#![no_std]
#![expect(clippy::missing_panics_doc)]

extern crate alloc;

mod dot;
mod node;

use alloc::{boxed::Box, sync::Arc, vec::Vec};

use self::node::{
	base::{
		Call, DataDrop, DataNew, ElementsDrop, ElementsNew, ExtendType, GlobalGet, GlobalNew,
		GlobalSet, Identity, IntegerBinaryOperation, IntegerBinaryOperator,
		IntegerCompareOperation, IntegerCompareOperator, IntegerConvertToNumber, IntegerExtend,
		IntegerNarrow, IntegerTransmuteToNumber, IntegerType, IntegerUnaryOperation,
		IntegerUnaryOperator, IntegerWiden, LoadType, Location, MemoryCopy, MemoryFill, MemoryGrow,
		MemoryInit, MemoryLoad, MemoryNew, MemorySize, MemoryStore, Merge, NumberBinaryOperation,
		NumberBinaryOperator, NumberCompareOperation, NumberCompareOperator, NumberNarrow,
		NumberTransmuteToInteger, NumberTruncateToInteger, NumberType, NumberUnaryOperation,
		NumberUnaryOperator, NumberWiden, RefIsNull, StoreType, TableCopy, TableFill, TableGet,
		TableGrow, TableInit, TableNew, TableSet, TableSize,
	},
	control::{
		Export, FunctionType, GammaIn, GammaOut, Import, LambdaIn, LambdaOut, OmegaIn, OmegaOut,
		RegionIn, RegionOut, ThetaIn, ThetaOut,
	},
};

pub use self::{
	dot::Dot,
	node::{Link, Node, base, control},
};

/// A directed graph of nodes containing operations.
pub struct DataFlowGraph {
	nodes: Vec<Node>,
}

impl DataFlowGraph {
	#[must_use]
	pub const fn new() -> Self {
		Self { nodes: Vec::new() }
	}

	#[must_use]
	pub const fn len(&self) -> usize {
		self.nodes.len()
	}

	#[must_use]
	pub const fn is_empty(&self) -> bool {
		self.nodes.is_empty()
	}

	#[must_use]
	pub fn get(&self, id: u32) -> &Node {
		&self.nodes[usize::try_from(id).unwrap()]
	}

	pub fn get_mut(&mut self, id: u32) -> &mut Node {
		&mut self.nodes[usize::try_from(id).unwrap()]
	}

	pub const fn inner_mut(&mut self) -> &mut Vec<Node> {
		&mut self.nodes
	}

	pub fn nodes(&self) -> core::slice::Iter<'_, Node> {
		self.nodes.iter()
	}

	pub fn nodes_mut(&mut self) -> core::slice::IterMut<'_, Node> {
		self.nodes.iter_mut()
	}

	pub fn add_node(&mut self, node: Node) -> u32 {
		let position = self.nodes.len();

		self.nodes.push(node);

		position.try_into().unwrap()
	}

	pub fn add_lambda_in(&mut self, r#type: Box<FunctionType>, dependencies: Vec<Link>) -> u32 {
		let node = Node::LambdaIn(LambdaIn {
			output: u32::MAX,
			r#type,
			dependencies,
		});

		self.add_node(node)
	}

	/// # Panics
	///
	/// Panics if `input` is not a [`LambdaIn`] reference.
	pub fn add_lambda_out(&mut self, input: u32, results: Vec<Link>) -> u32 {
		let node = Node::LambdaOut(LambdaOut { input, results });
		let id = self.add_node(node);

		let LambdaIn { output, .. } = self.get_mut(input).as_mut_lambda_in().unwrap();

		*output = id;

		id
	}

	pub fn add_region_in(&mut self, input: u32) -> u32 {
		let node = Node::RegionIn(RegionIn {
			output: u32::MAX,
			input,
		});

		self.add_node(node)
	}

	/// # Panics
	///
	/// Panics if `input` is not a [`RegionIn`] reference.
	pub fn add_region_out(&mut self, input: u32, results: Vec<Link>) -> u32 {
		let node = Node::RegionOut(RegionOut {
			output: u32::MAX,
			input,
			results,
		});
		let id = self.add_node(node);

		let RegionIn { output, .. } = self.get_mut(input).as_mut_region_in().unwrap();

		*output = id;

		id
	}

	pub fn add_gamma_in(&mut self, arguments: Vec<Link>, condition: Link) -> u32 {
		let node = Node::GammaIn(GammaIn {
			output: u32::MAX,
			arguments,
			condition,
		});

		self.add_node(node)
	}

	/// # Panics
	///
	/// Panics if `input` is not a [`GammaIn`] reference,
	/// or any of the regions is not a [`RegionOut`] reference.
	pub fn add_gamma_out(&mut self, input: u32, regions: Vec<u32>) -> u32 {
		let id = self.add_trap().0;

		let GammaIn { output, .. } = self.get_mut(input).as_mut_gamma_in().unwrap();

		*output = id;

		for &region in &regions {
			let RegionOut { output, .. } = self.get_mut(region).as_mut_region_out().unwrap();

			*output = id;
		}

		*self.get_mut(id) = Node::GammaOut(GammaOut { input, regions });

		id
	}

	pub fn add_theta_in(&mut self, arguments: Vec<Link>) -> u32 {
		let node = Node::ThetaIn(ThetaIn {
			output: u32::MAX,
			arguments,
		});

		self.add_node(node)
	}

	/// # Panics
	///
	/// Panics if `input` is not a [`ThetaIn`] reference.
	pub fn add_theta_out(&mut self, input: u32, results: Vec<Link>, condition: Link) -> u32 {
		let node = Node::ThetaOut(ThetaOut {
			input,
			results,
			condition,
		});
		let id = self.add_node(node);

		let ThetaIn { output, .. } = self.get_mut(input).as_mut_theta_in().unwrap();

		*output = id;

		id
	}

	pub fn add_omega_in(&mut self) -> u32 {
		let node = Node::OmegaIn(OmegaIn { output: u32::MAX });

		self.add_node(node)
	}

	/// # Panics
	///
	/// Panics if `input` is not a [`OmegaIn`] reference.
	pub fn add_omega_out(&mut self, input: u32, state: Link, exports: Vec<Export>) -> u32 {
		let node = Node::OmegaOut(OmegaOut {
			input,
			state,
			exports,
		});
		let id = self.add_node(node);

		let OmegaIn { output, .. } = self.get_mut(input).as_mut_omega_in().unwrap();

		*output = id;

		id
	}

	pub fn add_import(
		&mut self,
		environment: Link,
		namespace: Arc<str>,
		identifier: Arc<str>,
	) -> Link {
		let node = Node::Import(
			Import {
				environment,
				namespace,
				identifier,
			}
			.into(),
		);

		Link(self.add_node(node), 0)
	}

	pub fn add_trap(&mut self) -> Link {
		Link(self.add_node(Node::Trap), 0)
	}

	pub fn add_null(&mut self) -> Link {
		Link(self.add_node(Node::Null), 0)
	}

	pub fn add_identity(&mut self, source: Link) -> Link {
		let node = Node::Identity(Identity { source });

		Link(self.add_node(node), 0)
	}

	pub fn add_i32(&mut self, value: i32) -> Link {
		let node = Node::I32(value);

		Link(self.add_node(node), 0)
	}

	pub fn add_i64(&mut self, value: i64) -> Link {
		let node = Node::I64(value);

		Link(self.add_node(node), 0)
	}

	pub fn add_f32(&mut self, value: f32) -> Link {
		let node = Node::F32(value);

		Link(self.add_node(node), 0)
	}

	pub fn add_f64(&mut self, value: f64) -> Link {
		let node = Node::F64(value);

		Link(self.add_node(node), 0)
	}

	pub fn add_ref_is_null(&mut self, source: Link) -> Link {
		let node = Node::RefIsNull(RefIsNull { source });

		Link(self.add_node(node), 0)
	}

	pub fn add_call(
		&mut self,
		function: Link,
		arguments: Vec<Link>,
		results: u16,
		states: u16,
	) -> u32 {
		let node = Node::Call(Call {
			function,
			arguments,
			results,
			states,
		});

		self.add_node(node)
	}

	pub fn add_merge(&mut self, states: Vec<Link>) -> Link {
		let node = Node::Merge(Merge { states });

		Link(self.add_node(node), 0)
	}

	pub fn add_integer_unary_operation(
		&mut self,
		source: Link,
		r#type: IntegerType,
		operator: IntegerUnaryOperator,
	) -> Link {
		let node = Node::IntegerUnaryOperation(IntegerUnaryOperation {
			source,
			r#type,
			operator,
		});

		Link(self.add_node(node), 0)
	}

	pub fn add_integer_binary_operation(
		&mut self,
		lhs: Link,
		rhs: Link,
		r#type: IntegerType,
		operator: IntegerBinaryOperator,
	) -> Link {
		let node = Node::IntegerBinaryOperation(IntegerBinaryOperation {
			lhs,
			rhs,
			r#type,
			operator,
		});

		Link(self.add_node(node), 0)
	}

	pub fn add_integer_compare_operation(
		&mut self,
		lhs: Link,
		rhs: Link,
		r#type: IntegerType,
		operator: IntegerCompareOperator,
	) -> Link {
		let node = Node::IntegerCompareOperation(IntegerCompareOperation {
			lhs,
			rhs,
			r#type,
			operator,
		});

		Link(self.add_node(node), 0)
	}

	pub fn add_integer_narrow(&mut self, source: Link) -> Link {
		let node = Node::IntegerNarrow(IntegerNarrow { source });

		Link(self.add_node(node), 0)
	}

	pub fn add_integer_widen(&mut self, source: Link) -> Link {
		let node = Node::IntegerWiden(IntegerWiden { source });

		Link(self.add_node(node), 0)
	}

	pub fn add_integer_extend(&mut self, source: Link, r#type: ExtendType) -> Link {
		let node = Node::IntegerExtend(IntegerExtend { source, r#type });

		Link(self.add_node(node), 0)
	}

	pub fn add_integer_convert_to_number(
		&mut self,
		source: Link,
		signed: bool,
		to: NumberType,
		from: IntegerType,
	) -> Link {
		let node = Node::IntegerConvertToNumber(IntegerConvertToNumber {
			source,
			signed,
			to,
			from,
		});

		Link(self.add_node(node), 0)
	}

	pub fn add_integer_transmute_to_number(&mut self, source: Link, from: IntegerType) -> Link {
		let node = Node::IntegerTransmuteToNumber(IntegerTransmuteToNumber { source, from });

		Link(self.add_node(node), 0)
	}

	pub fn add_number_unary_operation(
		&mut self,
		source: Link,
		r#type: NumberType,
		operator: NumberUnaryOperator,
	) -> Link {
		let node = Node::NumberUnaryOperation(NumberUnaryOperation {
			source,
			r#type,
			operator,
		});

		Link(self.add_node(node), 0)
	}

	pub fn add_number_binary_operation(
		&mut self,
		lhs: Link,
		rhs: Link,
		r#type: NumberType,
		operator: NumberBinaryOperator,
	) -> Link {
		let node = Node::NumberBinaryOperation(NumberBinaryOperation {
			lhs,
			rhs,
			r#type,
			operator,
		});

		Link(self.add_node(node), 0)
	}

	pub fn add_number_compare_operation(
		&mut self,
		lhs: Link,
		rhs: Link,
		r#type: NumberType,
		operator: NumberCompareOperator,
	) -> Link {
		let node = Node::NumberCompareOperation(NumberCompareOperation {
			lhs,
			rhs,
			r#type,
			operator,
		});

		Link(self.add_node(node), 0)
	}

	pub fn add_number_narrow(&mut self, source: Link) -> Link {
		let node = Node::NumberNarrow(NumberNarrow { source });

		Link(self.add_node(node), 0)
	}

	pub fn add_number_widen(&mut self, source: Link) -> Link {
		let node = Node::NumberWiden(NumberWiden { source });

		Link(self.add_node(node), 0)
	}

	pub fn add_number_truncate_to_integer(
		&mut self,
		source: Link,
		signed: bool,
		saturate: bool,
		to: IntegerType,
		from: NumberType,
	) -> Link {
		let node = Node::NumberTruncateToInteger(NumberTruncateToInteger {
			source,
			signed,
			saturate,
			to,
			from,
		});

		Link(self.add_node(node), 0)
	}

	pub fn add_number_transmute_to_integer(&mut self, source: Link, from: NumberType) -> Link {
		let node = Node::NumberTransmuteToInteger(NumberTransmuteToInteger { source, from });

		Link(self.add_node(node), 0)
	}

	pub fn add_global_new(&mut self, initializer: Link) -> Link {
		let node = Node::GlobalNew(GlobalNew { initializer });

		Link(self.add_node(node), 0)
	}

	pub fn add_global_get(&mut self, source: Link) -> Link {
		let node = Node::GlobalGet(GlobalGet { source });

		Link(self.add_node(node), GlobalGet::RESULT_PORT)
	}

	pub fn add_global_set(&mut self, destination: Link, source: Link) -> Link {
		let node = Node::GlobalSet(GlobalSet {
			destination,
			source,
		});

		Link(self.add_node(node), GlobalSet::STATE_PORT)
	}

	pub fn add_table_new(&mut self, initializer: Link, minimum: u32, maximum: u32) -> Link {
		let node = Node::TableNew(TableNew {
			initializer,
			minimum,
			maximum,
		});

		Link(self.add_node(node), 0)
	}

	pub fn add_table_get(&mut self, source: Location) -> Link {
		let node = Node::TableGet(TableGet { source });

		Link(self.add_node(node), TableGet::RESULT_PORT)
	}

	pub fn add_table_set(&mut self, destination: Location, source: Link) -> Link {
		let node = Node::TableSet(TableSet {
			destination,
			source,
		});

		Link(self.add_node(node), TableSet::STATE_PORT)
	}

	pub fn add_table_size(&mut self, source: Link) -> Link {
		let node = Node::TableSize(TableSize { source });

		Link(self.add_node(node), TableSize::RESULT_PORT)
	}

	pub fn add_table_grow(
		&mut self,
		destination: Link,
		initializer: Link,
		size: Link,
	) -> (Link, Link) {
		let node = Node::TableGrow(TableGrow {
			destination,
			initializer,
			size,
		});
		let id = self.add_node(node);

		(
			Link(id, TableGrow::RESULT_PORT),
			Link(id, TableGrow::STATE_PORT),
		)
	}

	pub fn add_table_fill(&mut self, destination: Location, source: Link, size: Link) -> Link {
		let node = Node::TableFill(TableFill {
			destination,
			source,
			size,
		});

		Link(self.add_node(node), TableFill::STATE_PORT)
	}

	pub fn add_table_copy(&mut self, destination: Location, source: Location, size: Link) -> Link {
		let node = Node::TableCopy(TableCopy {
			destination,
			source,
			size,
		});

		Link(self.add_node(node), TableCopy::DESTINATION_STATE_PORT)
	}

	pub fn add_table_init(&mut self, destination: Location, source: Location, size: Link) -> Link {
		let node = Node::TableInit(TableInit {
			destination,
			source,
			size,
		});

		Link(self.add_node(node), TableInit::DESTINATION_STATE_PORT)
	}

	pub fn add_elements_new(&mut self, content: Vec<Link>) -> Link {
		let node = Node::ElementsNew(ElementsNew { content });

		Link(self.add_node(node), 0)
	}

	pub fn add_elements_drop(&mut self, source: Link) -> Link {
		let node = Node::ElementsDrop(ElementsDrop { source });

		Link(self.add_node(node), ElementsDrop::STATE_PORT)
	}

	pub fn add_memory_new(&mut self, minimum: u32, maximum: u32) -> Link {
		let node = Node::MemoryNew(MemoryNew { minimum, maximum });

		Link(self.add_node(node), 0)
	}

	pub fn add_memory_load(&mut self, source: Location, r#type: LoadType) -> Link {
		let node = Node::MemoryLoad(MemoryLoad { source, r#type });

		Link(self.add_node(node), MemoryLoad::RESULT_PORT)
	}

	pub fn add_memory_store(
		&mut self,
		destination: Location,
		source: Link,
		r#type: StoreType,
	) -> Link {
		let node = Node::MemoryStore(MemoryStore {
			destination,
			source,
			r#type,
		});

		Link(self.add_node(node), MemoryStore::STATE_PORT)
	}

	pub fn add_memory_size(&mut self, source: Link) -> Link {
		let node = Node::MemorySize(MemorySize { source });

		Link(self.add_node(node), MemorySize::RESULT_PORT)
	}

	pub fn add_memory_grow(&mut self, destination: Link, size: Link) -> (Link, Link) {
		let node = Node::MemoryGrow(MemoryGrow { destination, size });
		let id = self.add_node(node);

		(
			Link(id, MemoryGrow::RESULT_PORT),
			Link(id, MemoryGrow::STATE_PORT),
		)
	}

	pub fn add_memory_fill(&mut self, destination: Location, byte: Link, size: Link) -> Link {
		let node = Node::MemoryFill(MemoryFill {
			destination,
			byte,
			size,
		});

		Link(self.add_node(node), MemoryFill::STATE_PORT)
	}

	pub fn add_memory_copy(&mut self, destination: Location, source: Location, size: Link) -> Link {
		let node = Node::MemoryCopy(MemoryCopy {
			destination,
			source,
			size,
		});

		Link(self.add_node(node), MemoryCopy::DESTINATION_STATE_PORT)
	}

	pub fn add_memory_init(&mut self, destination: Location, source: Location, size: Link) -> Link {
		let node = Node::MemoryInit(MemoryInit {
			destination,
			source,
			size,
		});

		Link(self.add_node(node), MemoryInit::DESTINATION_STATE_PORT)
	}

	pub fn add_data_new(&mut self, content: Arc<[u8]>) -> Link {
		let node = Node::DataNew(DataNew { content });

		Link(self.add_node(node), 0)
	}

	pub fn add_data_drop(&mut self, source: Link) -> Link {
		let node = Node::DataDrop(DataDrop { source });

		Link(self.add_node(node), DataDrop::STATE_PORT)
	}
}

impl Default for DataFlowGraph {
	fn default() -> Self {
		Self::new()
	}
}
