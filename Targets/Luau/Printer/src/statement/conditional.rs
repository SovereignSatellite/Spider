use std::io::{Result, Write};

use luau_tree::{expression::Expression, statement::Sequence};

use crate::{LuauPrinter, print::Print as _};

fn print_match_tree(
	branches: &[Sequence],
	condition: &Expression,
	branch_offset: usize,
	printer: &mut LuauPrinter,
	out: &mut dyn Write,
) -> Result<()> {
	if branches.len() == 1 {
		return branches[0].print(printer, out);
	}

	let split_index = branches.len() / 2;
	let threshold = branch_offset + split_index;
	let (lower_branches, upper_branches) = branches.split_at(split_index);

	printer.write_indent(out)?;
	write!(out, "if (")?;
	condition.print(printer, out)?;
	writeln!(out, ") < {threshold} then")?;

	printer.indent();
	print_match_tree(lower_branches, condition, branch_offset, printer, out)?;
	printer.outdent();

	printer.write_indent(out)?;
	writeln!(out, "else")?;

	printer.indent();
	print_match_tree(upper_branches, condition, threshold, printer, out)?;
	printer.outdent();

	printer.write_indent(out)?;
	writeln!(out, "end")
}

pub fn print_match(
	branches: &[Sequence],
	condition: &Expression,
	printer: &mut LuauPrinter,
	out: &mut dyn Write,
) -> Result<()> {
	print_match_tree(branches, condition, 0, printer, out)
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
