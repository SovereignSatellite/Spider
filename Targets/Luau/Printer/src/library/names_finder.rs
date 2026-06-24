use core::ops::ControlFlow;

use luau_tree::{
	expression::{
		Aggregate, Apply, BufferLength, Expression, Extract, Function, GlobalGet, GlobalNew, Index,
		MemoryNew, TableNew, TableSize, VectorX,
	},
	statement::{GlobalSet, SetIndex, Statement},
	visitor::Visitor,
};

/// A trait for items that may need a runtime library function name.
pub trait NeedsName {
	/// Returns the runtime library function name needed, or empty if none.
	fn needs_name(&self) -> &'static str;
}

impl NeedsName for i32 {
	fn needs_name(&self) -> &'static str {
		""
	}
}

impl NeedsName for i64 {
	fn needs_name(&self) -> &'static str {
		"into_bits_i64"
	}
}

impl NeedsName for f32 {
	fn needs_name(&self) -> &'static str {
		""
	}
}

impl NeedsName for f64 {
	fn needs_name(&self) -> &'static str {
		if self.is_finite() {
			""
		} else {
			"into_bits_i64"
		}
	}
}

impl<const N: usize> NeedsName for Apply<N> {
	fn needs_name(&self) -> &'static str {
		self.name.strip_prefix("rt_").unwrap_or(self.name)
	}
}

impl NeedsName for GlobalNew {
	fn needs_name(&self) -> &'static str {
		""
	}
}

impl NeedsName for GlobalGet {
	fn needs_name(&self) -> &'static str {
		""
	}
}

impl NeedsName for Aggregate {
	fn needs_name(&self) -> &'static str {
		""
	}
}

impl NeedsName for Extract {
	fn needs_name(&self) -> &'static str {
		""
	}
}

impl NeedsName for Index {
	fn needs_name(&self) -> &'static str {
		""
	}
}

impl NeedsName for TableNew {
	fn needs_name(&self) -> &'static str {
		"table_new"
	}
}

impl NeedsName for TableSize {
	fn needs_name(&self) -> &'static str {
		"table_size"
	}
}

impl NeedsName for MemoryNew {
	fn needs_name(&self) -> &'static str {
		"memory_new"
	}
}

impl NeedsName for BufferLength {
	fn needs_name(&self) -> &'static str {
		""
	}
}

impl NeedsName for VectorX {
	fn needs_name(&self) -> &'static str {
		""
	}
}

impl NeedsName for Expression {
	fn needs_name(&self) -> &'static str {
		match self {
			Self::Function(_)
			| Self::Match(_)
			| Self::Trap
			| Self::Null
			| Self::Local(_)
			| Self::String(_)
			| Self::Call(_)
			| Self::Infix(_)
			| Self::Prefix(_)
			| Self::BooleanToInteger(_)
			| Self::RefIsNull(_) => "",

			Self::I32(i32) => i32.needs_name(),
			Self::I64(i64) => i64.needs_name(),
			Self::F32(f32) => f32.needs_name(),
			Self::F64(f64) => f64.needs_name(),

			Self::Apply0Arguments(apply) => apply.needs_name(),
			Self::Apply1Argument(apply) => apply.needs_name(),
			Self::Apply2Arguments(apply) => apply.needs_name(),
			Self::Apply3Arguments(apply) => apply.needs_name(),
			Self::Apply4Arguments(apply) => apply.needs_name(),
			Self::Apply5Arguments(apply) => apply.needs_name(),

			Self::GlobalNew(global_new) => global_new.needs_name(),
			Self::GlobalGet(global_get) => global_get.needs_name(),
			Self::Aggregate(aggregate) => aggregate.needs_name(),
			Self::Extract(extract) => extract.needs_name(),
			Self::TableNew(table_new) => table_new.needs_name(),
			Self::TableSize(table_size) => table_size.needs_name(),
			Self::Index(index) => index.needs_name(),
			Self::MemoryNew(memory_new) => memory_new.needs_name(),
			Self::BufferLength(buffer_length) => buffer_length.needs_name(),
			Self::VectorX(vector_x) => vector_x.needs_name(),
		}
	}
}

impl NeedsName for GlobalSet {
	fn needs_name(&self) -> &'static str {
		""
	}
}

impl NeedsName for SetIndex {
	fn needs_name(&self) -> &'static str {
		""
	}
}

impl NeedsName for Statement {
	fn needs_name(&self) -> &'static str {
		match self {
			Self::Match(_)
			| Self::Repeat(_)
			| Self::Assign(_)
			| Self::SwapAll(_)
			| Self::Call(_) => "",

			Self::GlobalSet(global_set) => global_set.needs_name(),
			Self::SetIndex(set_index) => set_index.needs_name(),
		}
	}
}

/// Collects all runtime library function names needed by a function.
pub struct NamesFinder<'names> {
	names: &'names mut Vec<&'static str>,
}

impl<'names> NamesFinder<'names> {
	/// Creates a new names finder writing to the given list.
	pub const fn new(names: &'names mut Vec<&'static str>) -> Self {
		Self { names }
	}

	/// Collects all needed names from the function.
	pub fn run(&mut self, function: &Function) {
		self.names.extend(["stack_acquire", "stack_release"]);

		function
			.accept(self)
			.continue_value()
			.unwrap_or_else(|| unreachable!("names finder must not fail"));
	}
}

impl Visitor for NamesFinder<'_> {
	type Output = ();

	fn visit_expression(&mut self, expression: &Expression) -> ControlFlow<Self::Output> {
		let name = expression.needs_name();

		if !name.is_empty() {
			self.names.push(name);
		}

		ControlFlow::Continue(())
	}

	fn visit_statement(&mut self, statement: &Statement) -> ControlFlow<Self::Output> {
		let name = statement.needs_name();

		if !name.is_empty() {
			self.names.push(name);
		}

		ControlFlow::Continue(())
	}
}
