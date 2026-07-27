use std::io::{Result, Write};

use luau_tree::{
	expression::Expression,
	statement::{Assign, Call, Match, Repeat, Sequence, SetIndex, Statement, SwapAll},
};

use super::{LuauPrinter, expression::fmt_delimited, print::Print};

mod conditional {
	use core::ops::Range;
	use std::io::{Result, Write};

	use luau_tree::{expression::Expression, statement::Sequence};

	use crate::{LuauPrinter, print::Print as _};

	fn print_recursive(
		branches: &[Sequence],
		condition: &Expression,
		range: Range<usize>,
		printer: &mut LuauPrinter,
		out: &mut dyn Write,
	) -> Result<()> {
		let center = range.start + (range.end - range.start) / 2;
		let has_minimum = range.start != center;
		let has_maximum = range.end != center + 1;

		if has_minimum {
			printer.write_indent(out)?;
			write!(out, "if (")?;

			condition.print(printer, out)?;

			writeln!(out, ") < {center} then")?;

			printer.indent();
			print_recursive(branches, condition, range.start..center, printer, out)?;
			printer.outdent();

			printer.write_indent(out)?;
			write!(out, "else")?;

			if has_maximum {
				write!(out, "if (")?;

				condition.print(printer, out)?;

				writeln!(out, ") > {center} then")?;

				printer.indent();
				print_recursive(branches, condition, (center + 1)..range.end, printer, out)?;
				printer.outdent();

				printer.write_indent(out)?;
				write!(out, "else")?;
			}

			writeln!(out)?;

			printer.indent();
		}

		branches[center].print(printer, out)?;

		if has_minimum {
			printer.outdent();

			printer.write_indent(out)?;
			writeln!(out, "end")
		} else {
			Ok(())
		}
	}

	pub fn print_match(
		branches: &[Sequence],
		condition: &Expression,
		printer: &mut LuauPrinter,
		out: &mut dyn Write,
	) -> Result<()> {
		print_recursive(branches, condition, 0..branches.len(), printer, out)
	}

	fn print_if_true(
		code: &Sequence,
		condition: &Expression,
		printer: &mut LuauPrinter,
		out: &mut dyn Write,
	) -> Result<()> {
		printer.write_indent(out)?;
		write!(out, "if ")?;

		condition.print(printer, out)?;

		writeln!(out, " then")?;

		printer.indent();
		code.print(printer, out)?;
		printer.outdent();

		printer.write_indent(out)?;
		writeln!(out, "end")
	}

	fn print_if_false(
		code: &Sequence,
		condition: &Expression,
		printer: &mut LuauPrinter,
		out: &mut dyn Write,
	) -> Result<()> {
		printer.write_indent(out)?;
		write!(out, "if not (")?;

		condition.print(printer, out)?;

		writeln!(out, ") then")?;

		printer.indent();
		code.print(printer, out)?;
		printer.outdent();

		printer.write_indent(out)?;
		writeln!(out, "end")
	}

	fn print_if_else(
		on_false: &Sequence,
		on_true: &Sequence,
		condition: &Expression,
		printer: &mut LuauPrinter,
		out: &mut dyn Write,
	) -> Result<()> {
		printer.write_indent(out)?;
		write!(out, "if ")?;

		condition.print(printer, out)?;

		writeln!(out, " then")?;

		printer.indent();
		on_true.print(printer, out)?;
		printer.outdent();

		printer.write_indent(out)?;
		writeln!(out, "else")?;

		printer.indent();
		on_false.print(printer, out)?;
		printer.outdent();

		printer.write_indent(out)?;
		writeln!(out, "end")
	}

	pub fn print_if(
		on_false: &Sequence,
		on_true: &Sequence,
		condition: &Expression,
		printer: &mut LuauPrinter,
		out: &mut dyn Write,
	) -> Result<()> {
		let false_empty = on_false.statements.is_empty();
		let true_empty = on_true.statements.is_empty();

		match (false_empty, true_empty) {
			(true, true | false) => print_if_true(on_true, condition, printer, out),
			(false, true) => print_if_false(on_false, condition, printer, out),
			(false, false) => print_if_else(on_false, on_true, condition, printer, out),
		}
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

fn print_repeat_exit_condition(
	condition: &Expression,
	printer: &mut LuauPrinter,
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
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
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
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
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
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
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
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
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
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
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
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
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
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		stacker::maybe_grow(0x1_0000, 0x10_0000, || {
			self.statements
				.iter()
				.try_for_each(|statement| statement.print(printer, out))
		})
	}
}
