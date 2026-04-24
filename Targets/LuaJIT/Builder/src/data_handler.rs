use alloc::sync::Arc;

use hashbrown::HashMap;

use ir_graph::{Link, operation};
use luajit_tree::expression::{
	Aggregate, BooleanToInteger, Call, Expression, Extract, GlobalGet, GlobalNew,
	IntegerBinaryOperation, IntegerCompareOperation, IntegerConvertToNumber, IntegerExtend,
	IntegerNarrow, IntegerTransmuteToNumber, IntegerUnaryOperation, IntegerWiden, Local, Location,
	MemoryGrow, MemoryLoad, MemorySize, Name, NumberBinaryOperation, NumberCompareOperation,
	NumberNarrow, NumberTransmuteToInteger, NumberTruncateToInteger, NumberUnaryOperation,
	NumberWiden, RefIsNull, RuntimeCall, TableGet, TableGrow, TableNew, TableSize,
};
use web_assembly_foreign::Import as WasmImport;

use super::policy::PHYSICAL_REGISTERS;

type ScopedLink = (Link, usize);
type ScopedId = (u32, usize);

pub struct DataHandler {
	registers: HashMap<ScopedLink, u32>,
	expressions: HashMap<ScopedId, Expression>,
}

fn reg_to_local(reg: u32) -> Local {
	if reg < PHYSICAL_REGISTERS {
		Local::Fast {
			name: Name { id: reg },
		}
	} else {
		let offset = u16::try_from(reg - PHYSICAL_REGISTERS).unwrap();

		Local::Slow { offset }
	}
}

fn load_runtime_call(name: &'static str, arguments: Vec<Expression>) -> Expression {
	let expression = RuntimeCall { name, arguments };

	Expression::RuntimeCall(expression.into())
}

pub fn load_wasm_import(node: &WasmImport) -> Expression {
	let arguments = vec![
		Expression::String(Arc::clone(&node.namespace)),
		Expression::String(Arc::clone(&node.identifier)),
	];

	load_runtime_call("import", arguments)
}

pub fn load_turing_ask() -> Expression {
	load_runtime_call("turing_ask", Vec::new())
}

pub fn load_mutable_get(source: Expression) -> Expression {
	Expression::GlobalGet(GlobalGet { source }.into())
}

pub fn load_table_get(source: Location) -> Expression {
	Expression::TableGet(TableGet { source }.into())
}

pub fn load_table_size(source: Expression) -> Expression {
	Expression::TableSize(TableSize { source }.into())
}

pub fn load_table_grow(
	destination: Expression,
	initializer: Expression,
	size: Expression,
) -> Expression {
	Expression::TableGrow(
		TableGrow {
			destination,
			initializer,
			size,
		}
		.into(),
	)
}

pub fn load_memory_load(source: Location, kind: operation::LoadType) -> Expression {
	Expression::MemoryLoad(MemoryLoad { source, kind }.into())
}

pub fn load_memory_size(source: Expression) -> Expression {
	Expression::MemorySize(MemorySize { source }.into())
}

pub fn load_memory_grow(destination: Expression, size: Expression) -> Expression {
	Expression::MemoryGrow(MemoryGrow { destination, size }.into())
}

impl DataHandler {
	pub fn new() -> Self {
		Self {
			registers: HashMap::new(),
			expressions: HashMap::new(),
		}
	}

	pub const fn registers_mut(&mut self) -> &mut HashMap<ScopedLink, u32> {
		&mut self.registers
	}

	pub fn store_expression(&mut self, scope: usize, id: u32, source: Expression) {
		if self.expressions.try_insert((id, scope), source).is_err() {
			unreachable!("expression already stored for id {id}")
		}
	}

	pub fn take_expression(&mut self, id: u32, scope: usize) -> Option<Expression> {
		self.expressions.remove(&(id, scope))
	}

	pub fn get_local(&self, scope: usize, link: Link) -> Option<Local> {
		let reg = self.registers.get(&(link, scope)).copied()?;

		Some(reg_to_local(reg))
	}

	pub fn load(&mut self, scope: usize, link: Link) -> Expression {
		self.get_local(scope, link).map_or_else(
			|| self.take_expression(link.0, scope).unwrap(),
			Expression::Local,
		)
	}

	pub fn load_all(&mut self, scope: usize, sources: &[Link]) -> Vec<Expression> {
		sources.iter().map(|&link| self.load(scope, link)).collect()
	}

	pub fn load_result_locals(&self, scope: usize, id: u32, ports: u16) -> Vec<Local> {
		(0..ports)
			.map(|port| {
				let key = (Link(id, port), scope);

				reg_to_local(self.registers[&key])
			})
			.collect()
	}

	pub fn load_local_moves(
		&self,
		destination_scope: usize,
		id: u32,
		sources: &[Link],
		source_scope: usize,
	) -> Vec<(Local, Local)> {
		sources
			.iter()
			.enumerate()
			.map(|(port, &source_link)| {
				let port = u16::try_from(port).unwrap();
				let destination_key = (Link(id, port), destination_scope);
				let source_key = (source_link, source_scope);
				let destination = reg_to_local(self.registers[&destination_key]);
				let source_local = reg_to_local(self.registers[&source_key]);

				(destination, source_local)
			})
			.collect()
	}

	pub fn load_call(&mut self, scope: usize, node: &operation::Apply) -> Expression {
		let function = self.load(scope, node.function);
		let arguments = self.load_all(scope, &node.arguments);

		let call = Call {
			function,
			arguments,
		};

		Expression::Call(call.into())
	}

	pub fn load_ref_is_null(&mut self, scope: usize, node: operation::RefIsNull) -> Expression {
		let expression = RefIsNull {
			source: self.load(scope, node.source),
		};

		let boolean = BooleanToInteger {
			source: Expression::RefIsNull(expression.into()),
		};

		Expression::BooleanToInteger(boolean.into())
	}

	pub fn load_integer_unary_operation(
		&mut self,
		scope: usize,
		node: operation::integer::UnaryOperation,
	) -> Expression {
		let expression = IntegerUnaryOperation {
			source: self.load(scope, node.source),
			kind: node.kind,
			operator: node.operator,
		};

		Expression::IntegerUnaryOperation(expression.into())
	}

	pub fn load_integer_binary_operation(
		&mut self,
		scope: usize,
		node: operation::integer::BinaryOperation,
	) -> Expression {
		let expression = IntegerBinaryOperation {
			lhs: self.load(scope, node.lhs),
			rhs: self.load(scope, node.rhs),
			kind: node.kind,
			operator: node.operator,
		};

		Expression::IntegerBinaryOperation(expression.into())
	}

	pub fn load_integer_compare_operation(
		&mut self,
		scope: usize,
		node: operation::integer::CompareOperation,
	) -> Expression {
		let expression = IntegerCompareOperation {
			lhs: self.load(scope, node.lhs),
			rhs: self.load(scope, node.rhs),
			kind: node.kind,
			operator: node.operator,
		};

		let boolean = BooleanToInteger {
			source: Expression::IntegerCompareOperation(expression.into()),
		};

		Expression::BooleanToInteger(boolean.into())
	}

	pub fn load_integer_narrow(
		&mut self,
		scope: usize,
		node: operation::IntegerNarrow,
	) -> Expression {
		let expression = IntegerNarrow {
			source: self.load(scope, node.source),
		};

		Expression::IntegerNarrow(expression.into())
	}

	pub fn load_integer_widen(
		&mut self,
		scope: usize,
		node: operation::IntegerWiden,
	) -> Expression {
		let expression = IntegerWiden {
			source: self.load(scope, node.source),
		};

		Expression::IntegerWiden(expression.into())
	}

	pub fn load_integer_sign_extend(
		&mut self,
		scope: usize,
		node: operation::IntegerSignExtend,
	) -> Expression {
		let expression = IntegerExtend {
			source: self.load(scope, node.source),
			kind: node.kind,
		};

		Expression::IntegerExtend(expression.into())
	}

	pub fn load_integer_convert_to_number(
		&mut self,
		scope: usize,
		node: operation::IntegerConvertToNumber,
	) -> Expression {
		let expression = IntegerConvertToNumber {
			source: self.load(scope, node.source),
			signed: node.signed,
			to: node.to,
			from: node.from,
		};

		Expression::IntegerConvertToNumber(expression.into())
	}

	pub fn load_integer_transmute_to_number(
		&mut self,
		scope: usize,
		node: operation::IntegerTransmuteToNumber,
	) -> Expression {
		let expression = IntegerTransmuteToNumber {
			source: self.load(scope, node.source),
			from: node.from,
		};

		Expression::IntegerTransmuteToNumber(expression.into())
	}

	pub fn load_number_unary_operation(
		&mut self,
		scope: usize,
		node: operation::number::UnaryOperation,
	) -> Expression {
		let expression = NumberUnaryOperation {
			source: self.load(scope, node.source),
			kind: node.kind,
			operator: node.operator,
		};

		Expression::NumberUnaryOperation(expression.into())
	}

	pub fn load_number_binary_operation(
		&mut self,
		scope: usize,
		node: operation::number::BinaryOperation,
	) -> Expression {
		let expression = NumberBinaryOperation {
			lhs: self.load(scope, node.lhs),
			rhs: self.load(scope, node.rhs),
			kind: node.kind,
			operator: node.operator,
		};

		Expression::NumberBinaryOperation(expression.into())
	}

	pub fn load_number_compare_operation(
		&mut self,
		scope: usize,
		node: operation::number::CompareOperation,
	) -> Expression {
		let expression = NumberCompareOperation {
			lhs: self.load(scope, node.lhs),
			rhs: self.load(scope, node.rhs),
			kind: node.kind,
			operator: node.operator,
		};

		let boolean = BooleanToInteger {
			source: Expression::NumberCompareOperation(expression.into()),
		};

		Expression::BooleanToInteger(boolean.into())
	}

	pub fn load_number_narrow(
		&mut self,
		scope: usize,
		node: operation::NumberNarrow,
	) -> Expression {
		let expression = NumberNarrow {
			source: self.load(scope, node.source),
		};

		Expression::NumberNarrow(expression.into())
	}

	pub fn load_number_widen(&mut self, scope: usize, node: operation::NumberWiden) -> Expression {
		let expression = NumberWiden {
			source: self.load(scope, node.source),
		};

		Expression::NumberWiden(expression.into())
	}

	pub fn load_number_truncate_to_integer(
		&mut self,
		scope: usize,
		node: operation::NumberTruncateToInteger,
	) -> Expression {
		let expression = NumberTruncateToInteger {
			source: self.load(scope, node.source),
			signed: node.signed,
			saturate: node.saturate,
			to: node.to,
			from: node.from,
		};

		Expression::NumberTruncateToInteger(expression.into())
	}

	pub fn load_number_transmute_to_integer(
		&mut self,
		scope: usize,
		node: operation::NumberTransmuteToInteger,
	) -> Expression {
		let expression = NumberTransmuteToInteger {
			source: self.load(scope, node.source),
			from: node.from,
		};

		Expression::NumberTransmuteToInteger(expression.into())
	}

	pub fn load_mutable_new(&mut self, scope: usize, node: operation::MutableNew) -> Expression {
		let expression = GlobalNew {
			initializer: self.load(scope, node.initializer),
		};

		Expression::GlobalNew(expression.into())
	}

	pub fn load_aggregate(&mut self, scope: usize, node: &operation::Aggregate) -> Expression {
		let fields = node
			.fields
			.iter()
			.map(|&link| self.load(scope, link))
			.collect();
		let expression = Aggregate { fields };

		Expression::Aggregate(expression.into())
	}

	pub fn load_extract(&mut self, scope: usize, node: &operation::Extract) -> Expression {
		let expression = Extract {
			source: self.load(scope, node.source),
			index: node.index,
		};

		Expression::Extract(expression.into())
	}

	pub fn load_table_new(&mut self, scope: usize, node: &operation::TableNew) -> Expression {
		let initializer = node
			.initializer
			.iter()
			.map(|&(link, offset)| (self.load(scope, link), offset))
			.collect();

		let expression = TableNew {
			initializer,
			minimum: node.minimum,
			maximum: node.maximum,
		};

		Expression::TableNew(expression.into())
	}
}
