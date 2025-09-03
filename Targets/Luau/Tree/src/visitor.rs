use core::ops::ControlFlow;

use crate::{
	LuauTree,
	expression::{
		Call as ExpressionCall, ElementsNew, Expression, Function, GlobalGet, GlobalNew, Import,
		IntegerBinaryOperation, IntegerCompareOperation, IntegerConvertToNumber, IntegerExtend,
		IntegerNarrow, IntegerTransmuteToNumber, IntegerUnaryOperation, IntegerWiden, Location,
		Match as ExpressionMatch, MemoryGrow, MemoryLoad, MemorySize, NumberBinaryOperation,
		NumberCompareOperation, NumberNarrow, NumberTransmuteToInteger, NumberTruncateToInteger,
		NumberUnaryOperation, NumberWiden, RefIsNull, Scoped, TableGet, TableGrow, TableNew,
		TableSize,
	},
	statement::{
		Assign, Call as StatementCall, DataDrop, ElementsDrop, Export, FastDefine, GlobalSet,
		Match as StatementMatch, MemoryCopy, MemoryFill, MemoryInit, MemoryStore, Repeat, Sequence,
		Statement, TableCopy, TableFill, TableInit, TableSet,
	},
};

pub trait Visitor {
	type Output;

	fn visit_expression(&mut self, expression: &Expression) -> ControlFlow<Self::Output>;

	fn visit_statement(&mut self, statement: &Statement) -> ControlFlow<Self::Output>;
}

impl Function {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			arguments: _,
			code,
			returns: _,
		} = self;

		code.accept(visitor)
	}
}

impl Scoped {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { locals, function } = self;

		locals.iter().try_for_each(|local| local.accept(visitor))?;
		function.accept(visitor)
	}
}

impl ExpressionMatch {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			condition,
			branches,
		} = self;

		condition.accept(visitor)?;
		branches
			.iter()
			.try_for_each(|branch| branch.accept(visitor))
	}
}

impl Import {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			environment,
			namespace: _,
			identifier: _,
		} = self;

		environment.accept(visitor)
	}
}

impl ExpressionCall {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			function,
			arguments,
		} = self;

		function.accept(visitor)?;
		arguments
			.iter()
			.try_for_each(|argument| argument.accept(visitor))
	}
}

impl RefIsNull {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source } = self;

		source.accept(visitor)
	}
}

impl IntegerUnaryOperation {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			source,
			r#type: _,
			operator: _,
		} = self;

		source.accept(visitor)
	}
}

impl IntegerBinaryOperation {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		lhs.accept(visitor)?;
		rhs.accept(visitor)
	}
}

impl IntegerCompareOperation {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		lhs.accept(visitor)?;
		rhs.accept(visitor)
	}
}

impl IntegerNarrow {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source } = self;

		source.accept(visitor)
	}
}

impl IntegerWiden {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source } = self;

		source.accept(visitor)
	}
}

impl IntegerExtend {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source, r#type: _ } = self;

		source.accept(visitor)
	}
}

impl IntegerConvertToNumber {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			source,
			signed: _,
			to: _,
			from: _,
		} = self;

		source.accept(visitor)
	}
}

impl IntegerTransmuteToNumber {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source, from: _ } = self;

		source.accept(visitor)
	}
}

impl NumberUnaryOperation {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			source,
			r#type: _,
			operator: _,
		} = self;

		source.accept(visitor)
	}
}

impl NumberBinaryOperation {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		lhs.accept(visitor)?;
		rhs.accept(visitor)
	}
}

impl NumberCompareOperation {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		lhs.accept(visitor)?;
		rhs.accept(visitor)
	}
}

impl NumberNarrow {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source } = self;

		source.accept(visitor)
	}
}

impl NumberWiden {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source } = self;

		source.accept(visitor)
	}
}

impl NumberTruncateToInteger {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			source,
			signed: _,
			saturate: _,
			to: _,
			from: _,
		} = self;

		source.accept(visitor)
	}
}

impl NumberTransmuteToInteger {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source, from: _ } = self;

		source.accept(visitor)
	}
}

impl Location {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { reference, offset } = self;

		reference.accept(visitor)?;
		offset.accept(visitor)
	}
}

impl GlobalNew {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { initializer } = self;

		initializer.accept(visitor)
	}
}

impl GlobalGet {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source } = self;

		source.accept(visitor)
	}
}

impl TableNew {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			initializer,
			minimum: _,
			maximum: _,
		} = self;

		initializer.accept(visitor)
	}
}

impl TableGet {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source } = self;

		source.accept(visitor)
	}
}

impl TableSize {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source } = self;

		source.accept(visitor)
	}
}

impl TableGrow {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			destination,
			initializer,
			size,
		} = self;

		destination.accept(visitor)?;
		initializer.accept(visitor)?;
		size.accept(visitor)
	}
}

impl ElementsNew {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { content } = self;

		content
			.iter()
			.try_for_each(|element| element.accept(visitor))
	}
}

impl MemoryLoad {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source, r#type: _ } = self;

		source.accept(visitor)
	}
}

impl MemorySize {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source } = self;

		source.accept(visitor)
	}
}

impl MemoryGrow {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { destination, size } = self;

		destination.accept(visitor)?;
		size.accept(visitor)
	}
}

impl Expression {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		visitor.visit_expression(self)?;

		match self {
			Self::Trap
			| Self::Null
			| Self::Local(_)
			| Self::I32(_)
			| Self::I64(_)
			| Self::F32(_)
			| Self::F64(_)
			| Self::MemoryNew(_)
			| Self::DataNew(_) => ControlFlow::Continue(()),

			Self::Function(function) => function.accept(visitor),
			Self::Scoped(scoped) => scoped.accept(visitor),
			Self::Match(r#match) => r#match.accept(visitor),
			Self::Import(import) => import.accept(visitor),
			Self::Call(call) => call.accept(visitor),
			Self::RefIsNull(ref_is_null) => ref_is_null.accept(visitor),
			Self::IntegerUnaryOperation(integer_unary_operation) => {
				integer_unary_operation.accept(visitor)
			}
			Self::IntegerBinaryOperation(integer_binary_operation) => {
				integer_binary_operation.accept(visitor)
			}
			Self::IntegerCompareOperation(integer_compare_operation) => {
				integer_compare_operation.accept(visitor)
			}
			Self::IntegerNarrow(integer_narrow) => integer_narrow.accept(visitor),
			Self::IntegerWiden(integer_widen) => integer_widen.accept(visitor),
			Self::IntegerExtend(integer_extend) => integer_extend.accept(visitor),
			Self::IntegerConvertToNumber(integer_convert_to_number) => {
				integer_convert_to_number.accept(visitor)
			}
			Self::IntegerTransmuteToNumber(integer_transmute_to_number) => {
				integer_transmute_to_number.accept(visitor)
			}
			Self::NumberUnaryOperation(number_unary_operation) => {
				number_unary_operation.accept(visitor)
			}
			Self::NumberBinaryOperation(number_binary_operation) => {
				number_binary_operation.accept(visitor)
			}
			Self::NumberCompareOperation(number_compare_operation) => {
				number_compare_operation.accept(visitor)
			}
			Self::NumberNarrow(number_narrow) => number_narrow.accept(visitor),
			Self::NumberWiden(number_widen) => number_widen.accept(visitor),
			Self::NumberTruncateToInteger(number_truncate_to_integer) => {
				number_truncate_to_integer.accept(visitor)
			}
			Self::NumberTransmuteToInteger(number_transmute_to_integer) => {
				number_transmute_to_integer.accept(visitor)
			}
			Self::GlobalNew(global_new) => global_new.accept(visitor),
			Self::GlobalGet(global_get) => global_get.accept(visitor),
			Self::TableNew(table_new) => table_new.accept(visitor),
			Self::TableGet(table_get) => table_get.accept(visitor),
			Self::TableSize(table_size) => table_size.accept(visitor),
			Self::TableGrow(table_grow) => table_grow.accept(visitor),
			Self::ElementsNew(elements_new) => elements_new.accept(visitor),
			Self::MemoryLoad(memory_load) => memory_load.accept(visitor),
			Self::MemorySize(memory_size) => memory_size.accept(visitor),
			Self::MemoryGrow(memory_grow) => memory_grow.accept(visitor),
		}
	}
}

impl Sequence {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { list } = self;

		list.iter()
			.try_for_each(|statement| statement.accept(visitor))
	}
}

impl StatementMatch {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			branches,
			condition,
		} = self;

		branches
			.iter()
			.try_for_each(|branch| branch.accept(visitor))?;

		condition.accept(visitor)
	}
}

impl Repeat {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			code,
			post: _,
			condition,
		} = self;

		code.accept(visitor)?;
		condition.accept(visitor)
	}
}

impl FastDefine {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { name: _, source } = self;

		source.accept(visitor)
	}
}

impl Assign {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { local: _, source } = self;

		source.accept(visitor)
	}
}

impl StatementCall {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			function,
			results: _,
			arguments,
		} = self;

		function.accept(visitor)?;
		arguments
			.iter()
			.try_for_each(|argument| argument.accept(visitor))
	}
}

impl GlobalSet {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			destination,
			source,
		} = self;

		destination.accept(visitor)?;
		source.accept(visitor)
	}
}

impl TableSet {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			destination,
			source,
		} = self;

		destination.accept(visitor)?;
		source.accept(visitor)
	}
}

impl TableFill {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			destination,
			source,
			size,
		} = self;

		destination.accept(visitor)?;
		source.accept(visitor)?;
		size.accept(visitor)
	}
}

impl TableCopy {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			destination,
			source,
			size,
		} = self;

		destination.accept(visitor)?;
		source.accept(visitor)?;
		size.accept(visitor)
	}
}

impl TableInit {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			destination,
			source,
			size,
		} = self;

		destination.accept(visitor)?;
		source.accept(visitor)?;
		size.accept(visitor)
	}
}

impl ElementsDrop {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source } = self;

		source.accept(visitor)
	}
}

impl MemoryStore {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			destination,
			source,
			r#type: _,
		} = self;

		destination.accept(visitor)?;
		source.accept(visitor)
	}
}

impl MemoryFill {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			destination,
			byte,
			size,
		} = self;

		destination.accept(visitor)?;
		byte.accept(visitor)?;
		size.accept(visitor)
	}
}

impl MemoryCopy {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			destination,
			source,
			size,
		} = self;

		destination.accept(visitor)?;
		source.accept(visitor)?;
		size.accept(visitor)
	}
}

impl MemoryInit {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			destination,
			source,
			size,
		} = self;

		destination.accept(visitor)?;
		source.accept(visitor)?;
		size.accept(visitor)
	}
}

impl DataDrop {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source } = self;

		source.accept(visitor)
	}
}

impl Statement {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		visitor.visit_statement(self)?;

		match self {
			Self::SlowDefine(_) | Self::AssignAll(_) => ControlFlow::Continue(()),

			Self::Match(r#match) => r#match.accept(visitor),
			Self::Repeat(repeat) => repeat.accept(visitor),
			Self::FastDefine(fast_define) => fast_define.accept(visitor),
			Self::Assign(assign) => assign.accept(visitor),
			Self::Call(call) => call.accept(visitor),
			Self::GlobalSet(global_set) => global_set.accept(visitor),
			Self::TableSet(table_set) => table_set.accept(visitor),
			Self::TableFill(table_fill) => table_fill.accept(visitor),
			Self::TableCopy(table_copy) => table_copy.accept(visitor),
			Self::TableInit(table_init) => table_init.accept(visitor),
			Self::ElementsDrop(elements_drop) => elements_drop.accept(visitor),
			Self::MemoryStore(memory_store) => memory_store.accept(visitor),
			Self::MemoryFill(memory_fill) => memory_fill.accept(visitor),
			Self::MemoryCopy(memory_copy) => memory_copy.accept(visitor),
			Self::MemoryInit(memory_init) => memory_init.accept(visitor),
			Self::DataDrop(data_drop) => data_drop.accept(visitor),
		}
	}
}

impl Export {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			identifier: _,
			source,
		} = self;

		source.accept(visitor)
	}
}

impl LuauTree {
	pub fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			environment: _,
			code,
			exports,
		} = self;

		code.accept(visitor)?;
		exports.iter().try_for_each(|export| export.accept(visitor))
	}
}
