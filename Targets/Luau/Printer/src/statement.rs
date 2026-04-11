use std::io::{Result, Write};

use luau_tree::{
	LuauTree,
	statement::{
		Assign, Call, Export, GlobalSet, Match, MemoryCopy, MemoryDrop, MemoryFill, MemoryStore,
		Repeat, Sequence, Statement, SwapAll, TableCopy, TableDrop, TableFill, TableSet,
	},
};

use super::{
	LuauPrinter,
	expression::{fmt_delimited, fmt_stack_enter, fmt_stack_leave},
	library::NeedsName as _,
	print::Print,
};

mod conditional {
	use std::io::{Result, Write};

	use luau_tree::{expression::Expression, statement::Sequence};

	use crate::{LuauPrinter, print::Print as _};

	fn print_recursive(
		branches: &[Sequence],
		condition: &Expression,
		start: usize,
		end: usize,
		printer: &mut LuauPrinter,
		out: &mut dyn Write,
	) -> Result<()> {
		let center = start + (end - start) / 2;
		let has_minimum = start != center;
		let has_maximum = end != center + 1;

		if has_minimum {
			printer.tab(out)?;
			write!(out, "if (")?;

			condition.print(printer, out)?;

			writeln!(out, ") < {center} then")?;

			printer.indent();
			print_recursive(branches, condition, start, center, printer, out)?;
			printer.outdent();

			printer.tab(out)?;
			write!(out, "else")?;

			if has_maximum {
				write!(out, "if (")?;

				condition.print(printer, out)?;

				writeln!(out, ") > {center} then")?;

				printer.indent();
				print_recursive(branches, condition, center + 1, end, printer, out)?;
				printer.outdent();

				printer.tab(out)?;
				write!(out, "else")?;
			}

			writeln!(out)?;

			printer.indent();
		}

		branches[center].print(printer, out)?;

		if has_minimum {
			printer.outdent();

			printer.tab(out)?;
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
		print_recursive(branches, condition, 0, branches.len(), printer, out)
	}

	fn print_if_true(
		code: &Sequence,
		condition: &Expression,
		printer: &mut LuauPrinter,
		out: &mut dyn Write,
	) -> Result<()> {
		printer.tab(out)?;
		write!(out, "if ")?;

		condition.print(printer, out)?;

		writeln!(out, " then")?;

		printer.indent();
		code.print(printer, out)?;
		printer.outdent();

		printer.tab(out)?;
		writeln!(out, "end")
	}

	fn print_if_false(
		code: &Sequence,
		condition: &Expression,
		printer: &mut LuauPrinter,
		out: &mut dyn Write,
	) -> Result<()> {
		printer.tab(out)?;
		write!(out, "if not (")?;

		condition.print(printer, out)?;

		writeln!(out, ") then")?;

		printer.indent();
		code.print(printer, out)?;
		printer.outdent();

		printer.tab(out)?;
		writeln!(out, "end")
	}

	fn print_if_else(
		on_false: &Sequence,
		on_true: &Sequence,
		condition: &Expression,
		printer: &mut LuauPrinter,
		out: &mut dyn Write,
	) -> Result<()> {
		printer.tab(out)?;
		write!(out, "if ")?;

		condition.print(printer, out)?;

		writeln!(out, " then")?;

		printer.indent();
		on_true.print(printer, out)?;
		printer.outdent();

		printer.tab(out)?;
		writeln!(out, "else")?;

		printer.indent();
		on_false.print(printer, out)?;
		printer.outdent();

		printer.tab(out)?;
		writeln!(out, "end")
	}

	pub fn print_if(
		on_false: &Sequence,
		on_true: &Sequence,
		condition: &Expression,
		printer: &mut LuauPrinter,
		out: &mut dyn Write,
	) -> Result<()> {
		let false_empty = on_false.list.is_empty();
		let true_empty = on_true.list.is_empty();

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

impl Print for Repeat {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { code, condition } = self;

		printer.tab(out)?;
		writeln!(out, "repeat")?;

		printer.indent();
		code.print(printer, out)?;
		printer.outdent();

		printer.tab(out)?;
		write!(out, "until (")?;
		condition.print(printer, out)?;
		writeln!(out, ") == 0")
	}
}

impl Print for Assign {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			destination,
			source,
		} = self;

		printer.tab(out)?;
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
			printer.tab(out)?;

			pair[0].print(printer, out)?;

			write!(out, ", ")?;

			pair[1].print(printer, out)?;

			write!(out, " = ")?;

			pair[1].print(printer, out)?;

			write!(out, ", ")?;

			pair[0].print(printer, out)?;

			writeln!(out, ";")?;
		}

		Ok(())
	}
}

impl Print for Call {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			function,
			results,
			arguments,
		} = self;

		printer.tab(out)?;

		if !results.is_empty() {
			fmt_delimited(results, printer, out)?;

			write!(out, " = ")?;
		}

		function.print(printer, out)?;

		write!(out, "(")?;

		fmt_delimited(arguments, printer, out)?;

		writeln!(out, ")")
	}
}

impl Print for GlobalSet {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			destination,
			source,
		} = self;

		printer.tab(out)?;
		destination.print(printer, out)?;

		write!(out, "[1] = ")?;

		source.print(printer, out)?;

		writeln!(out, ";")
	}
}

impl Print for TableSet {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			destination,
			source,
		} = self;

		let intrinsic = self.needs_name();

		printer.tab(out)?;
		write!(out, "rt_{intrinsic}(")?;

		destination.print(printer, out)?;

		write!(out, ", ")?;

		source.print(printer, out)?;

		writeln!(out, ");")
	}
}

impl Print for TableFill {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			destination,
			source,
			size,
		} = self;

		let intrinsic = self.needs_name();

		printer.tab(out)?;
		write!(out, "rt_{intrinsic}(")?;

		destination.print(printer, out)?;

		write!(out, ", ")?;

		source.print(printer, out)?;

		write!(out, ", ")?;

		size.print(printer, out)?;

		writeln!(out, ")")
	}
}

impl Print for TableCopy {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			destination,
			source,
			size,
		} = self;

		let intrinsic = self.needs_name();

		printer.tab(out)?;
		write!(out, "rt_{intrinsic}(")?;

		destination.print(printer, out)?;

		write!(out, ", ")?;

		source.print(printer, out)?;

		write!(out, ", ")?;

		size.print(printer, out)?;

		writeln!(out, ")")
	}
}

impl Print for TableDrop {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		let intrinsic = self.needs_name();

		printer.tab(out)?;
		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		writeln!(out, ")")
	}
}

impl Print for MemoryStore {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			destination,
			source,
			..
		} = self;

		let intrinsic = self.needs_name();

		printer.tab(out)?;
		write!(out, "rt_{intrinsic}(")?;

		destination.print(printer, out)?;

		write!(out, ", ")?;

		source.print(printer, out)?;

		writeln!(out, ")")
	}
}

impl Print for MemoryFill {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			destination,
			byte,
			size,
		} = self;

		let intrinsic = self.needs_name();

		printer.tab(out)?;
		write!(out, "rt_{intrinsic}(")?;

		destination.print(printer, out)?;

		write!(out, ", ")?;

		byte.print(printer, out)?;

		write!(out, ", ")?;

		size.print(printer, out)?;

		writeln!(out, ")")
	}
}

impl Print for MemoryCopy {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			destination,
			source,
			size,
		} = self;

		let intrinsic = self.needs_name();

		printer.tab(out)?;
		write!(out, "rt_{intrinsic}(")?;

		destination.print(printer, out)?;

		write!(out, ", ")?;

		source.print(printer, out)?;

		write!(out, ", ")?;

		size.print(printer, out)?;

		writeln!(out, ")")
	}
}

impl Print for MemoryDrop {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		let intrinsic = self.needs_name();

		printer.tab(out)?;
		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		writeln!(out, ")")
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
			Self::GlobalSet(global_set) => global_set.print(printer, out),
			Self::TableSet(table_set) => table_set.print(printer, out),
			Self::TableFill(table_fill) => table_fill.print(printer, out),
			Self::TableCopy(table_copy) => table_copy.print(printer, out),
			Self::TableDrop(elements_drop) => elements_drop.print(printer, out),
			Self::MemoryStore(memory_store) => memory_store.print(printer, out),
			Self::MemoryFill(memory_fill) => memory_fill.print(printer, out),
			Self::MemoryCopy(memory_copy) => memory_copy.print(printer, out),
			Self::MemoryDrop(data_drop) => data_drop.print(printer, out),
		}
	}
}

impl Print for Sequence {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		self.list
			.iter()
			.try_for_each(|statement| statement.print(printer, out))
	}
}

impl Print for Export {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { identifier, source } = self;

		write!(out, "[\"{}\"] = ", identifier.as_bytes().escape_ascii())?;

		source.print(printer, out)
	}
}

fn fmt_export_list(
	exports: &[Export],
	printer: &mut LuauPrinter,
	out: &mut dyn Write,
) -> Result<()> {
	printer.tab(out)?;
	writeln!(out, "local export = {{")?;

	printer.indent();

	exports.iter().try_for_each(|export| {
		printer.tab(out)?;
		export.print(printer, out)?;

		writeln!(out, ",")
	})?;

	printer.outdent();

	printer.tab(out)?;
	writeln!(out, "}}")
}

impl Print for LuauTree {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			environment,
			stack,
			code,
			exports,
		} = self;

		printer.tab(out)?;
		write!(out, "local function module(")?;

		environment.print(printer, out)?;

		writeln!(out, ")")?;

		printer.indent();

		printer.tab(out)?;
		writeln!(out, "local excess_stack = {{ top = 0 }}")?;

		fmt_stack_enter(*stack, printer, out)?;

		code.print(printer, out)?;

		fmt_export_list(exports, printer, out)?;
		fmt_stack_leave(*stack, printer, out)?;

		printer.tab(out)?;
		writeln!(out, "return export")?;

		printer.outdent();

		printer.tab(out)?;
		writeln!(out, "end")
	}
}
