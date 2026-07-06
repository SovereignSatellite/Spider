use alloc::sync::Arc;
use std::io::{Result, Write};

use luau_tree::expression::{
	Aggregate, Apply, BooleanToInteger, BufferLength, Call, Expression, Extract, Field, Function,
	Index, Infix, Local, Match, MemoryNew, Name, Prefix, RefIsNull, TableNew,
};

use super::{LuauPrinter, library::NeedsName as _, print::Print};

mod conditional {
	use core::ops::Range;
	use std::io::{Result, Write};

	use luau_tree::expression::Expression;

	use crate::{LuauPrinter, print::Print as _};

	fn print_recursive(
		branches: &[Expression],
		condition: &Expression,
		range: Range<usize>,
		printer: &mut LuauPrinter,
		out: &mut dyn Write,
	) -> Result<()> {
		let center = range.start + (range.end - range.start) / 2;

		if range.start != center {
			write!(out, "if (")?;

			condition.print(printer, out)?;

			write!(out, ") < {center} then ")?;

			print_recursive(branches, condition, range.start..center, printer, out)?;

			write!(out, " else")?;

			if range.end != center + 1 {
				write!(out, "if (")?;

				condition.print(printer, out)?;

				write!(out, ") > {center} then ")?;

				print_recursive(branches, condition, (center + 1)..range.end, printer, out)?;

				write!(out, " else")?;
			}

			write!(out, " ")?;
		}

		branches[center].print(printer, out)
	}

	pub fn print_match(
		branches: &[Expression],
		condition: &Expression,
		printer: &mut LuauPrinter,
		out: &mut dyn Write,
	) -> Result<()> {
		print_recursive(branches, condition, 0..branches.len(), printer, out)
	}

	pub fn print_if(
		on_false: &Expression,
		on_true: &Expression,
		condition: &Expression,
		printer: &mut LuauPrinter,
		out: &mut dyn Write,
	) -> Result<()> {
		write!(out, "if ")?;

		condition.print(printer, out)?;

		write!(out, " then ")?;

		on_true.print(printer, out)?;

		write!(out, " else ")?;

		on_false.print(printer, out)
	}
}

pub fn fmt_delimited<T, I>(items: I, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()>
where
	T: Print,
	I: IntoIterator<Item = T>,
{
	let mut iter = items.into_iter();

	if let Some(first) = iter.next() {
		first.print(printer, out)?;

		iter.try_for_each(|item| {
			write!(out, ", ")?;
			item.print(printer, out)
		})
	} else {
		Ok(())
	}
}

// A trailing argument that is itself a multi-value call (e.g. `from_bits_i64`)
// would otherwise spill its extra results into the call; parenthesizing the
// final argument collapses it to the single value the call position intends.
fn fmt_arguments(
	arguments: &[Expression],
	printer: &mut LuauPrinter,
	out: &mut dyn Write,
) -> Result<()> {
	let Some((last, leading)) = arguments.split_last() else {
		return Ok(());
	};

	for argument in leading {
		argument.print(printer, out)?;

		write!(out, ", ")?;
	}

	write!(out, "(")?;

	last.print(printer, out)?;

	write!(out, ")")
}

fn fmt_stack_enter(size: u16, printer: &LuauPrinter, out: &mut dyn Write) -> Result<()> {
	if size == 0 {
		return Ok(());
	}

	printer.write_indent(out)?;
	writeln!(out, "local stack = stack_acquire({size})")
}

fn fmt_stack_leave(size: u16, printer: &LuauPrinter, out: &mut dyn Write) -> Result<()> {
	if size == 0 {
		return Ok(());
	}

	printer.write_indent(out)?;
	writeln!(out, "stack_release({size}, stack)")
}

fn fmt_infix_operator(
	lhs: &Expression,
	rhs: &Expression,
	operator: &'static str,
	printer: &mut LuauPrinter,
	out: &mut dyn Write,
) -> Result<()> {
	write!(out, "(")?;

	lhs.print(printer, out)?;

	write!(out, ") {operator} (")?;

	rhs.print(printer, out)?;

	write!(out, ")")
}

impl Print for Name {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { id } = self;
		let prefix = printer.get_name(*self).unwrap_or("loc");

		write!(out, "{prefix}_{id}_")
	}
}

fn fmt_locals(names: &[Name], printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
	if names.is_empty() {
		return Ok(());
	}

	printer.write_indent(out)?;
	write!(out, "local ")?;

	fmt_delimited(names, printer, out)?;

	writeln!(out)
}

impl Print for Local {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		match self {
			Self::Fast { name } => name.print(printer, out),
			Self::Slow { offset } => {
				write!(out, "stack[{}]", offset + 1)
			}
		}
	}
}

impl Print for Function {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			arguments,
			locals,
			stack,
			code,
			returns,
		} = self;

		write!(out, "(function(")?;

		fmt_delimited(arguments, printer, out)?;

		writeln!(out, ")")?;

		printer.indent();

		fmt_stack_enter(*stack, printer, out)?;
		fmt_locals(locals, printer, out)?;

		code.print(printer, out)?;

		fmt_stack_leave(*stack, printer, out)?;

		if !returns.is_empty() {
			printer.write_indent(out)?;
			write!(out, "return ")?;

			fmt_delimited(returns, printer, out)?;

			writeln!(out)?;
		}

		printer.outdent();

		printer.write_indent(out)?;
		write!(out, "end)")
	}
}

impl Print for Match {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			branches,
			condition,
		} = self;

		if let [on_false, on_true] = branches.as_slice() {
			conditional::print_if(on_false, on_true, condition, printer, out)
		} else {
			conditional::print_match(branches, condition, printer, out)
		}
	}
}

impl Print for i32 {
	fn print(&self, _printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let inner = u32::from_ne_bytes(self.to_ne_bytes());

		write!(out, "{inner}")
	}
}

impl Print for i64 {
	fn print(&self, _printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let [b1, b2, b3, b4, b5, b6, b7, b8] = self.to_le_bytes();

		let low_bits = u32::from_le_bytes([b1, b2, b3, b4]);
		let high_bits = u32::from_le_bytes([b5, b6, b7, b8]);

		let intrinsic = self.needs_name();

		write!(out, "{intrinsic}(0x{low_bits:08X}, 0x{high_bits:08X})")
	}
}

impl Print for f32 {
	fn print(&self, _printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let inner = self.to_bits();

		write!(out, "0x{inner:08X}")
	}
}

impl Print for f64 {
	fn print(&self, _printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		if self.is_finite() {
			write!(out, "{self:e}")
		} else {
			let [b1, b2, b3, b4, b5, b6, b7, b8] = self.to_le_bytes();

			let low_bits = u32::from_le_bytes([b1, b2, b3, b4]);
			let high_bits = u32::from_le_bytes([b5, b6, b7, b8]);

			let intrinsic = self.needs_name();

			write!(out, "{intrinsic}(0x{low_bits:08X}, 0x{high_bits:08X})")
		}
	}
}

impl Print for Arc<str> {
	fn print(&self, _printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let escaped = self.as_bytes().escape_ascii();

		write!(out, "\"{escaped}\"")
	}
}

impl Print for Call {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			function,
			arguments,
		} = self;

		function.print(printer, out)?;

		write!(out, "(")?;

		fmt_arguments(arguments, printer, out)?;

		write!(out, ")")
	}
}

impl<const N: usize> Print for Apply<N> {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		write!(out, "{}(", self.name)?;

		fmt_arguments(&self.arguments, printer, out)?;

		write!(out, ")")
	}
}

impl Print for Infix {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { operator, lhs, rhs } = self;

		fmt_infix_operator(lhs, rhs, operator, printer, out)
	}
}

impl Print for Prefix {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { operator, source } = self;

		write!(out, "{operator}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for BooleanToInteger {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		write!(out, "if ")?;

		source.print(printer, out)?;

		write!(out, " then 1 else 0")
	}
}

impl Print for RefIsNull {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		write!(out, "(")?;

		source.print(printer, out)?;

		write!(out, ") == nil")
	}
}

impl Print for Aggregate {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { fields } = self;

		write!(out, "{{ ")?;

		for field in fields {
			field.print(printer, out)?;

			write!(out, ", ")?;
		}

		write!(out, "}}")
	}
}

impl Print for Extract {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source, index } = self;

		write!(out, "(")?;

		source.print(printer, out)?;

		write!(out, ")[{}]", index + 1)
	}
}

impl Print for Field {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source, name } = self;

		write!(out, "(")?;

		source.print(printer, out)?;

		write!(out, ").{name}")
	}
}

impl Print for Index {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source, offset } = self;

		write!(out, "(")?;

		source.print(printer, out)?;

		write!(out, ")[")?;

		offset.print(printer, out)?;

		write!(out, "]")
	}
}

impl Print for TableNew {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			initializer,
			minimum,
			maximum,
		} = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}({{ ")?;

		for (expression, offset) in initializer {
			write!(out, "[{offset}] = ")?;

			expression.print(printer, out)?;

			write!(out, ", ")?;
		}

		write!(out, "}}, {minimum}, {maximum})")
	}
}

impl Print for BufferLength {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		write!(out, "buffer.len((")?;

		source.print(printer, out)?;

		write!(out, ")[1])")
	}
}

impl Print for MemoryNew {
	fn print(&self, _printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			initializer,
			minimum,
			maximum,
		} = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}({{ ")?;

		for (data, offset) in initializer {
			write!(out, "[{offset}] = \"{}\", ", data.escape_ascii())?;
		}

		write!(out, "}}, {minimum}, {maximum})")
	}
}

impl Print for Expression {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		stacker::maybe_grow(crate::STACK_RED_ZONE, crate::STACK_SEGMENT, || match self {
			Self::Function(function) => function.print(printer, out),
			Self::Match(inner) => inner.print(printer, out),
			Self::Trap => write!(out, "error('unreachable code')"),
			Self::Null => write!(out, "nil"),
			Self::Local(local) => local.print(printer, out),
			Self::I32(i32) => i32.print(printer, out),
			Self::I64(i64) => i64.print(printer, out),
			Self::F32(f32) => f32.print(printer, out),
			Self::F64(f64) => f64.print(printer, out),
			Self::String(string) => string.print(printer, out),
			Self::Call(call) => call.print(printer, out),
			Self::Apply0Arguments(apply) => apply.print(printer, out),
			Self::Apply1Argument(apply) => apply.print(printer, out),
			Self::Apply2Arguments(apply) => apply.print(printer, out),
			Self::Apply3Arguments(apply) => apply.print(printer, out),
			Self::Apply4Arguments(apply) => apply.print(printer, out),
			Self::Apply5Arguments(apply) => apply.print(printer, out),
			Self::Infix(infix) => infix.print(printer, out),
			Self::Prefix(prefix) => prefix.print(printer, out),
			Self::BooleanToInteger(boolean_to_integer) => boolean_to_integer.print(printer, out),
			Self::RefIsNull(ref_is_null) => ref_is_null.print(printer, out),
			Self::Aggregate(aggregate) => aggregate.print(printer, out),
			Self::Extract(extract) => extract.print(printer, out),
			Self::Field(field) => field.print(printer, out),
			Self::TableNew(table_new) => table_new.print(printer, out),
			Self::Index(index) => index.print(printer, out),
			Self::MemoryNew(memory_new) => memory_new.print(printer, out),
			Self::BufferLength(buffer_length) => buffer_length.print(printer, out),
		})
	}
}
