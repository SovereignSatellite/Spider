use alloc::sync::Arc;
use std::io::{Result, Write};

use luajit_tree::expression::{
	Aggregate, BooleanToInteger, Call, Expression, Extract, Function, GlobalGet, GlobalNew,
	IntegerBinaryOperation, IntegerCompareOperation, IntegerConvertToNumber, IntegerExtend,
	IntegerNarrow, IntegerTransmuteToNumber, IntegerUnaryOperation, IntegerWiden, Local, Location,
	MemoryLoad, MemoryNew, Name, NumberBinaryOperation, NumberCompareOperation, NumberNarrow,
	NumberTransmuteToInteger, NumberTruncateToInteger, NumberUnaryOperation, NumberWiden,
	RefIsNull, RuntimeCall, TableGet, TableGrow, TableNew, TableSize,
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

pub fn fmt_runtime_call(
	name: &str,
	arguments: &[Expression],
	printer: &mut LuaJITPrinter,
	out: &mut dyn Write,
) -> Result<()> {
	write!(out, "rt_{name}(")?;

	fmt_delimited(arguments, printer, out)?;

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

		write!(out, "{bits} --[[ {self}_f32 ]]")
	}
}

impl Print for f64 {
	fn print(&self, _printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let bits = i64::from_ne_bytes(self.to_ne_bytes());

		write!(out, "{bits}LL --[[ {self}_f64 ]]")
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

		fmt_delimited(arguments, printer, out)?;

		write!(out, ")")
	}
}

impl Print for RuntimeCall {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { name, arguments } = self;

		fmt_runtime_call(name, arguments, printer, out)
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

impl Print for IntegerUnaryOperation {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source, .. } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for IntegerBinaryOperation {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { lhs, rhs, .. } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		lhs.print(printer, out)?;

		write!(out, ", ")?;

		rhs.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for IntegerCompareOperation {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { lhs, rhs, .. } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		lhs.print(printer, out)?;

		write!(out, ", ")?;

		rhs.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for IntegerNarrow {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for IntegerWiden {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for IntegerExtend {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source, .. } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for IntegerConvertToNumber {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source, .. } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for IntegerTransmuteToNumber {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source, .. } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for NumberUnaryOperation {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source, .. } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for NumberBinaryOperation {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { lhs, rhs, .. } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		lhs.print(printer, out)?;

		write!(out, ", ")?;

		rhs.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for NumberCompareOperation {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { lhs, rhs, .. } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		lhs.print(printer, out)?;

		write!(out, ", ")?;

		rhs.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for NumberNarrow {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for NumberWiden {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for NumberTruncateToInteger {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source, .. } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for NumberTransmuteToInteger {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source, .. } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for Location {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { reference, offset } = self;

		reference.print(printer, out)?;

		write!(out, ", ")?;

		offset.print(printer, out)
	}
}

impl Print for GlobalNew {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { initializer } = self;

		write!(out, "{{ ")?;

		initializer.print(printer, out)?;

		write!(out, " }}")
	}
}

impl Print for GlobalGet {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		write!(out, "(")?;

		source.print(printer, out)?;

		write!(out, ")[1]")
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

impl Print for TableGet {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for TableSize {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source } = self;

		write!(out, "(")?;

		source.print(printer, out)?;

		write!(out, ").minimum")
	}
}

impl Print for TableGrow {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
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

impl Print for MemoryLoad {
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		let Self { source, .. } = self;

		let intrinsic = self.needs_name();

		write!(out, "rt_{intrinsic}(")?;

		source.print(printer, out)?;

		write!(out, ")")
	}
}

impl Print for Expression {
	#[expect(
		clippy::too_many_lines,
		reason = "exhaustive match over expression variants"
	)]
	fn print(&self, printer: &mut LuaJITPrinter, out: &mut dyn Write) -> Result<()> {
		match self {
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
			Self::RuntimeCall(runtime_call) => runtime_call.print(printer, out),
			Self::BooleanToInteger(boolean_to_integer) => boolean_to_integer.print(printer, out),
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
			Self::Aggregate(aggregate) => aggregate.print(printer, out),
			Self::Extract(extract) => extract.print(printer, out),
			Self::TableNew(table_new) => table_new.print(printer, out),
			Self::TableGet(table_get) => table_get.print(printer, out),
			Self::TableSize(table_size) => table_size.print(printer, out),
			Self::TableGrow(table_grow) => table_grow.print(printer, out),
			Self::MemoryNew(memory_new) => memory_new.print(printer, out),
			Self::MemoryLoad(memory_load) => memory_load.print(printer, out),
		}
	}
}
