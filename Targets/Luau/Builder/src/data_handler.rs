use alloc::vec::Vec;
use data_flow_graph::{Link, base, control};
use hashbrown::HashMap;
use luau_tree::{
	expression::{
		BooleanToInteger, Call, ElementsNew, Expression, Function, GlobalGet, GlobalNew, Import,
		IntegerBinaryOperation, IntegerCompareOperation, IntegerConvertToNumber, IntegerExtend,
		IntegerNarrow, IntegerTransmuteToNumber, IntegerUnaryOperation, IntegerWiden, Local,
		Location, Match, MemoryGrow, MemoryLoad, MemorySize, Name, NumberBinaryOperation,
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
		let last = self.expressions.insert(id, source);

		debug_assert!(last.is_none(), "expression should set only once");
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
		let destinations = (0..).map(|port| Link(id, port));
		let iter = destinations
			.zip(sources)
			.filter_map(|(destination, &source)| {
				let destination = self.assignments[&destination];
				let source = self.assignments[&source];

				(destination != source).then_some((destination, source))
			});

		iter.collect()
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
		&self,
		results: &[Link],
		function_type: &control::FunctionType,
	) -> Vec<Local> {
		let returns = results.iter().map(|&name| self.assignments[&name]);
		let len = function_type.results.len();

		returns.take(len).collect()
	}

	pub fn load_scoped(
		dependencies: Vec<(Name, Expression)>,
		arguments: Vec<Name>,
		locals: Vec<Name>,
		stack: u16,
		code: Sequence,
		returns: Vec<Local>,
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

	fn load_match_expression(condition: Expression, branches: Vec<Sequence>) -> Expression {
		let branches = branches
			.into_iter()
			.map(Sequence::into_assign_source)
			.collect();

		Expression::Match(
			Match {
				condition,
				branches,
			}
			.into(),
		)
	}

	// FIXME: This doesn't handle the case where we don't have assignments
	// because they are just forwarding values?
	#[expect(dead_code)]
	pub fn load_match_expression_optional(
		condition: Expression,
		branches: Vec<Sequence>,
	) -> Result<Expression, (Expression, Vec<Sequence>)> {
		let mut locals = branches.iter().map(Sequence::as_assign_destination);

		if let Some(local) = locals.next().flatten()
			&& locals.all(|other| other == Some(local))
		{
			let expression = Self::load_match_expression(condition, branches);

			Ok(expression)
		} else {
			Err((condition, branches))
		}
	}

	pub fn load_import(&mut self, import: &control::Import) -> Expression {
		let import = Import {
			environment: self.load(import.environment),
			namespace: import.namespace.clone(),
			identifier: import.identifier.clone(),
		};

		Expression::Import(import.into())
	}

	fn load_export(&mut self, export: &control::Export) -> Export {
		Export {
			identifier: export.identifier.clone(),
			source: self.load(export.reference),
		}
	}

	pub fn load_exports(&mut self, exports: &[control::Export]) -> Vec<Export> {
		exports
			.iter()
			.map(|export| self.load_export(export))
			.collect()
	}

	pub fn load_identity(&mut self, identity: base::Identity) -> Expression {
		self.load(identity.source)
	}

	pub fn load_call(&mut self, call: &base::Call) -> Expression {
		let end = call.arguments.len() - usize::from(call.states);
		let call = Call {
			function: self.load(call.function),
			arguments: self.load_all(&call.arguments[..end]),
		};

		Expression::Call(call.into())
	}

	pub fn load_location(&mut self, location: base::Location) -> Location {
		Location {
			reference: self.load(location.reference),
			offset: self.load(location.offset),
		}
	}

	pub fn load_ref_is_null(&mut self, ref_is_null: base::RefIsNull) -> Expression {
		let operation = RefIsNull {
			source: self.load(ref_is_null.source),
		};

		let boolean = BooleanToInteger {
			source: Expression::RefIsNull(operation.into()),
		};

		Expression::BooleanToInteger(boolean.into())
	}

	pub fn load_integer_unary_operation(
		&mut self,
		operation: base::IntegerUnaryOperation,
	) -> Expression {
		let operation = IntegerUnaryOperation {
			source: self.load(operation.source),
			r#type: operation.r#type,
			operator: operation.operator,
		};

		Expression::IntegerUnaryOperation(operation.into())
	}

	pub fn load_integer_binary_operation(
		&mut self,
		operation: base::IntegerBinaryOperation,
	) -> Expression {
		let operation = IntegerBinaryOperation {
			lhs: self.load(operation.lhs),
			rhs: self.load(operation.rhs),
			r#type: operation.r#type,
			operator: operation.operator,
		};

		Expression::IntegerBinaryOperation(operation.into())
	}

	pub fn load_integer_compare_operation(
		&mut self,
		operation: base::IntegerCompareOperation,
	) -> Expression {
		let operation = IntegerCompareOperation {
			lhs: self.load(operation.lhs),
			rhs: self.load(operation.rhs),
			r#type: operation.r#type,
			operator: operation.operator,
		};

		let boolean = BooleanToInteger {
			source: Expression::IntegerCompareOperation(operation.into()),
		};

		Expression::BooleanToInteger(boolean.into())
	}

	pub fn load_integer_narrow(&mut self, operation: base::IntegerNarrow) -> Expression {
		let operation = IntegerNarrow {
			source: self.load(operation.source),
		};

		Expression::IntegerNarrow(operation.into())
	}

	pub fn load_integer_widen(&mut self, operation: base::IntegerWiden) -> Expression {
		let operation = IntegerWiden {
			source: self.load(operation.source),
		};

		Expression::IntegerWiden(operation.into())
	}

	pub fn load_integer_extend(&mut self, operation: base::IntegerExtend) -> Expression {
		let operation = IntegerExtend {
			source: self.load(operation.source),
			r#type: operation.r#type,
		};

		Expression::IntegerExtend(operation.into())
	}

	pub fn load_integer_convert_to_number(
		&mut self,
		operation: base::IntegerConvertToNumber,
	) -> Expression {
		let operation = IntegerConvertToNumber {
			source: self.load(operation.source),
			signed: operation.signed,
			to: operation.to,
			from: operation.from,
		};

		Expression::IntegerConvertToNumber(operation.into())
	}

	pub fn load_integer_transmute_to_number(
		&mut self,
		operation: base::IntegerTransmuteToNumber,
	) -> Expression {
		let operation = IntegerTransmuteToNumber {
			source: self.load(operation.source),
			from: operation.from,
		};

		Expression::IntegerTransmuteToNumber(operation.into())
	}

	pub fn load_number_unary_operation(
		&mut self,
		operation: base::NumberUnaryOperation,
	) -> Expression {
		let operation = NumberUnaryOperation {
			source: self.load(operation.source),
			r#type: operation.r#type,
			operator: operation.operator,
		};

		Expression::NumberUnaryOperation(operation.into())
	}

	pub fn load_number_binary_operation(
		&mut self,
		operation: base::NumberBinaryOperation,
	) -> Expression {
		let operation = NumberBinaryOperation {
			lhs: self.load(operation.lhs),
			rhs: self.load(operation.rhs),
			r#type: operation.r#type,
			operator: operation.operator,
		};

		Expression::NumberBinaryOperation(operation.into())
	}

	pub fn load_number_compare_operation(
		&mut self,
		operation: base::NumberCompareOperation,
	) -> Expression {
		let operation = NumberCompareOperation {
			lhs: self.load(operation.lhs),
			rhs: self.load(operation.rhs),
			r#type: operation.r#type,
			operator: operation.operator,
		};

		let boolean = BooleanToInteger {
			source: Expression::NumberCompareOperation(operation.into()),
		};

		Expression::BooleanToInteger(boolean.into())
	}

	pub fn load_number_narrow(&mut self, operation: base::NumberNarrow) -> Expression {
		let operation = NumberNarrow {
			source: self.load(operation.source),
		};

		Expression::NumberNarrow(operation.into())
	}

	pub fn load_number_widen(&mut self, operation: base::NumberWiden) -> Expression {
		let operation = NumberWiden {
			source: self.load(operation.source),
		};

		Expression::NumberWiden(operation.into())
	}

	pub fn load_number_truncate_to_integer(
		&mut self,
		operation: base::NumberTruncateToInteger,
	) -> Expression {
		let operation = NumberTruncateToInteger {
			source: self.load(operation.source),
			signed: operation.signed,
			saturate: operation.saturate,
			to: operation.to,
			from: operation.from,
		};

		Expression::NumberTruncateToInteger(operation.into())
	}

	pub fn load_number_transmute_to_integer(
		&mut self,
		operation: base::NumberTransmuteToInteger,
	) -> Expression {
		let operation = NumberTransmuteToInteger {
			source: self.load(operation.source),
			from: operation.from,
		};

		Expression::NumberTransmuteToInteger(operation.into())
	}

	pub fn load_global_new(&mut self, global_new: base::GlobalNew) -> Expression {
		let global_new = GlobalNew {
			initializer: self.load(global_new.initializer),
		};

		Expression::GlobalNew(global_new.into())
	}

	pub fn load_global_get(&mut self, global_get: base::GlobalGet) -> Expression {
		let global_get = GlobalGet {
			source: self.load(global_get.source),
		};

		Expression::GlobalGet(global_get.into())
	}

	pub fn load_table_new(&mut self, table_new: base::TableNew) -> Expression {
		let table_new = TableNew {
			initializer: self.load(table_new.initializer),
			minimum: table_new.minimum,
			maximum: table_new.maximum,
		};

		Expression::TableNew(table_new.into())
	}

	pub fn load_table_get(&mut self, table_get: base::TableGet) -> Expression {
		let table_get = TableGet {
			source: self.load_location(table_get.source),
		};

		Expression::TableGet(table_get.into())
	}

	pub fn load_table_size(&mut self, table_size: base::TableSize) -> Expression {
		let table_size = TableSize {
			source: self.load(table_size.source),
		};

		Expression::TableSize(table_size.into())
	}

	pub fn load_table_grow(&mut self, table_grow: base::TableGrow) -> Expression {
		let table_grow = TableGrow {
			destination: self.load(table_grow.destination),
			initializer: self.load(table_grow.initializer),
			size: self.load(table_grow.size),
		};

		Expression::TableGrow(table_grow.into())
	}

	pub fn load_elements_new(&mut self, elements_new: &base::ElementsNew) -> Expression {
		let elements_new = ElementsNew {
			content: self.load_all(&elements_new.content),
		};

		Expression::ElementsNew(elements_new.into())
	}

	pub fn load_memory_load(&mut self, memory_load: base::MemoryLoad) -> Expression {
		let memory_load = MemoryLoad {
			source: self.load_location(memory_load.source),
			r#type: memory_load.r#type,
		};

		Expression::MemoryLoad(memory_load.into())
	}

	pub fn load_memory_size(&mut self, memory_size: base::MemorySize) -> Expression {
		let memory_size = MemorySize {
			source: self.load(memory_size.source),
		};

		Expression::MemorySize(memory_size.into())
	}

	pub fn load_memory_grow(&mut self, memory_grow: base::MemoryGrow) -> Expression {
		let memory_grow = MemoryGrow {
			destination: self.load(memory_grow.destination),
			size: self.load(memory_grow.size),
		};

		Expression::MemoryGrow(memory_grow.into())
	}
}
