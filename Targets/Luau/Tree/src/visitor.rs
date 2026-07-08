//! Visitor pattern implementation for traversing the Luau tree.

use core::ops::ControlFlow;

use super::{
	expression::{
		Aggregate, Apply, BooleanToInteger, BufferLength, Call as ExpressionCall, Expression,
		Extract, Field, Function, Index, Infix, Match as ExpressionMatch, Prefix, RefIsNull,
		TableNew,
	},
	statement::{
		Assign, Call as StatementCall, Match as StatementMatch, Repeat, Sequence, SetIndex,
		Statement,
	},
};

/// A visitor for traversing tree nodes.
pub trait Visitor {
	/// The output type produced when traversal is interrupted.
	type Output;

	/// Visits an expression node.
	fn visit_expression(&mut self, expression: &Expression) -> ControlFlow<Self::Output>;

	/// Visits a statement node.
	fn visit_statement(&mut self, statement: &Statement) -> ControlFlow<Self::Output>;
}

impl Function {
	/// Accepts a visitor and traverses the function.
	pub fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { code, returns, .. } = self;

		code.accept_unbounded(visitor)?;

		returns
			.iter()
			.try_for_each(|inner| inner.accept_unbounded(visitor))
	}
}

impl ExpressionMatch {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			branches,
			condition,
		} = self;

		condition.accept_unbounded(visitor)?;
		branches
			.iter()
			.try_for_each(|branch| branch.accept_unbounded(visitor))
	}
}

impl ExpressionCall {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			function,
			arguments,
		} = self;

		function.accept_unbounded(visitor)?;
		arguments
			.iter()
			.try_for_each(|argument| argument.accept_unbounded(visitor))
	}
}

impl<const N: usize> Apply<N> {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		self.arguments
			.iter()
			.try_for_each(|argument| argument.accept_unbounded(visitor))
	}
}

impl Infix {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { lhs, rhs, .. } = self;

		lhs.accept_unbounded(visitor)?;
		rhs.accept_unbounded(visitor)
	}
}

impl Prefix {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source, .. } = self;

		source.accept_unbounded(visitor)
	}
}

impl BooleanToInteger {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source } = self;

		source.accept_unbounded(visitor)
	}
}

impl RefIsNull {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source } = self;

		source.accept_unbounded(visitor)
	}
}

impl Aggregate {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { fields } = self;

		fields
			.iter()
			.try_for_each(|field| field.accept_unbounded(visitor))
	}
}

impl Extract {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source, .. } = self;

		source.accept_unbounded(visitor)
	}
}

impl Field {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source, .. } = self;

		source.accept_unbounded(visitor)
	}
}

impl TableNew {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { initializer, .. } = self;

		initializer
			.iter()
			.try_for_each(|item| item.0.accept_unbounded(visitor))
	}
}

impl Index {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source, offset } = self;

		source.accept_unbounded(visitor)?;
		offset.accept_unbounded(visitor)
	}
}

impl BufferLength {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source } = self;

		source.accept_unbounded(visitor)
	}
}

impl Expression {
	fn accept_unbounded<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		stacker::maybe_grow(0x1_0000, 0x10_0000, || self.accept(visitor))
	}

	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		visitor.visit_expression(self)?;

		match self {
			Self::Function(function) => function.accept(visitor),
			Self::Match(inner) => inner.accept(visitor),

			Self::Trap
			| Self::Null
			| Self::Local(_)
			| Self::I32(_)
			| Self::I64(_)
			| Self::F32(_)
			| Self::F64(_)
			| Self::String(_)
			| Self::MemoryNew(_) => ControlFlow::Continue(()),

			Self::Call(call) => call.accept(visitor),
			Self::Apply0Arguments(apply) => apply.accept(visitor),
			Self::Apply1Argument(apply) => apply.accept(visitor),
			Self::Apply2Arguments(apply) => apply.accept(visitor),
			Self::Apply3Arguments(apply) => apply.accept(visitor),
			Self::Apply4Arguments(apply) => apply.accept(visitor),
			Self::Apply5Arguments(apply) => apply.accept(visitor),
			Self::Infix(infix) => infix.accept(visitor),
			Self::Prefix(prefix) => prefix.accept(visitor),
			Self::BooleanToInteger(boolean_to_integer) => boolean_to_integer.accept(visitor),
			Self::RefIsNull(ref_is_null) => ref_is_null.accept(visitor),
			Self::Aggregate(aggregate) => aggregate.accept(visitor),
			Self::Extract(extract) => extract.accept(visitor),
			Self::Field(field) => field.accept(visitor),
			Self::TableNew(table_new) => table_new.accept(visitor),
			Self::Index(index) => index.accept(visitor),
			Self::BufferLength(buffer_length) => buffer_length.accept(visitor),
		}
	}
}

impl Sequence {
	fn accept_unbounded<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		stacker::maybe_grow(0x1_0000, 0x10_0000, || self.accept(visitor))
	}

	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { statements } = self;

		statements
			.iter()
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
			.try_for_each(|branch| branch.accept_unbounded(visitor))?;

		condition.accept_unbounded(visitor)
	}
}

impl Repeat {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			code,
			condition,
			rotation,
		} = self;

		code.accept_unbounded(visitor)?;
		condition.accept_unbounded(visitor)?;
		rotation.accept_unbounded(visitor)
	}
}

impl Assign {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { source, .. } = self;

		source.accept_unbounded(visitor)
	}
}

impl StatementCall {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self { call, .. } = self;

		call.accept_unbounded(visitor)
	}
}

impl SetIndex {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		let Self {
			table,
			offset,
			value,
		} = self;

		table.accept_unbounded(visitor)?;
		offset.accept_unbounded(visitor)?;
		value.accept_unbounded(visitor)
	}
}

impl Statement {
	fn accept<T: Visitor>(&self, visitor: &mut T) -> ControlFlow<T::Output> {
		visitor.visit_statement(self)?;

		match self {
			Self::Match(inner) => inner.accept(visitor),
			Self::Repeat(repeat) => repeat.accept(visitor),
			Self::Assign(assign) => assign.accept(visitor),
			Self::SwapAll(_) => ControlFlow::Continue(()),
			Self::Call(call) => call.accept(visitor),
			Self::SetIndex(set_index) => set_index.accept(visitor),
		}
	}
}
