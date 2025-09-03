use alloc::vec::Vec;
use data_flow_graph::{Link, mvp, nested};
use hashbrown::HashMap;
use luau_tree::{
	expression::{
		Call, ElementsNew, Expression, Function, GlobalGet, GlobalNew, Import,
		IntegerBinaryOperation, IntegerCompareOperation, IntegerConvertToNumber, IntegerExtend,
		IntegerNarrow, IntegerTransmuteToNumber, IntegerUnaryOperation, IntegerWiden, Local,
		Location, Match, MemoryGrow, MemoryLoad, MemorySize, Name, NumberBinaryOperation,
		NumberCompareOperation, NumberNarrow, NumberTransmuteToInteger, NumberTruncateToInteger,
		NumberUnaryOperation, NumberWiden, RefIsNull, Scoped, TableGet, TableGrow, TableNew,
		TableSize,
	},
	statement::{Export, FastDefine, Sequence},
};

pub struct DataHandler {
	expressions: HashMap<u32, Expression>,
	locals: HashMap<Link, Local>,

	links: Vec<Link>,
	scopes: Vec<usize>,
}

impl DataHandler {
	pub fn new() -> Self {
		Self {
			expressions: HashMap::new(),
			locals: HashMap::new(),

			links: Vec::new(),
			scopes: Vec::new(),
		}
	}

	pub fn pop_scope(&mut self) {
		let start = self.scopes.pop().unwrap_or_default();

		for link in &self.links[start..] {
			self.locals.remove(link).unwrap();
		}

		self.links.truncate(start);
	}

	pub fn push_scope(&mut self) {
		self.scopes.push(self.links.len());
	}

	pub fn define(&mut self, id: u32, source: Expression) {
		let last = self.expressions.insert(id, source);

		debug_assert!(last.is_none(), "expression should not be set");
	}

	pub fn alias(&mut self, link: Link, local: Local) {
		let last = self.locals.insert(link, local);

		self.links.push(link);

		debug_assert!(last.is_none(), "local should not be set");
	}

	pub fn load_local(&self, link: Link) -> Option<Local> {
		self.locals.get(&link).copied()
	}

	pub fn load(&mut self, link: Link) -> Option<Expression> {
		self.load_local(link).map(Expression::Local).or_else(|| {
			if link.1 == 0 {
				self.expressions.remove(&link.0)
			} else {
				None
			}
		})
	}

	pub fn load_sources(&mut self, sources: &[Link]) -> Vec<Expression> {
		sources.iter().map_while(|&link| self.load(link)).collect()
	}

	pub fn load_locals(&mut self, sources: &[Link]) -> Vec<Local> {
		fn assert_into_local(local: Expression) -> Local {
			if let Expression::Local(local) = local {
				local
			} else {
				unreachable!()
			}
		}

		sources
			.iter()
			.map_while(|&link| self.load(link))
			.map(assert_into_local)
			.collect()
	}

	pub fn load_scoped(
		locals: Vec<FastDefine>,
		arguments: Vec<Name>,
		code: Sequence,
		returns: Vec<Local>,
	) -> Expression {
		let function = Function {
			arguments,
			code,
			returns,
		};

		if locals.is_empty() {
			Expression::Function(function.into())
		} else {
			let scoped = Scoped { locals, function };

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

	pub fn load_import(&mut self, import: &nested::Import) -> Expression {
		let import = Import {
			environment: self.load(import.environment).unwrap(),
			namespace: import.namespace.clone(),
			identifier: import.identifier.clone(),
		};

		Expression::Import(import.into())
	}

	pub fn load_export(&mut self, export: &nested::Export) -> Export {
		Export {
			identifier: export.identifier.clone(),
			source: self.load(export.reference).unwrap(),
		}
	}

	pub fn load_identity(&mut self, identity: mvp::Identity) -> Expression {
		self.load(identity.source).unwrap()
	}

	pub fn load_call(&mut self, call: &mvp::Call) -> Expression {
		let end = call.arguments.len() - usize::from(call.states);
		let call = Call {
			function: self.load(call.function).unwrap(),
			arguments: self.load_sources(&call.arguments[..end]),
		};

		Expression::Call(call.into())
	}

	pub fn load_location(&mut self, location: mvp::Location) -> Location {
		Location {
			reference: self.load(location.reference).unwrap(),
			offset: self.load(location.offset).unwrap(),
		}
	}

	pub fn load_ref_is_null(&mut self, ref_is_null: mvp::RefIsNull) -> Expression {
		let operation = RefIsNull {
			source: self.load(ref_is_null.source).unwrap(),
		};

		Expression::RefIsNull(operation.into())
	}

	pub fn load_integer_unary_operation(
		&mut self,
		operation: mvp::IntegerUnaryOperation,
	) -> Expression {
		let operation = IntegerUnaryOperation {
			source: self.load(operation.source).unwrap(),
			r#type: operation.r#type,
			operator: operation.operator,
		};

		Expression::IntegerUnaryOperation(operation.into())
	}

	pub fn load_integer_binary_operation(
		&mut self,
		operation: mvp::IntegerBinaryOperation,
	) -> Expression {
		let operation = IntegerBinaryOperation {
			lhs: self.load(operation.lhs).unwrap(),
			rhs: self.load(operation.rhs).unwrap(),
			r#type: operation.r#type,
			operator: operation.operator,
		};

		Expression::IntegerBinaryOperation(operation.into())
	}

	pub fn load_integer_compare_operation(
		&mut self,
		operation: mvp::IntegerCompareOperation,
	) -> Expression {
		let operation = IntegerCompareOperation {
			lhs: self.load(operation.lhs).unwrap(),
			rhs: self.load(operation.rhs).unwrap(),
			r#type: operation.r#type,
			operator: operation.operator,
		};

		Expression::IntegerCompareOperation(operation.into())
	}

	pub fn load_integer_narrow(&mut self, operation: mvp::IntegerNarrow) -> Expression {
		let operation = IntegerNarrow {
			source: self.load(operation.source).unwrap(),
		};

		Expression::IntegerNarrow(operation.into())
	}

	pub fn load_integer_widen(&mut self, operation: mvp::IntegerWiden) -> Expression {
		let operation = IntegerWiden {
			source: self.load(operation.source).unwrap(),
		};

		Expression::IntegerWiden(operation.into())
	}

	pub fn load_integer_extend(&mut self, operation: mvp::IntegerExtend) -> Expression {
		let operation = IntegerExtend {
			source: self.load(operation.source).unwrap(),
			r#type: operation.r#type,
		};

		Expression::IntegerExtend(operation.into())
	}

	pub fn load_integer_convert_to_number(
		&mut self,
		operation: mvp::IntegerConvertToNumber,
	) -> Expression {
		let operation = IntegerConvertToNumber {
			source: self.load(operation.source).unwrap(),
			signed: operation.signed,
			to: operation.to,
			from: operation.from,
		};

		Expression::IntegerConvertToNumber(operation.into())
	}

	pub fn load_integer_transmute_to_number(
		&mut self,
		operation: mvp::IntegerTransmuteToNumber,
	) -> Expression {
		let operation = IntegerTransmuteToNumber {
			source: self.load(operation.source).unwrap(),
			from: operation.from,
		};

		Expression::IntegerTransmuteToNumber(operation.into())
	}

	pub fn load_number_unary_operation(
		&mut self,
		operation: mvp::NumberUnaryOperation,
	) -> Expression {
		let operation = NumberUnaryOperation {
			source: self.load(operation.source).unwrap(),
			r#type: operation.r#type,
			operator: operation.operator,
		};

		Expression::NumberUnaryOperation(operation.into())
	}

	pub fn load_number_binary_operation(
		&mut self,
		operation: mvp::NumberBinaryOperation,
	) -> Expression {
		let operation = NumberBinaryOperation {
			lhs: self.load(operation.lhs).unwrap(),
			rhs: self.load(operation.rhs).unwrap(),
			r#type: operation.r#type,
			operator: operation.operator,
		};

		Expression::NumberBinaryOperation(operation.into())
	}

	pub fn load_number_compare_operation(
		&mut self,
		operation: mvp::NumberCompareOperation,
	) -> Expression {
		let operation = NumberCompareOperation {
			lhs: self.load(operation.lhs).unwrap(),
			rhs: self.load(operation.rhs).unwrap(),
			r#type: operation.r#type,
			operator: operation.operator,
		};

		Expression::NumberCompareOperation(operation.into())
	}

	pub fn load_number_narrow(&mut self, operation: mvp::NumberNarrow) -> Expression {
		let operation = NumberNarrow {
			source: self.load(operation.source).unwrap(),
		};

		Expression::NumberNarrow(operation.into())
	}

	pub fn load_number_widen(&mut self, operation: mvp::NumberWiden) -> Expression {
		let operation = NumberWiden {
			source: self.load(operation.source).unwrap(),
		};

		Expression::NumberWiden(operation.into())
	}

	pub fn load_number_truncate_to_integer(
		&mut self,
		operation: mvp::NumberTruncateToInteger,
	) -> Expression {
		let operation = NumberTruncateToInteger {
			source: self.load(operation.source).unwrap(),
			signed: operation.signed,
			saturate: operation.saturate,
			to: operation.to,
			from: operation.from,
		};

		Expression::NumberTruncateToInteger(operation.into())
	}

	pub fn load_number_transmute_to_integer(
		&mut self,
		operation: mvp::NumberTransmuteToInteger,
	) -> Expression {
		let operation = NumberTransmuteToInteger {
			source: self.load(operation.source).unwrap(),
			from: operation.from,
		};

		Expression::NumberTransmuteToInteger(operation.into())
	}

	pub fn load_global_new(&mut self, global_new: mvp::GlobalNew) -> Expression {
		let global_new = GlobalNew {
			initializer: self.load(global_new.initializer).unwrap(),
		};

		Expression::GlobalNew(global_new.into())
	}

	pub fn load_global_get(&mut self, global_get: mvp::GlobalGet) -> Expression {
		let global_get = GlobalGet {
			source: self.load(global_get.source).unwrap(),
		};

		Expression::GlobalGet(global_get.into())
	}

	pub fn load_table_new(&mut self, table_new: mvp::TableNew) -> Expression {
		let table_new = TableNew {
			initializer: self.load(table_new.initializer).unwrap(),
			minimum: table_new.minimum,
			maximum: table_new.maximum,
		};

		Expression::TableNew(table_new.into())
	}

	pub fn load_table_get(&mut self, table_get: mvp::TableGet) -> Expression {
		let table_get = TableGet {
			source: self.load_location(table_get.source),
		};

		Expression::TableGet(table_get.into())
	}

	pub fn load_table_size(&mut self, table_size: mvp::TableSize) -> Expression {
		let table_size = TableSize {
			source: self.load(table_size.source).unwrap(),
		};

		Expression::TableSize(table_size.into())
	}

	pub fn load_table_grow(&mut self, table_grow: mvp::TableGrow) -> Expression {
		let table_grow = TableGrow {
			destination: self.load(table_grow.destination).unwrap(),
			initializer: self.load(table_grow.initializer).unwrap(),
			size: self.load(table_grow.size).unwrap(),
		};

		Expression::TableGrow(table_grow.into())
	}

	pub fn load_elements_new(&mut self, elements_new: &mvp::ElementsNew) -> Expression {
		let elements_new = ElementsNew {
			content: self.load_sources(&elements_new.content),
		};

		Expression::ElementsNew(elements_new.into())
	}

	pub fn load_memory_load(&mut self, memory_load: mvp::MemoryLoad) -> Expression {
		let memory_load = MemoryLoad {
			source: self.load_location(memory_load.source),
			r#type: memory_load.r#type,
		};

		Expression::MemoryLoad(memory_load.into())
	}

	pub fn load_memory_size(&mut self, memory_size: mvp::MemorySize) -> Expression {
		let memory_size = MemorySize {
			source: self.load(memory_size.source).unwrap(),
		};

		Expression::MemorySize(memory_size.into())
	}

	pub fn load_memory_grow(&mut self, memory_grow: mvp::MemoryGrow) -> Expression {
		let memory_grow = MemoryGrow {
			destination: self.load(memory_grow.destination).unwrap(),
			size: self.load(memory_grow.size).unwrap(),
		};

		Expression::MemoryGrow(memory_grow.into())
	}
}
