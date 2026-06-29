use alloc::sync::Arc;

use hashbrown::HashMap;

use ir_allocator::DEFERRED;
use ir_graph::{Link, operation};
use luajit_tree::expression::{
	Aggregate, BooleanToInteger, Expression, Extract, GlobalGet, GlobalNew, IntegerBinaryOperation,
	IntegerCompareOperation, IntegerConvertToNumber, IntegerExtend, IntegerNarrow,
	IntegerTransmuteToNumber, IntegerUnaryOperation, IntegerWiden, Local, Location, MemoryGrow,
	MemoryLoad, MemorySize, Name, NumberBinaryOperation, NumberCompareOperation, NumberNarrow,
	NumberTransmuteToInteger, NumberTruncateToInteger, NumberUnaryOperation, NumberWiden,
	RefIsNull, RuntimeCall, TableGet, TableGrow, TableNew, TableSize,
};

use super::policy::PHYSICAL_REGISTERS;

fn register_to_local(register: u32) -> Local {
	if register < PHYSICAL_REGISTERS {
		Local::Fast {
			name: Name { id: register },
		}
	} else {
		let offset = u16::try_from(register - PHYSICAL_REGISTERS).unwrap();

		Local::Slow { offset }
	}
}

fn build_runtime_call(name: &'static str, arguments: Vec<Expression>) -> Expression {
	let expression = RuntimeCall { name, arguments };

	Expression::RuntimeCall(expression.into())
}

pub fn build_import(node: &operation::Import) -> Expression {
	let arguments = vec![
		Expression::String(Arc::clone(&node.namespace)),
		Expression::String(Arc::clone(&node.identifier)),
	];

	build_runtime_call("import", arguments)
}

pub struct DataHandler {
	arena: ir_allocator::Arena,
	deferred_expressions: HashMap<(u32, u32), Expression>,
}

impl DataHandler {
	#[must_use]
	pub fn new() -> Self {
		Self {
			arena: ir_allocator::Arena::new(),
			deferred_expressions: HashMap::new(),
		}
	}

	pub fn install(&mut self, arena: ir_allocator::Arena) {
		self.arena = arena;
	}

	#[must_use]
	pub fn node_count(&self, region: u32) -> usize {
		self.arena.node_count(region)
	}

	#[must_use]
	pub fn is_deferred(&self, region: u32, id: u32) -> bool {
		self.arena.register(region, Link(id, 0)) == DEFERRED
	}

	pub fn store(&mut self, region: u32, id: u32, expression: Expression) {
		self.deferred_expressions.insert((region, id), expression);
	}

	#[must_use]
	pub fn local_of(&self, region: u32, link: Link) -> Local {
		register_to_local(self.arena.register(region, link))
	}

	#[must_use]
	pub fn port_locals(&self, region: u32, id: u32, count: u16) -> Vec<Local> {
		(0..count)
			.map(|port| self.local_of(region, Link(id, port)))
			.collect()
	}

	pub fn load(&mut self, region: u32, link: Link) -> Expression {
		let register = self.arena.register(region, link);

		if register == DEFERRED {
			return self
				.deferred_expressions
				.remove(&(region, link.0))
				.expect("a deferred port must have a stored expression");
		}

		Expression::Local(register_to_local(register))
	}

	pub fn load_all(&mut self, region: u32, sources: &[Link]) -> Vec<Expression> {
		sources
			.iter()
			.map(|&link| self.load(region, link))
			.collect()
	}

	pub fn load_location(&mut self, region: u32, location: operation::Location) -> Location {
		Location {
			reference: self.load(region, location.reference),
			offset: self.load(region, location.offset),
		}
	}

	pub fn build_ref_is_null(&mut self, region: u32, node: operation::RefIsNull) -> Expression {
		let expression = RefIsNull {
			source: self.load(region, node.source),
		};
		let boolean = BooleanToInteger {
			source: Expression::RefIsNull(expression.into()),
		};

		Expression::BooleanToInteger(boolean.into())
	}

	pub fn build_integer_unary_operation(
		&mut self,
		region: u32,
		node: operation::integer::UnaryOperation,
	) -> Expression {
		let expression = IntegerUnaryOperation {
			source: self.load(region, node.source),
			kind: node.kind,
			operator: node.operator,
		};

		Expression::IntegerUnaryOperation(expression.into())
	}

	pub fn build_integer_binary_operation(
		&mut self,
		region: u32,
		node: operation::integer::BinaryOperation,
	) -> Expression {
		let expression = IntegerBinaryOperation {
			lhs: self.load(region, node.lhs),
			rhs: self.load(region, node.rhs),
			kind: node.kind,
			operator: node.operator,
		};

		Expression::IntegerBinaryOperation(expression.into())
	}

	pub fn build_integer_compare_operation(
		&mut self,
		region: u32,
		node: operation::integer::CompareOperation,
	) -> Expression {
		let expression = IntegerCompareOperation {
			lhs: self.load(region, node.lhs),
			rhs: self.load(region, node.rhs),
			kind: node.kind,
			operator: node.operator,
		};
		let boolean = BooleanToInteger {
			source: Expression::IntegerCompareOperation(expression.into()),
		};

		Expression::BooleanToInteger(boolean.into())
	}

	pub fn build_integer_narrow(
		&mut self,
		region: u32,
		node: operation::IntegerNarrow,
	) -> Expression {
		let expression = IntegerNarrow {
			source: self.load(region, node.source),
		};

		Expression::IntegerNarrow(expression.into())
	}

	pub fn build_integer_widen(
		&mut self,
		region: u32,
		node: operation::IntegerWiden,
	) -> Expression {
		let expression = IntegerWiden {
			source: self.load(region, node.source),
		};

		Expression::IntegerWiden(expression.into())
	}

	pub fn build_integer_sign_extend(
		&mut self,
		region: u32,
		node: operation::IntegerSignExtend,
	) -> Expression {
		let expression = IntegerExtend {
			source: self.load(region, node.source),
			kind: node.kind,
		};

		Expression::IntegerExtend(expression.into())
	}

	pub fn build_integer_convert_to_number(
		&mut self,
		region: u32,
		node: operation::IntegerConvertToNumber,
	) -> Expression {
		let expression = IntegerConvertToNumber {
			source: self.load(region, node.source),
			is_signed: node.is_signed,
			to: node.to,
			from: node.from,
		};

		Expression::IntegerConvertToNumber(expression.into())
	}

	pub fn build_integer_transmute_to_number(
		&mut self,
		region: u32,
		node: operation::IntegerTransmuteToNumber,
	) -> Expression {
		let expression = IntegerTransmuteToNumber {
			source: self.load(region, node.source),
			from: node.from,
		};

		Expression::IntegerTransmuteToNumber(expression.into())
	}

	pub fn build_number_unary_operation(
		&mut self,
		region: u32,
		node: operation::number::UnaryOperation,
	) -> Expression {
		let expression = NumberUnaryOperation {
			source: self.load(region, node.source),
			kind: node.kind,
			operator: node.operator,
		};

		Expression::NumberUnaryOperation(expression.into())
	}

	pub fn build_number_binary_operation(
		&mut self,
		region: u32,
		node: operation::number::BinaryOperation,
	) -> Expression {
		let expression = NumberBinaryOperation {
			lhs: self.load(region, node.lhs),
			rhs: self.load(region, node.rhs),
			kind: node.kind,
			operator: node.operator,
		};

		Expression::NumberBinaryOperation(expression.into())
	}

	pub fn build_number_compare_operation(
		&mut self,
		region: u32,
		node: operation::number::CompareOperation,
	) -> Expression {
		let expression = NumberCompareOperation {
			lhs: self.load(region, node.lhs),
			rhs: self.load(region, node.rhs),
			kind: node.kind,
			operator: node.operator,
		};
		let boolean = BooleanToInteger {
			source: Expression::NumberCompareOperation(expression.into()),
		};

		Expression::BooleanToInteger(boolean.into())
	}

	pub fn build_number_narrow(
		&mut self,
		region: u32,
		node: operation::NumberNarrow,
	) -> Expression {
		let expression = NumberNarrow {
			source: self.load(region, node.source),
		};

		Expression::NumberNarrow(expression.into())
	}

	pub fn build_number_widen(&mut self, region: u32, node: operation::NumberWiden) -> Expression {
		let expression = NumberWiden {
			source: self.load(region, node.source),
		};

		Expression::NumberWiden(expression.into())
	}

	pub fn build_number_truncate_to_integer(
		&mut self,
		region: u32,
		node: operation::NumberTruncateToInteger,
	) -> Expression {
		let expression = NumberTruncateToInteger {
			source: self.load(region, node.source),
			is_signed: node.is_signed,
			is_saturating: node.is_saturating,
			to: node.to,
			from: node.from,
		};

		Expression::NumberTruncateToInteger(expression.into())
	}

	pub fn build_number_transmute_to_integer(
		&mut self,
		region: u32,
		node: operation::NumberTransmuteToInteger,
	) -> Expression {
		let expression = NumberTransmuteToInteger {
			source: self.load(region, node.source),
			from: node.from,
		};

		Expression::NumberTransmuteToInteger(expression.into())
	}

	pub fn build_mutable_new(&mut self, region: u32, node: operation::MutableNew) -> Expression {
		let expression = GlobalNew {
			initializer: self.load(region, node.initializer),
		};

		Expression::GlobalNew(expression.into())
	}

	pub fn build_mutable_get(&mut self, region: u32, node: operation::MutableGet) -> Expression {
		let expression = GlobalGet {
			source: self.load(region, node.source),
		};

		Expression::GlobalGet(expression.into())
	}

	pub fn build_aggregate(&mut self, region: u32, node: &operation::Aggregate) -> Expression {
		let fields = node
			.fields
			.iter()
			.map(|&link| self.load(region, link))
			.collect();
		let expression = Aggregate { fields };

		Expression::Aggregate(expression.into())
	}

	pub fn build_extract(&mut self, region: u32, node: operation::Extract) -> Expression {
		let expression = Extract {
			source: self.load(region, node.source),
			index: node.index,
		};

		Expression::Extract(expression.into())
	}

	pub fn build_table_new(&mut self, region: u32, node: &operation::TableNew) -> Expression {
		let initializer = node
			.initializer
			.iter()
			.map(|&(link, offset)| (self.load(region, link), offset))
			.collect();
		let expression = TableNew {
			initializer,
			minimum: node.minimum,
			maximum: node.maximum,
		};

		Expression::TableNew(expression.into())
	}

	pub fn build_table_get(&mut self, region: u32, node: operation::TableGet) -> Expression {
		let source = self.load_location(region, node.source);

		Expression::TableGet(TableGet { source }.into())
	}

	pub fn build_table_size(&mut self, region: u32, node: operation::TableSize) -> Expression {
		let source = self.load(region, node.source);

		Expression::TableSize(TableSize { source }.into())
	}

	pub fn build_table_grow(&mut self, region: u32, node: operation::TableGrow) -> Expression {
		let expression = TableGrow {
			destination: self.load(region, node.destination),
			initializer: self.load(region, node.initializer),
			size: self.load(region, node.size),
		};

		Expression::TableGrow(expression.into())
	}

	pub fn build_memory_load(&mut self, region: u32, node: operation::MemoryLoad) -> Expression {
		let source = self.load_location(region, node.source);

		Expression::MemoryLoad(
			MemoryLoad {
				source,
				kind: node.kind,
			}
			.into(),
		)
	}

	pub fn build_memory_size(&mut self, region: u32, node: operation::MemorySize) -> Expression {
		let source = self.load(region, node.source);

		Expression::MemorySize(MemorySize { source }.into())
	}

	pub fn build_memory_grow(&mut self, region: u32, node: operation::MemoryGrow) -> Expression {
		let expression = MemoryGrow {
			destination: self.load(region, node.destination),
			size: self.load(region, node.size),
		};

		Expression::MemoryGrow(expression.into())
	}
}

impl Default for DataHandler {
	fn default() -> Self {
		Self::new()
	}
}
