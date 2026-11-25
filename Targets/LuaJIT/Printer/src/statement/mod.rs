use std::io::{Result, Write};

use luajit_tree::{
	expression::Expression,
	statement::{Assign, Call, Match, Repeat, Sequence, SetIndex, Statement, SwapAll},
};

use super::{LuaJITPrinter, expression::fmt_delimited, print::Print};

mod conditional;

impl Print for Match {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
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

fn print_repeat_exit_condition(
	condition: &Expression,
	printer: &mut LuaJITPrinter,
	out: &mut dyn Write,
) -> Result<()> {
	if let Expression::BooleanToInteger(conversion) = condition {
		write!(out, "not (")?;
		conversion.source.print(printer, out)?;
		write!(out, ")")
	} else {
		write!(out, "(")?;
		condition.print(printer, out)?;
		write!(out, ") == 0")
	}
}

impl Print for Repeat {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			code,
			condition,
			rotation,
		} = self;

		printer.write_indent(out)?;
		writeln!(out, "while true do")?;

		printer.indent();
		code.print(printer, out)?;

		printer.write_indent(out)?;
		write!(out, "if ")?;
		print_repeat_exit_condition(condition, printer, out)?;
		writeln!(out, " then break end")?;

		rotation.print(printer, out)?;
		printer.outdent();

		printer.write_indent(out)?;
		writeln!(out, "end")
	}
}

impl Print for Assign {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			destination,
			source,
		} = self;

		printer.write_indent(out)?;
		destination.print(printer, out)?;

		write!(out, " = ")?;

		source.print(printer, out)?;

		writeln!(out, ";")
	}
}

impl Print for SwapAll {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { locals } = self;

		for pair in locals.windows(2) {
			printer.write_indent(out)?;

			fmt_delimited([&pair[0], &pair[1]], printer, out)?;

			write!(out, " = ")?;

			fmt_delimited([&pair[1], &pair[0]], printer, out)?;

			writeln!(out, ";")?;
		}

		Ok(())
	}
}

impl Print for Call {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { results, call } = self;

		printer.write_indent(out)?;

		if !results.is_empty() {
			fmt_delimited(results, printer, out)?;

			write!(out, " = ")?;
		}

		call.print(printer, out)?;

		writeln!(out, ";")
	}
}

impl Print for SetIndex {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			table,
			offset,
			value,
		} = self;

		printer.write_indent(out)?;
		write!(out, "(")?;

		table.print(printer, out)?;

		write!(out, ")[")?;

		offset.print(printer, out)?;

		write!(out, "] = ")?;

		value.print(printer, out)?;

		writeln!(out, ";")
	}
}

impl Print for Statement {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		match self {
			Self::Match(inner) => inner.print(printer, out),
			Self::Repeat(repeat) => repeat.print(printer, out),
			Self::Assign(assign) => assign.print(printer, out),
			Self::SwapAll(swap_all) => swap_all.print(printer, out),
			Self::Call(call) => call.print(printer, out),
			Self::SetIndex(set_index) => set_index.print(printer, out),
		}
	}
}

impl Print for Sequence {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		stacker::maybe_grow(0x1_0000, 0x10_0000, || {
			self.statements
				.iter()
				.try_for_each(|statement| statement.print(printer, out))
		})
	}
}
