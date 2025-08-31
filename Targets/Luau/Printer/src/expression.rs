use std::io::{Result, Write};

use luau_tree::expression::{
	Call, DataNew, ElementsNew, Expression, Function, GlobalGet, GlobalNew, Import,
	IntegerBinaryOperation, IntegerCompareOperation, IntegerConvertToNumber, IntegerExtend,
	IntegerNarrow, IntegerTransmuteToNumber, IntegerUnaryOperation, IntegerWiden, Local, Location,
	Match, MemoryGrow, MemoryLoad, MemoryNew, MemorySize, Name, NumberBinaryOperation,
	NumberBinaryOperator, NumberCompareOperation, NumberCompareOperator, NumberNarrow,
	NumberTransmuteToInteger, NumberTruncateToInteger, NumberType, NumberUnaryOperation,
	NumberUnaryOperator, NumberWiden, RefIsNull, Scoped, TableGet, TableGrow, TableNew, TableSize,
};

use crate::{library::NeedsName, print::Print, LuauPrinter};

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

impl Print for Name {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { id } = self;
		let prefix = printer.get_name(*self).unwrap_or("loc");

		write!(out, "{prefix}_{id}_")
	}
}

impl Print for Local {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		match self {
			Self::Fast { name } => name.print(printer, out),
			Self::Slow { table, index } => {
				table.print(printer, out)?;

				write!(out, "[{}]", index + 1)
			}
		}
	}
}

impl Print for Function {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			arguments,
			code,
			returns,
		} = self;

		write!(out, "(function(")?;

		fmt_delimited(arguments, printer, out)?;

		writeln!(out, ")")?;

		printer.indent();
		code.print(printer, out)?;

		if !returns.is_empty() {
			printer.tab(out)?;
			write!(out, "return ")?;

			fmt_delimited(returns, printer, out)?;

			writeln!(out)?;
		}

		printer.outdent();

		printer.tab(out)?;
		write!(out, "end)")
	}
}

impl Print for Scoped {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { locals, function } = self;

		if locals.is_empty() {
			return function.print(printer, out);
		}

		writeln!(out, "(function()")?;

		printer.indent();

		for local in locals {
			local.print(printer, out)?;
		}

		printer.tab(out)?;
		write!(out, "return ")?;
		function.print(printer, out)?;
		writeln!(out)?;

		printer.outdent();
		printer.tab(out)?;
		write!(out, "end)()")
	}
}

impl Print for Match {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		fn print_recursive(
			branches: &[Expression],
			condition: &Expression,
			start: usize,
			end: usize,
			printer: &mut LuauPrinter,
			out: &mut dyn Write,
		) -> Result<()> {
			let center = start + (end - start) / 2;

			if start != center {
				write!(out, "if (")?;

				condition.print(printer, out)?;

				write!(out, ") < {center} then ")?;

				print_recursive(branches, condition, start, center, printer, out)?;

				write!(out, " else")?;

				if end != center + 1 {
					write!(out, "if (")?;

					condition.print(printer, out)?;

					write!(out, ") > {center} then ")?;

					print_recursive(branches, condition, center + 1, end, printer, out)?;

					write!(out, " else")?;
				}

				write!(out, " ")?;
			}

			branches[center].print(printer, out)
		}

		let Self {
			condition,
			branches,
		} = self;

		print_recursive(branches, condition, 0, branches.len(), printer, out)
	}
}

impl Print for Import {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			environment,
			namespace,
			identifier,
		} = self;

		write!(out, "assert(")?;

		environment.print(printer, out)?;

		let namespace = namespace.as_bytes().escape_ascii();
		let identifier = identifier.as_bytes().escape_ascii();

		write!(
			out,
			"[\"{namespace}\"][\"{identifier}\"], '`{namespace}.{identifier}` should be present')"
		)
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

		let source_1 = u32::from_le_bytes([b1, b2, b3, b4]);
		let source_2 = u32::from_le_bytes([b5, b6, b7, b8]);

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(0x{source_1:08X}, 0x{source_2:08X})")
	}
}

impl Print for f32 {
	fn print(&self, _printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let intrinsic = self.needs_name();

		if self.is_finite() {
			write!(out, "{intrinsic}({self:e}, 0, 0)")
		} else {
			let bits = self.to_bits();

			write!(out, "rt_{intrinsic}(0x{bits:08X})")
		}
	}
}

impl Print for f64 {
	fn print(&self, _printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		if self.is_finite() {
			write!(out, "{self:e}")
		} else {
			let [b1, b2, b3, b4, b5, b6, b7, b8] = self.to_le_bytes();

			let source_1 = u32::from_le_bytes([b1, b2, b3, b4]);
			let source_2 = u32::from_le_bytes([b5, b6, b7, b8]);

			let intrinsic = self.needs_name();

			write!(out, "rt_{intrinsic}(0x{source_1:08X}, 0x{source_2:08X})")
		}
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

		fmt_delimited(arguments, printer, out)?;

		write!(out, ")")
	}
}

impl Print for RefIsNull {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		write!(out, "(if ")?;

		source.print(printer, out)?;

		write!(out, " == nil then 1 else 0)")
	}
}

impl Print for IntegerUnaryOperation {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			source,
			r#type: _,
			operator: _,
		} = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for IntegerBinaryOperation {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		lhs.print(printer, out)?;

		write!(out, ", ")?;

		rhs.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for IntegerCompareOperation {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			lhs,
			rhs,
			r#type: _,
			operator: _,
		} = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		lhs.print(printer, out)?;

		write!(out, ", ")?;

		rhs.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for IntegerNarrow {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for IntegerWiden {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for IntegerExtend {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source, r#type: _ } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for IntegerConvertToNumber {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			source,
			signed: _,
			to: _,
			from: _,
		} = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for IntegerTransmuteToNumber {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source, from: _ } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for NumberUnaryOperation {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			source,
			r#type: _,
			operator,
		} = self;

		if *operator == NumberUnaryOperator::Negate {
			write!(out, "(-")?;

			source.print(printer, out)?;

			return write!(out, ")");
		}

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for NumberBinaryOperation {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		fn fmt_infix(
			lhs: &Expression,
			rhs: &Expression,
			operator: char,
			printer: &mut LuauPrinter,
			out: &mut dyn Write,
		) -> Result<()> {
			write!(out, "(")?;

			lhs.print(printer, out)?;

			write!(out, " {operator} ")?;

			rhs.print(printer, out)?;

			write!(out, ")")
		}

		let Self {
			lhs,
			rhs,
			r#type: _,
			operator,
		} = self;

		if let Some(operator) = match operator {
			NumberBinaryOperator::Add => Some('+'),
			NumberBinaryOperator::Subtract => Some('-'),
			NumberBinaryOperator::Multiply => Some('*'),
			NumberBinaryOperator::Divide => Some('/'),

			_ => None,
		} {
			return fmt_infix(lhs, rhs, operator, printer, out);
		}

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		lhs.print(printer, out)?;

		write!(out, ", ")?;

		rhs.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for NumberCompareOperation {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		fn fmt_infix(
			lhs: &Expression,
			rhs: &Expression,
			operator: &str,
			printer: &mut LuauPrinter,
			out: &mut dyn Write,
		) -> Result<()> {
			write!(out, "(if ")?;

			lhs.print(printer, out)?;

			write!(out, " {operator} ")?;

			rhs.print(printer, out)?;

			write!(out, " then 1 else 0)")
		}

		let Self {
			lhs,
			rhs,
			r#type,
			operator,
		} = self;

		if *r#type == NumberType::F64 {
			let operator = match operator {
				NumberCompareOperator::Equal => "==",
				NumberCompareOperator::NotEqual => "~=",
				NumberCompareOperator::LessThan => "<",
				NumberCompareOperator::GreaterThan => ">",
				NumberCompareOperator::LessThanEqual => "<=",
				NumberCompareOperator::GreaterThanEqual => ">=",
			};

			return fmt_infix(lhs, rhs, operator, printer, out);
		}

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		lhs.print(printer, out)?;

		write!(out, ", ")?;

		rhs.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for NumberNarrow {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for NumberWiden {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for NumberTruncateToInteger {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			source,
			signed: _,
			saturate: _,
			to: _,
			from: _,
		} = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for NumberTransmuteToInteger {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source, from: _ } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for Location {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { reference, offset } = self;

		reference.print(printer, out)?;

		write!(out, ", ")?;

		offset.print(printer, out)
	}
}

impl Print for GlobalNew {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { initializer } = self;

		write!(out, "{{ ")?;

		initializer.print(printer, out)?;

		write!(out, " }}")
	}
}

impl Print for GlobalGet {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		source.print(printer, out)?;

		write!(out, "[1]")
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

		write!(out, "rt_{intrinsic}(")?;

		initializer.print(printer, out)?;

		write!(out, ", {minimum}, {maximum})")
	}
}

impl Print for TableGet {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for TableSize {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		source.print(printer, out)?;

		write!(out, ".minimum")
	}
}

impl Print for TableGrow {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self {
			destination,
			initializer,
			size,
		} = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		destination.print(printer, out)?;

		write!(out, ", ")?;

		initializer.print(printer, out)?;

		write!(out, ", ")?;

		size.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for ElementsNew {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { content } = self;

		write!(out, "{{ ")?;

		fmt_delimited(content, printer, out)?;

		let count = content.len();

		if count != 0 {
			write!(out, ", ")?;
		}

		write!(out, "count = {count} }}")
	}
}

impl Print for MemoryNew {
	fn print(&self, _printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { minimum, maximum } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}({minimum}, {maximum})")
	}
}

impl Print for MemoryLoad {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source, r#type: _ } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for MemorySize {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for MemoryGrow {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { destination, size } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		destination.print(printer, out)?;

		write!(out, ", ")?;

		size.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for DataNew {
	fn print(&self, _printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { content } = self;

		write!(
			out,
			"{{ buffer.fromstring(\"{}\") }}",
			content.escape_ascii()
		)
	}
}

impl Print for Expression {
	fn print(&self, printer: &mut LuauPrinter, out: &mut dyn Write) -> Result<()> {
		match self {
			Self::Function(function) => function.print(printer, out),
			Self::Scoped(scoped) => scoped.print(printer, out),
			Self::Match(r#match) => r#match.print(printer, out),
			Self::Import(import) => import.print(printer, out),
			Self::Trap => write!(out, "error('unreachable code')"),
			Self::Null => write!(out, "nil"),
			Self::Local(local) => local.print(printer, out),
			Self::I32(i32) => i32.print(printer, out),
			Self::I64(i64) => i64.print(printer, out),
			Self::F32(f32) => f32.print(printer, out),
			Self::F64(f64) => f64.print(printer, out),
			Self::Call(call) => call.print(printer, out),
			Self::RefIsNull(ref_is_null) => ref_is_null.print(printer, out),
			Self::IntegerUnaryOperation(integer_unary_operation) => {
				integer_unary_operation.print(printer, out)
			}
			Self::IntegerBinaryOperation(integer_binary_operation) => {
				integer_binary_operation.print(printer, out)
			}
			Self::IntegerCompareOperation(integer_compare_operation) => {
				integer_compare_operation.print(printer, out)
			}
			Self::IntegerNarrow(integer_narrow) => integer_narrow.print(printer, out),
			Self::IntegerWiden(integer_widen) => integer_widen.print(printer, out),
			Self::IntegerExtend(integer_extend) => integer_extend.print(printer, out),
			Self::IntegerConvertToNumber(integer_convert_to_number) => {
				integer_convert_to_number.print(printer, out)
			}
			Self::IntegerTransmuteToNumber(integer_transmute_to_number) => {
				integer_transmute_to_number.print(printer, out)
			}
			Self::NumberUnaryOperation(number_unary_operation) => {
				number_unary_operation.print(printer, out)
			}
			Self::NumberBinaryOperation(number_binary_operation) => {
				number_binary_operation.print(printer, out)
			}
			Self::NumberCompareOperation(number_compare_operation) => {
				number_compare_operation.print(printer, out)
			}
			Self::NumberNarrow(number_narrow) => number_narrow.print(printer, out),
			Self::NumberWiden(number_widen) => number_widen.print(printer, out),
			Self::NumberTruncateToInteger(number_truncate_to_integer) => {
				number_truncate_to_integer.print(printer, out)
			}
			Self::NumberTransmuteToInteger(number_transmute_to_integer) => {
				number_transmute_to_integer.print(printer, out)
			}
			Self::GlobalNew(global_new) => global_new.print(printer, out),
			Self::GlobalGet(global_get) => global_get.print(printer, out),
			Self::TableNew(table_new) => table_new.print(printer, out),
			Self::TableGet(table_get) => table_get.print(printer, out),
			Self::TableSize(table_size) => table_size.print(printer, out),
			Self::TableGrow(table_grow) => table_grow.print(printer, out),
			Self::ElementsNew(elements_new) => elements_new.print(printer, out),
			Self::MemoryNew(memory_new) => memory_new.print(printer, out),
			Self::MemoryLoad(memory_load) => memory_load.print(printer, out),
			Self::MemorySize(memory_size) => memory_size.print(printer, out),
			Self::MemoryGrow(memory_grow) => memory_grow.print(printer, out),
			Self::DataNew(data_new) => data_new.print(printer, out),
		}
	}
}
