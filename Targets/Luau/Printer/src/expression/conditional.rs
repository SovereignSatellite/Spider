use std::io::{Result, Write};

use luau_tree::expression::Expression;

use crate::{LuauPrinter, print::Print as _};

fn print_match_tree(
	branches: &[Expression],
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

	write!(out, "if (")?;
	condition.print(printer, out)?;
	write!(out, ") < {threshold} then ")?;

	print_match_tree(lower_branches, condition, branch_offset, printer, out)?;

	write!(out, " else ")?;

	print_match_tree(upper_branches, condition, threshold, printer, out)
}

pub fn print_match(
	branches: &[Expression],
	condition: &Expression,
	printer: &mut LuauPrinter,
	out: &mut dyn Write,
) -> Result<()> {
	print_match_tree(branches, condition, 0, printer, out)
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
