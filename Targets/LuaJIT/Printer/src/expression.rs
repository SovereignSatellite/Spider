use alloc::sync::Arc;
use std::io::{Result, Write};

use luajit_tree::expression::{
	Aggregate, Apply, BooleanToInteger, Call, Expression, Extract, Field, Function, Index, Infix,
	Local, MemoryNew, Name, Prefix, RefIsNull, TableNew,
};

use super::{LuaJITPrinter, library::NeedsName as _, print::Print};

pub fn fmt_delimited<T, I>(items: I, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()>
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
	printer: &mut LuaJITPrinter,
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

fn fmt_stack_enter(size: u16, printer: &LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
	if size == 0 {
		return Ok(());
	}

	printer.write_indent(out)?;
	writeln!(out, "local stack = stack_acquire({size})")
}

fn fmt_stack_leave(size: u16, printer: &LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
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
	printer: &mut LuaJITPrinter,
	out: &mut dyn Write,
) -> Result<()> {
	write!(out, "(")?;

	lhs.print(printer, out)?;

	write!(out, ") {operator} (")?;

	rhs.print(printer, out)?;

	write!(out, ")")
}

impl Print for Name {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { id } = self;
		let prefix = printer.get_name(*self).unwrap_or("loc");

		write!(out, "{prefix}_{id}_")
	}
}

fn fmt_locals(names: &[Name], printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
	if names.is_empty() {
		return Ok(());
	}

	printer.write_indent(out)?;
	write!(out, "local ")?;

	fmt_delimited(names, printer, out)?;

	writeln!(out)
}

impl Print for Local {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		match self {
			Self::Fast { name } => name.print(printer, out),
			Self::Slow { offset } => {
				write!(out, "stack[{}]", offset + 1)
			}
		}
	}
}

impl Print for Function {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
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

impl Print for i32 {
	fn print(&self, _printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		write!(out, "{self}")
	}
}

impl Print for i64 {
	fn print(&self, _printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		write!(out, "{self}LL")
	}
}

impl Print for f32 {
	fn print(&self, _printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let bits = i32::from_ne_bytes(self.to_ne_bytes());

		write!(out, "{bits}")
	}
}

impl Print for f64 {
	fn print(&self, _printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let bits = i64::from_ne_bytes(self.to_ne_bytes());

		write!(out, "{bits}LL")
	}
}

impl Print for Arc<str> {
	fn print(&self, _printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let escaped = self.as_bytes().escape_ascii();

		write!(out, "\"{escaped}\"")
	}
}

impl Print for Call {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
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
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		write!(out, "{}(", self.name)?;

		fmt_arguments(&self.arguments, printer, out)?;

		write!(out, ")")
	}
}

impl Print for Infix {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { operator, lhs, rhs } = self;

		fmt_infix_operator(lhs, rhs, operator, printer, out)
	}
}

impl Print for Prefix {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { operator, source } = self;

		write!(out, "{operator}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for BooleanToInteger {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		write!(out, "(")?;

		source.print(printer, out)?;

		write!(out, " and 1) or 0")
	}
}

impl Print for RefIsNull {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		write!(out, "(")?;

		source.print(printer, out)?;

		write!(out, ") == nil")
	}
}

impl Print for Aggregate {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
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
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source, index } = self;

		write!(out, "(")?;

		source.print(printer, out)?;

		write!(out, ")[{}]", index + 1)
	}
}

impl Print for Field {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source, name } = self;

		write!(out, "(")?;

		source.print(printer, out)?;

		write!(out, ").{name}")
	}
}

impl Print for Index {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source, offset } = self;

		write!(out, "(")?;

		source.print(printer, out)?;

		write!(out, ")[")?;

		offset.print(printer, out)?;

		write!(out, "]")
	}
}

impl Print for TableNew {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
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

impl Print for MemoryNew {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { initializer, size } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}({{ ")?;

		for (data, offset) in initializer {
			write!(out, "[{offset}] = \"{}\", ", data.escape_ascii())?;
		}

		write!(out, "}}, ")?;

		size.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for Expression {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		stacker::maybe_grow(0x1_0000, 0x10_0000, || match self {
			Self::Function(function) => function.print(printer, out),
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
		})
	}
}
