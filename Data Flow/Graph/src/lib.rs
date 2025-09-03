#![no_std]
#![expect(clippy::missing_panics_doc)]

extern crate alloc;

mod dot;
mod node;

use alloc::{boxed::Box, sync::Arc, vec::Vec};

use self::node::{
	mvp::{
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
	nested::{
		Export, FunctionType, GammaIn, GammaOut, Import, LambdaIn, LambdaOut, OmegaIn, OmegaOut,
		RegionIn, RegionOut, ThetaIn, ThetaOut,
	},
};

pub use self::{
	dot::Dot,
	node::{Link, Node, mvp, nested},
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
		self.add_node(Node::LambdaIn(LambdaIn {
			output: u32::MAX,
			r#type,
			dependencies,
		}))
	}

	/// # Panics
	///
	/// Panics if `input` is not a [`LambdaIn`] reference.
	pub fn add_lambda_out(&mut self, input: u32, results: Vec<Link>) -> u32 {
		let lambda_out = self.add_node(Node::LambdaOut(LambdaOut { input, results }));

		let LambdaIn { output, .. } = self.get_mut(input).as_mut_lambda_in().unwrap();

		*output = lambda_out;

		lambda_out
	}

	pub fn add_region_in(&mut self, input: u32) -> u32 {
		self.add_node(Node::RegionIn(RegionIn {
			output: u32::MAX,
			input,
		}))
	}

	/// # Panics
	///
	/// Panics if `input` is not a [`RegionIn`] reference.
	pub fn add_region_out(&mut self, input: u32, results: Vec<Link>) -> u32 {
		let region_out = self.add_node(Node::RegionOut(RegionOut {
			output: u32::MAX,
			input,
			results,
		}));

		let RegionIn { output, .. } = self.get_mut(input).as_mut_region_in().unwrap();

		*output = region_out;

		region_out
	}

	pub fn add_gamma_in(&mut self, condition: Link, arguments: Vec<Link>) -> u32 {
		self.add_node(Node::GammaIn(GammaIn {
			output: u32::MAX,
			condition,
			arguments,
		}))
	}

	/// # Panics
	///
	/// Panics if `input` is not a [`GammaIn`] reference,
	/// or any of the regions is not a [`RegionOut`] reference.
	pub fn add_gamma_out(&mut self, input: u32, regions: Vec<u32>) -> u32 {
		let gamma_out = self.add_trap().0;

		let GammaIn { output, .. } = self.get_mut(input).as_mut_gamma_in().unwrap();

		*output = gamma_out;

		for &region in &regions {
			let RegionOut { output, .. } = self.get_mut(region).as_mut_region_out().unwrap();

			*output = gamma_out;
		}

		*self.get_mut(gamma_out) = Node::GammaOut(GammaOut { input, regions });

		gamma_out
	}

	pub fn add_theta_in(&mut self, arguments: Vec<Link>) -> u32 {
		self.add_node(Node::ThetaIn(ThetaIn {
			output: u32::MAX,
			arguments,
		}))
	}

	/// # Panics
	///
	/// Panics if `input` is not a [`ThetaIn`] reference.
	pub fn add_theta_out(&mut self, input: u32, condition: Link, results: Vec<Link>) -> u32 {
		let theta_out = self.add_node(Node::ThetaOut(ThetaOut {
			input,
			condition,
			results,
		}));

		let ThetaIn { output, .. } = self.get_mut(input).as_mut_theta_in().unwrap();

		*output = theta_out;

		theta_out
	}

	pub fn add_omega_in(&mut self) -> u32 {
		self.add_node(Node::OmegaIn(OmegaIn { output: u32::MAX }))
	}

	/// # Panics
	///
	/// Panics if `input` is not a [`OmegaIn`] reference.
	pub fn add_omega_out(&mut self, input: u32, state: Link, exports: Vec<Export>) -> u32 {
		let omega_out = self.add_node(Node::OmegaOut(OmegaOut {
			input,
			state,
			exports,
		}));

		let OmegaIn { output, .. } = self.get_mut(input).as_mut_omega_in().unwrap();

		*output = omega_out;

		omega_out
	}

	pub fn add_import(
		&mut self,
		environment: Link,
		namespace: Arc<str>,
		identifier: Arc<str>,
	) -> Link {
		let import = Node::Import(
			Import {
				environment,
				namespace,
				identifier,
			}
			.into(),
		);

		Link(self.add_node(import), 0)
	}

	pub fn add_trap(&mut self) -> Link {
		Link(self.add_node(Node::Trap), 0)
	}

	pub fn add_null(&mut self) -> Link {
		Link(self.add_node(Node::Null), 0)
	}

	pub fn add_identity(&mut self, source: Link) -> Link {
		let identity = Node::Identity(Identity { source });

		Link(self.add_node(identity), 0)
	}

	pub fn add_i32(&mut self, value: i32) -> Link {
		let i32 = Node::I32(value);

		Link(self.add_node(i32), 0)
	}

	pub fn add_i64(&mut self, value: i64) -> Link {
		let i64 = Node::I64(value);

		Link(self.add_node(i64), 0)
	}

	pub fn add_f32(&mut self, value: f32) -> Link {
		let f32 = Node::F32(value);

		Link(self.add_node(f32), 0)
	}

	pub fn add_f64(&mut self, value: f64) -> Link {
		let f64 = Node::F64(value);

		Link(self.add_node(f64), 0)
	}

	pub fn add_ref_is_null(&mut self, source: Link) -> Link {
		let ref_is_null = Node::RefIsNull(RefIsNull { source });

		Link(self.add_node(ref_is_null), 0)
	}

	pub fn add_call(
		&mut self,
		function: Link,
		arguments: Vec<Link>,
		results: u16,
		states: u16,
	) -> u32 {
		let call = Node::Call(Call {
			function,
			arguments,
			results,
			states,
		});

		self.add_node(call)
	}

	pub fn add_merge(&mut self, states: Vec<Link>) -> Link {
		let merge = Node::Merge(Merge { states });

		Link(self.add_node(merge), 0)
	}

	pub fn add_integer_unary_operation(
		&mut self,
		source: Link,
		r#type: IntegerType,
		operator: IntegerUnaryOperator,
	) -> Link {
		let unary_operation = Node::IntegerUnaryOperation(IntegerUnaryOperation {
			source,
			r#type,
			operator,
		});

		Link(self.add_node(unary_operation), 0)
	}

	pub fn add_integer_binary_operation(
		&mut self,
		lhs: Link,
		rhs: Link,
		r#type: IntegerType,
		operator: IntegerBinaryOperator,
	) -> Link {
		let binary_operation = Node::IntegerBinaryOperation(IntegerBinaryOperation {
			lhs,
			rhs,
			r#type,
			operator,
		});

		Link(self.add_node(binary_operation), 0)
	}

	pub fn add_integer_compare_operation(
		&mut self,
		lhs: Link,
		rhs: Link,
		r#type: IntegerType,
		operator: IntegerCompareOperator,
	) -> Link {
		let compare_operation = Node::IntegerCompareOperation(IntegerCompareOperation {
			lhs,
			rhs,
			r#type,
			operator,
		});

		Link(self.add_node(compare_operation), 0)
	}

	pub fn add_integer_narrow(&mut self, source: Link) -> Link {
		let narrow = Node::IntegerNarrow(IntegerNarrow { source });

		Link(self.add_node(narrow), 0)
	}

	pub fn add_integer_widen(&mut self, source: Link) -> Link {
		let widen = Node::IntegerWiden(IntegerWiden { source });

		Link(self.add_node(widen), 0)
	}

	pub fn add_integer_extend(&mut self, source: Link, r#type: ExtendType) -> Link {
		let extend = Node::IntegerExtend(IntegerExtend { source, r#type });

		Link(self.add_node(extend), 0)
	}

	pub fn add_integer_convert_to_number(
		&mut self,
		source: Link,
		signed: bool,
		to: NumberType,
		from: IntegerType,
	) -> Link {
		let convert_to_number = Node::IntegerConvertToNumber(IntegerConvertToNumber {
			source,
			signed,
			to,
			from,
		});

		Link(self.add_node(convert_to_number), 0)
	}

	pub fn add_integer_transmute_to_number(&mut self, source: Link, from: IntegerType) -> Link {
		let transmute_to_number =
			Node::IntegerTransmuteToNumber(IntegerTransmuteToNumber { source, from });

		Link(self.add_node(transmute_to_number), 0)
	}

	pub fn add_number_unary_operation(
		&mut self,
		source: Link,
		r#type: NumberType,
		operator: NumberUnaryOperator,
	) -> Link {
		let unary_operation = Node::NumberUnaryOperation(NumberUnaryOperation {
			source,
			r#type,
			operator,
		});

		Link(self.add_node(unary_operation), 0)
	}

	pub fn add_number_binary_operation(
		&mut self,
		lhs: Link,
		rhs: Link,
		r#type: NumberType,
		operator: NumberBinaryOperator,
	) -> Link {
		let binary_operation = Node::NumberBinaryOperation(NumberBinaryOperation {
			lhs,
			rhs,
			r#type,
			operator,
		});

		Link(self.add_node(binary_operation), 0)
	}

	pub fn add_number_compare_operation(
		&mut self,
		lhs: Link,
		rhs: Link,
		r#type: NumberType,
		operator: NumberCompareOperator,
	) -> Link {
		let compare_operation = Node::NumberCompareOperation(NumberCompareOperation {
			lhs,
			rhs,
			r#type,
			operator,
		});

		Link(self.add_node(compare_operation), 0)
	}

	pub fn add_number_narrow(&mut self, source: Link) -> Link {
		let narrow = Node::NumberNarrow(NumberNarrow { source });

		Link(self.add_node(narrow), 0)
	}

	pub fn add_number_widen(&mut self, source: Link) -> Link {
		let widen = Node::NumberWiden(NumberWiden { source });

		Link(self.add_node(widen), 0)
	}

	pub fn add_number_truncate_to_integer(
		&mut self,
		source: Link,
		signed: bool,
		saturate: bool,
		to: IntegerType,
		from: NumberType,
	) -> Link {
		let truncate_to_integer = Node::NumberTruncateToInteger(NumberTruncateToInteger {
			source,
			signed,
			saturate,
			to,
			from,
		});

		Link(self.add_node(truncate_to_integer), 0)
	}

	pub fn add_number_transmute_to_integer(&mut self, source: Link, from: NumberType) -> Link {
		let transmute_to_integer =
			Node::NumberTransmuteToInteger(NumberTransmuteToInteger { source, from });

		Link(self.add_node(transmute_to_integer), 0)
	}

	pub fn add_global_new(&mut self, initializer: Link) -> Link {
		let global_new = Node::GlobalNew(GlobalNew { initializer });

		Link(self.add_node(global_new), 0)
	}

	pub fn add_global_get(&mut self, source: Link) -> Link {
		let global_get = Node::GlobalGet(GlobalGet { source });

		Link(self.add_node(global_get), 0)
	}

	pub fn add_global_set(&mut self, destination: Link, source: Link) -> Link {
		let global_set = Node::GlobalSet(GlobalSet {
			destination,
			source,
		});

		Link(self.add_node(global_set), 0)
	}

	pub fn add_table_new(&mut self, initializer: Link, minimum: u32, maximum: u32) -> Link {
		let table_new = Node::TableNew(TableNew {
			initializer,
			minimum,
			maximum,
		});

		Link(self.add_node(table_new), 0)
	}

	pub fn add_table_get(&mut self, source: Location) -> Link {
		let table_get = Node::TableGet(TableGet { source });

		Link(self.add_node(table_get), 0)
	}

	pub fn add_table_set(&mut self, destination: Location, source: Link) -> Link {
		let table_set = Node::TableSet(TableSet {
			destination,
			source,
		});

		Link(self.add_node(table_set), 0)
	}

	pub fn add_table_size(&mut self, source: Link) -> Link {
		let table_size = Node::TableSize(TableSize { source });

		Link(self.add_node(table_size), 0)
	}

	pub fn add_table_grow(
		&mut self,
		destination: Link,
		initializer: Link,
		size: Link,
	) -> (Link, Link) {
		let table_grow = Node::TableGrow(TableGrow {
			destination,
			initializer,
			size,
		});
		let id = self.add_node(table_grow);

		(Link(id, 0), Link(id, 1))
	}

	pub fn add_table_fill(&mut self, destination: Location, source: Link, size: Link) -> Link {
		let table_fill = Node::TableFill(TableFill {
			destination,
			source,
			size,
		});

		Link(self.add_node(table_fill), 0)
	}

	pub fn add_table_copy(&mut self, destination: Location, source: Location, size: Link) -> Link {
		let table_copy = Node::TableCopy(TableCopy {
			destination,
			source,
			size,
		});

		Link(self.add_node(table_copy), 0)
	}

	pub fn add_table_init(&mut self, destination: Location, source: Location, size: Link) -> Link {
		let table_init = Node::TableInit(TableInit {
			destination,
			source,
			size,
		});

		Link(self.add_node(table_init), 0)
	}

	pub fn add_elements_new(&mut self, content: Vec<Link>) -> Link {
		let elements_new = Node::ElementsNew(ElementsNew { content });

		Link(self.add_node(elements_new), 0)
	}

	pub fn add_elements_drop(&mut self, source: Link) -> Link {
		let elements_drop = Node::ElementsDrop(ElementsDrop { source });

		Link(self.add_node(elements_drop), 0)
	}

	pub fn add_memory_new(&mut self, minimum: u32, maximum: u32) -> Link {
		let memory_new = Node::MemoryNew(MemoryNew { minimum, maximum });

		Link(self.add_node(memory_new), 0)
	}

	pub fn add_memory_load(&mut self, source: Location, r#type: LoadType) -> Link {
		let memory_load = Node::MemoryLoad(MemoryLoad { source, r#type });

		Link(self.add_node(memory_load), 0)
	}

	pub fn add_memory_store(
		&mut self,
		destination: Location,
		source: Link,
		r#type: StoreType,
	) -> Link {
		let memory_store = Node::MemoryStore(MemoryStore {
			destination,
			source,
			r#type,
		});

		Link(self.add_node(memory_store), 0)
	}

	pub fn add_memory_size(&mut self, source: Link) -> Link {
		let memory_size = Node::MemorySize(MemorySize { source });

		Link(self.add_node(memory_size), 0)
	}

	pub fn add_memory_grow(&mut self, destination: Link, size: Link) -> (Link, Link) {
		let memory_grow = Node::MemoryGrow(MemoryGrow { destination, size });
		let id = self.add_node(memory_grow);

		(Link(id, 0), Link(id, 1))
	}

	pub fn add_memory_fill(&mut self, destination: Location, byte: Link, size: Link) -> Link {
		let memory_fill = Node::MemoryFill(MemoryFill {
			destination,
			byte,
			size,
		});

		Link(self.add_node(memory_fill), 0)
	}

	pub fn add_memory_copy(&mut self, destination: Location, source: Location, size: Link) -> Link {
		let memory_copy = Node::MemoryCopy(MemoryCopy {
			destination,
			source,
			size,
		});

		Link(self.add_node(memory_copy), 0)
	}

	pub fn add_memory_init(&mut self, destination: Location, source: Location, size: Link) -> Link {
		let memory_init = Node::MemoryInit(MemoryInit {
			destination,
			source,
			size,
		});

		Link(self.add_node(memory_init), 0)
	}

	pub fn add_data_new(&mut self, content: Arc<[u8]>) -> Link {
		let data_new = Node::DataNew(DataNew { content });

		Link(self.add_node(data_new), 0)
	}

	pub fn add_data_drop(&mut self, source: Link) -> Link {
		let data_drop = Node::DataDrop(DataDrop { source });

		Link(self.add_node(data_drop), 0)
	}
}

impl Default for DataFlowGraph {
	fn default() -> Self {
		Self::new()
	}
}
