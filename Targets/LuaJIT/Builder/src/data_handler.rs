use alloc::sync::Arc;

use hashbrown::HashMap;

use ir_graph::{Link, control, simple};
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
	statement::Export,
};

type ScopedLink = (Link, usize);
type ScopedId = (u32, usize);

pub struct DataHandler {
	stack_sizes: HashMap<usize, u16>,
	assignments: HashMap<ScopedLink, Local>,
	expressions: HashMap<ScopedId, Expression>,

	current_scope: usize,
}

impl DataHandler {
	pub fn new() -> Self {
		Self {
			stack_sizes: HashMap::new(),
			assignments: HashMap::new(),
			expressions: HashMap::new(),

			current_scope: 0,
		}
	}

	pub const fn scope(&self) -> usize {
		self.current_scope
	}

	pub const fn set_scope(&mut self, scope: usize) {
		self.current_scope = scope;
	}

	pub fn clear(&mut self) {
		self.stack_sizes.clear();
		self.assignments.clear();
		self.expressions.clear();
	}

	pub const fn locals_mut(
		&mut self,
	) -> (&mut HashMap<usize, u16>, &mut HashMap<ScopedLink, Local>) {
		(&mut self.stack_sizes, &mut self.assignments)
	}

	pub fn store_expression(&mut self, id: u32, source: Expression) {
		if self
			.expressions
			.try_insert((id, self.current_scope), source)
			.is_err()
		{
			unreachable!("expression already stored for id {id}")
		}
	}

	pub fn take_expression(&mut self, id: u32, scope: usize) -> Option<Expression> {
		self.expressions.remove(&(id, scope))
	}

	pub fn get_stack_size(&self, scope: usize) -> u16 {
		self.stack_sizes[&scope]
	}

	pub fn get_local(&self, link: Link) -> Option<Local> {
		self.get_scoped_local(link, self.current_scope)
	}

	pub fn get_scoped_local(&self, link: Link, scope: usize) -> Option<Local> {
		self.assignments.get(&(link, scope)).copied()
	}

	pub fn load(&mut self, link: Link) -> Expression {
		self.get_local(link).map_or_else(
			|| self.take_expression(link.0, self.current_scope).unwrap(),
			Expression::Local,
		)
	}

	pub fn load_all(&mut self, sources: &[Link]) -> Vec<Expression> {
		sources.iter().map(|&link| self.load(link)).collect()
	}

	pub fn load_local_assignments(&self, id: u32, ports: u16) -> Vec<Local> {
		let names = (0..ports).map(|port| (Link(id, port), self.current_scope));

		names.map(|name| self.assignments[&name]).collect()
	}

	pub fn load_assign_all(
		&self,
		id: u32,
		sources: &[Link],
		source_scope: usize,
	) -> Vec<(Local, Local)> {
		let destination_scope = self.current_scope;
		let destinations =
			(0..).map(move |port| self.assignments[&(Link(id, port), destination_scope)]);
		let sources = sources
			.iter()
			.map(|&link| self.assignments[&(link, source_scope)]);

		destinations.zip(sources).collect()
	}

	pub fn load_scoped(dependencies: Vec<(Name, Expression)>, function: Function) -> Expression {
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
		let environment = self.load(node.environment);

		let expression = Import {
			environment,
			namespace: Arc::clone(&node.namespace),
			identifier: Arc::clone(&node.identifier),
		};

		Expression::Import(expression.into())
	}

	fn load_export(&mut self, node: &control::Export) -> Export {
		let source = self.load(node.reference);

		Export {
			identifier: Arc::clone(&node.identifier),
			source,
		}
	}

	pub fn load_exports(&mut self, nodes: &[control::Export]) -> Vec<Export> {
		nodes
			.iter()
			.map(|export| self.load_export(export))
			.collect()
	}

	pub fn load_call(&mut self, node: &simple::Apply) -> Expression {
		let function = self.load(node.function);
		let arguments = self.load_all(&node.arguments);

		let call = Call {
			function,
			arguments,
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
			kind: node.kind,
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
			kind: node.kind,
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
			kind: node.kind,
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

	pub fn load_integer_sign_extend(&mut self, node: simple::IntegerSignExtend) -> Expression {
		let expression = IntegerExtend {
			source: self.load(node.source),
			kind: node.kind,
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
			kind: node.kind,
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
			kind: node.kind,
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
			kind: node.kind,
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

	pub fn load_mutable_new(&mut self, node: simple::MutableNew) -> Expression {
		let expression = GlobalNew {
			initializer: self.load(node.initializer),
		};

		Expression::GlobalNew(expression.into())
	}

	pub fn load_mutable_get(&mut self, node: simple::MutableGet) -> Expression {
		let expression = GlobalGet {
			source: self.load(node.source),
		};

		Expression::GlobalGet(expression.into())
	}

	pub fn load_location(&mut self, location: simple::Location) -> Location {
		let reference = self.load(location.reference);
		let offset = self.load(location.offset);

		Location { reference, offset }
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
			kind: node.kind,
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
