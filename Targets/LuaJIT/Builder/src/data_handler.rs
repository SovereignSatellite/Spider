use alloc::vec::Vec;
use data_flow_graph::{Link, control, simple};
use hashbrown::HashMap;
use luajit_tree::{
	expression::{
		BooleanToInteger, Call, Expression, Function, GlobalGet, GlobalNew, Import,
		IntegerBinaryOperation, IntegerCompareOperation, IntegerConvertToNumber, IntegerExtend,
		IntegerNarrow, IntegerTransmuteToNumber, IntegerUnaryOperation, IntegerWiden, Local,
		Location, MemoryGrow, MemoryLoad, MemorySize, Name, NumberBinaryOperation,
		NumberCompareOperation, NumberNarrow, NumberTransmuteToInteger, NumberTruncateToInteger,
		NumberUnaryOperation, NumberWiden, RefIsNull, Scoped, TableGet, TableGrow, TableNew,
		TableSize,
	},
	statement::{Export, Sequence},
};

use crate::local_allocator::Declarations;

pub struct DataHandler {
	declarations: HashMap<u32, Declarations>,
	assignments: HashMap<Link, Local>,

	expressions: HashMap<u32, Expression>,
}

impl DataHandler {
	pub fn new() -> Self {
		Self {
			declarations: HashMap::new(),
			assignments: HashMap::new(),

			expressions: HashMap::new(),
		}
	}

	pub const fn locals_mut(
		&mut self,
	) -> (&mut HashMap<u32, Declarations>, &mut HashMap<Link, Local>) {
		(&mut self.declarations, &mut self.assignments)
	}

	pub fn store_expression(&mut self, id: u32, source: Expression) {
		self.expressions
			.try_insert(id, source)
			.unwrap_or_else(|_| panic!("expression should set only once"));
	}

	pub fn get_stack_size(&self, id: u32) -> u16 {
		self.declarations[&id].stack
	}

	pub fn get_local(&self, link: Link) -> Option<Local> {
		self.assignments.get(&link).copied()
	}

	pub fn load(&mut self, link: Link) -> Expression {
		self.get_local(link).map_or_else(
			|| {
				assert_eq!(link.1, 0, "expression should load from first port");

				self.expressions.remove(&link.0).unwrap()
			},
			Expression::Local,
		)
	}

	pub fn load_all(&mut self, sources: &[Link]) -> Vec<Expression> {
		sources.iter().map(|&link| self.load(link)).collect()
	}

	pub fn load_name_assignments(&self, id: u32, ports: core::ops::Range<u16>) -> Vec<Name> {
		let names = ports.map(|port| Link(id, port));

		names
			.map(|name| self.assignments[&name].into_name())
			.collect()
	}

	pub fn load_local_assignments(&self, id: u32, ports: core::ops::Range<u16>) -> Vec<Local> {
		let names = ports.map(|port| Link(id, port));

		names.map(|name| self.assignments[&name]).collect()
	}

	pub fn load_assign_all(&self, id: u32, sources: &[Link]) -> Vec<(Local, Local)> {
		let destinations = (0..)
			.map(|port| Link(id, port))
			.map(|link| self.assignments[&link]);

		let sources = sources.iter().map(|&link| self.assignments[&link]);

		destinations.zip(sources).collect()
	}

	pub fn load_dependencies(
		&mut self,
		id: u32,
		ports: core::ops::Range<u16>,
		dependencies: &[Link],
	) -> Vec<(Name, Expression)> {
		let names = ports.map(|port| Link(id, port));
		let iter = names.zip(dependencies).map(|(name, &dependency)| {
			let name = self.assignments[&name].into_name();
			let dependency = self.load(dependency);

			(name, dependency)
		});

		iter.collect()
	}

	pub fn load_declarations(&self, id: u32) -> Vec<Name> {
		let locals = self.declarations[&id].locals.clone();

		locals.map(|id| Name { id }).collect()
	}

	pub fn load_returns(
		&mut self,
		results: &[Link],
		function_type: &control::FunctionType,
	) -> Vec<Expression> {
		let returns = results.iter().map(|&name| self.load(name));
		let len = function_type.results.len();

		returns.take(len).collect()
	}

	pub fn load_scoped(
		dependencies: Vec<(Name, Expression)>,
		arguments: Vec<Name>,
		locals: Vec<Name>,
		stack: u16,
		code: Sequence,
		returns: Vec<Expression>,
	) -> Expression {
		let function = Function {
			arguments,
			locals,
			stack,
			code,
			returns,
		};

		if dependencies.is_empty() {
			Expression::Function(function.into())
		} else {
			let scoped = Scoped {
				dependencies,
				function,
			};

			Expression::Scoped(scoped.into())
		}
	}

	pub fn load_import(&mut self, node: &control::Import) -> Expression {
		let expression = Import {
			environment: self.load(node.environment),
			namespace: node.namespace.clone(),
			identifier: node.identifier.clone(),
		};

		Expression::Import(expression.into())
	}

	fn load_export(&mut self, node: &control::Export) -> Export {
		Export {
			identifier: node.identifier.clone(),
			source: self.load(node.reference),
		}
	}

	pub fn load_exports(&mut self, nodes: &[control::Export]) -> Vec<Export> {
		nodes
			.iter()
			.map(|export| self.load_export(export))
			.collect()
	}

	pub fn load_call(&mut self, node: &simple::Apply) -> Expression {
		let end = node.arguments.len() - usize::from(node.states);
		let call = Call {
			function: self.load(node.function),
			arguments: self.load_all(&node.arguments[..end]),
		};

		Expression::Call(call.into())
	}

	pub fn load_ref_is_null(&mut self, node: simple::RefIsNull) -> Expression {
		let expression = RefIsNull {
			source: self.load(node.source),
		};

		let boolean = BooleanToInteger {
			source: Expression::RefIsNull(expression.into()),
		};

		Expression::BooleanToInteger(boolean.into())
	}

	pub fn load_integer_unary_operation(
		&mut self,
		node: simple::IntegerUnaryOperation,
	) -> Expression {
		let expression = IntegerUnaryOperation {
			source: self.load(node.source),
			r#type: node.r#type,
			operator: node.operator,
		};

		Expression::IntegerUnaryOperation(expression.into())
	}

	pub fn load_integer_binary_operation(
		&mut self,
		node: simple::IntegerBinaryOperation,
	) -> Expression {
		let expression = IntegerBinaryOperation {
			lhs: self.load(node.lhs),
			rhs: self.load(node.rhs),
			r#type: node.r#type,
			operator: node.operator,
		};

		Expression::IntegerBinaryOperation(expression.into())
	}

	pub fn load_integer_compare_operation(
		&mut self,
		node: simple::IntegerCompareOperation,
	) -> Expression {
		let expression = IntegerCompareOperation {
			lhs: self.load(node.lhs),
			rhs: self.load(node.rhs),
			r#type: node.r#type,
			operator: node.operator,
		};

		let boolean = BooleanToInteger {
			source: Expression::IntegerCompareOperation(expression.into()),
		};

		Expression::BooleanToInteger(boolean.into())
	}

	pub fn load_integer_narrow(&mut self, node: simple::IntegerNarrow) -> Expression {
		let expression = IntegerNarrow {
			source: self.load(node.source),
		};

		Expression::IntegerNarrow(expression.into())
	}

	pub fn load_integer_widen(&mut self, node: simple::IntegerWiden) -> Expression {
		let expression = IntegerWiden {
			source: self.load(node.source),
		};

		Expression::IntegerWiden(expression.into())
	}

	pub fn load_integer_extend(&mut self, node: simple::IntegerExtend) -> Expression {
		let expression = IntegerExtend {
			source: self.load(node.source),
			r#type: node.r#type,
		};

		Expression::IntegerExtend(expression.into())
	}

	pub fn load_integer_convert_to_number(
		&mut self,
		node: simple::IntegerConvertToNumber,
	) -> Expression {
		let expression = IntegerConvertToNumber {
			source: self.load(node.source),
			signed: node.signed,
			to: node.to,
			from: node.from,
		};

		Expression::IntegerConvertToNumber(expression.into())
	}

	pub fn load_integer_transmute_to_number(
		&mut self,
		node: simple::IntegerTransmuteToNumber,
	) -> Expression {
		let expression = IntegerTransmuteToNumber {
			source: self.load(node.source),
			from: node.from,
		};

		Expression::IntegerTransmuteToNumber(expression.into())
	}

	pub fn load_number_unary_operation(
		&mut self,
		node: simple::NumberUnaryOperation,
	) -> Expression {
		let expression = NumberUnaryOperation {
			source: self.load(node.source),
			r#type: node.r#type,
			operator: node.operator,
		};

		Expression::NumberUnaryOperation(expression.into())
	}

	pub fn load_number_binary_operation(
		&mut self,
		node: simple::NumberBinaryOperation,
	) -> Expression {
		let expression = NumberBinaryOperation {
			lhs: self.load(node.lhs),
			rhs: self.load(node.rhs),
			r#type: node.r#type,
			operator: node.operator,
		};

		Expression::NumberBinaryOperation(expression.into())
	}

	pub fn load_number_compare_operation(
		&mut self,
		node: simple::NumberCompareOperation,
	) -> Expression {
		let expression = NumberCompareOperation {
			lhs: self.load(node.lhs),
			rhs: self.load(node.rhs),
			r#type: node.r#type,
			operator: node.operator,
		};

		let boolean = BooleanToInteger {
			source: Expression::NumberCompareOperation(expression.into()),
		};

		Expression::BooleanToInteger(boolean.into())
	}

	pub fn load_number_narrow(&mut self, node: simple::NumberNarrow) -> Expression {
		let expression = NumberNarrow {
			source: self.load(node.source),
		};

		Expression::NumberNarrow(expression.into())
	}

	pub fn load_number_widen(&mut self, node: simple::NumberWiden) -> Expression {
		let expression = NumberWiden {
			source: self.load(node.source),
		};

		Expression::NumberWiden(expression.into())
	}

	pub fn load_number_truncate_to_integer(
		&mut self,
		node: simple::NumberTruncateToInteger,
	) -> Expression {
		let expression = NumberTruncateToInteger {
			source: self.load(node.source),
			signed: node.signed,
			saturate: node.saturate,
			to: node.to,
			from: node.from,
		};

		Expression::NumberTruncateToInteger(expression.into())
	}

	pub fn load_number_transmute_to_integer(
		&mut self,
		node: simple::NumberTransmuteToInteger,
	) -> Expression {
		let expression = NumberTransmuteToInteger {
			source: self.load(node.source),
			from: node.from,
		};

		Expression::NumberTransmuteToInteger(expression.into())
	}

	pub fn load_global_new(&mut self, node: simple::GlobalNew) -> Expression {
		let expression = GlobalNew {
			initializer: self.load(node.initializer),
		};

		Expression::GlobalNew(expression.into())
	}

	pub fn load_global_get(&mut self, node: simple::GlobalGet) -> Expression {
		let expression = GlobalGet {
			source: self.load(node.source),
		};

		Expression::GlobalGet(expression.into())
	}

	pub fn load_location(&mut self, location: simple::Location) -> Location {
		Location {
			reference: self.load(location.reference),
			offset: self.load(location.offset),
		}
	}

	pub fn load_table_new(&mut self, node: &simple::TableNew) -> Expression {
		let initializer = node
			.initializer
			.iter()
			.map(|&(link, offset)| (self.load(link), offset))
			.collect();

		let expression = TableNew {
			initializer,
			minimum: node.minimum,
			maximum: node.maximum,
		};

		Expression::TableNew(expression.into())
	}

	pub fn load_table_get(&mut self, node: simple::TableGet) -> Expression {
		let expression = TableGet {
			source: self.load_location(node.source),
		};

		Expression::TableGet(expression.into())
	}

	pub fn load_table_size(&mut self, node: simple::TableSize) -> Expression {
		let expression = TableSize {
			source: self.load(node.source),
		};

		Expression::TableSize(expression.into())
	}

	pub fn load_table_grow(&mut self, node: simple::TableGrow) -> Expression {
		let expression = TableGrow {
			destination: self.load(node.destination),
			initializer: self.load(node.initializer),
			size: self.load(node.size),
		};

		Expression::TableGrow(expression.into())
	}

	pub fn load_memory_load(&mut self, node: simple::MemoryLoad) -> Expression {
		let expression = MemoryLoad {
			source: self.load_location(node.source),
			r#type: node.r#type,
		};

		Expression::MemoryLoad(expression.into())
	}

	pub fn load_memory_size(&mut self, node: simple::MemorySize) -> Expression {
		let expression = MemorySize {
			source: self.load(node.source),
		};

		Expression::MemorySize(expression.into())
	}

	pub fn load_memory_grow(&mut self, node: simple::MemoryGrow) -> Expression {
		let expression = MemoryGrow {
			destination: self.load(node.destination),
			size: self.load(node.size),
		};

		Expression::MemoryGrow(expression.into())
	}
}
