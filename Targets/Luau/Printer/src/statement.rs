use std::io::{Result, Write};

use luau_tree::{
	LuauTree,
	expression::Expression,
	statement::{
		Assign, AssignAll, Call, DataDrop, ElementsDrop, Export, FastDefine, GlobalSet, Match,
		MemoryCopy, MemoryFill, MemoryInit, MemoryStore, Repeat, Sequence, SlowDefine, Statement,
		TableCopy, TableFill, TableInit, TableSet,
	},
};

use crate::{LuauPrinter, expression::fmt_delimited, library::NeedsName, print::Print};

impl Print for Match {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
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

		let Self {
			branches,
			condition,
		} = self;

		print_recursive(branches, condition, 0, branches.len(), printer, out)
	}
}

impl Print for Repeat {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			code,
			post,
			condition,
		} = self;

		printer.tab(out)?;
		writeln!(out, "while true do")?;

		printer.indent();
		code.print(printer, out)?;

		printer.tab(out)?;
		write!(out, "if (")?;

		condition.print(printer, out)?;
		writeln!(out, ") == 0 then")?;

		printer.indent();
		post.print(printer, out)?;
		printer.tab(out)?;
		writeln!(out, "break")?;

		printer.outdent();
		printer.tab(out)?;
		writeln!(out, "else")?;

		printer.indent();
		post.print(printer, out)?;

		printer.outdent();
		printer.tab(out)?;
		writeln!(out, "end")?;

		printer.outdent();
		printer.tab(out)?;

		writeln!(out, "end")
	}
}

impl Print for FastDefine {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { name, source } = self;

		printer.tab(out)?;
		write!(out, "local ")?;

		name.print(printer, out)?;

		write!(out, " = ")?;

		source.print(printer, out)?;

		writeln!(out, ";")
	}
}

impl Print for SlowDefine {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { name, len } = self;

		printer.tab(out)?;
		write!(out, "local ")?;

		name.print(printer, out)?;

		writeln!(out, " = table.create({len});")
	}
}

impl Print for Assign {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { local, source } = self;

		printer.tab(out)?;
		local.print(printer, out)?;

		write!(out, " = ")?;

		source.print(printer, out)?;

		writeln!(out, ";")
	}
}

impl Print for AssignAll {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { assignments } = self;

		if assignments.is_empty() {
			return Ok(());
		}

		let locals = assignments.iter().map(|item| item.0);
		let sources = assignments.iter().map(|item| &item.1);

		printer.tab(out)?;
		fmt_delimited(locals, printer, out)?;

		write!(out, " = ")?;

		fmt_delimited(sources, printer, out)?;

		writeln!(out, ";")
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

impl Print for TableInit {
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

impl Print for ElementsDrop {
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
			r#type: _,
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

impl Print for MemoryInit {
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

impl Print for DataDrop {
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
			Self::Match(r#match) => r#match.print(printer, out),
			Self::Repeat(repeat) => repeat.print(printer, out),
			Self::FastDefine(fast_define) => fast_define.print(printer, out),
			Self::SlowDefine(slow_define) => slow_define.print(printer, out),
			Self::Assign(assign) => assign.print(printer, out),
			Self::AssignAll(assign_all) => assign_all.print(printer, out),
			Self::Call(call) => call.print(printer, out),
			Self::GlobalSet(global_set) => global_set.print(printer, out),
			Self::TableSet(table_set) => table_set.print(printer, out),
			Self::TableFill(table_fill) => table_fill.print(printer, out),
			Self::TableCopy(table_copy) => table_copy.print(printer, out),
			Self::TableInit(table_init) => table_init.print(printer, out),
			Self::ElementsDrop(elements_drop) => elements_drop.print(printer, out),
			Self::MemoryStore(memory_store) => memory_store.print(printer, out),
			Self::MemoryFill(memory_fill) => memory_fill.print(printer, out),
			Self::MemoryCopy(memory_copy) => memory_copy.print(printer, out),
			Self::MemoryInit(memory_init) => memory_init.print(printer, out),
			Self::DataDrop(data_drop) => data_drop.print(printer, out),
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
	writeln!(out, "return {{")?;

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
			code,
			exports,
		} = self;

		printer.tab(out)?;
		write!(out, "local function module(")?;
		environment.print(printer, out)?;
		writeln!(out, ")")?;

		printer.indent();
		code.print(printer, out)?;

		fmt_export_list(exports, printer, out)?;
		printer.outdent();

		printer.tab(out)?;
		writeln!(out, "end")
	}
}
