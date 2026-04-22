use std::io::{Error, Result, Write};

use ir_graph::{
	Node,
	operation::{ExtendType, IntegerSignExtend, integer, number},
};

#[must_use]
#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
pub fn get_static(node: &Node) -> Option<&'static str> {
	let name = match node {
		Node::Function(_) => "Function",
		Node::Match(_) => "Match",
		Node::Repeat(_) => "Repeat",

		Node::ModuleArguments(_) => "Module Arguments",
		Node::ModuleResults(_) => "Module Results",
		Node::FunctionCaptures(_) => "Function Captures",
		Node::FunctionArguments(_) => "Function Arguments",
		Node::FunctionResults(_) => "Function Results",
		Node::BranchArguments(_) => "Branch Arguments",
		Node::BranchResults(_) => "Branch Results",
		Node::RepeatArguments(_) => "Repeat Arguments",
		Node::RepeatResults(_) => "Repeat Results",

		Node::I32(_)
		| Node::I64(_)
		| Node::F32(_)
		| Node::F64(_)
		| Node::IntegerUnaryOperation(_)
		| Node::IntegerBinaryOperation(_)
		| Node::IntegerCompareOperation(_)
		| Node::IntegerSignExtend(_)
		| Node::NumberUnaryOperation(_)
		| Node::NumberBinaryOperation(_)
		| Node::NumberCompareOperation(_) => return None,

		Node::Foreign(foreign) => foreign.identifier(),
		Node::Trap => "Trap",
		Node::Null => "Null",
		Node::Identity(_) => "Identity",
		Node::Fence(_) => "Fence",
		Node::Apply(_) => "Apply",
		Node::RefIsNull(_) => "Ref Is Null",
		Node::IntegerNarrow(_) => "Integer Narrow",
		Node::IntegerWiden(_) => "Integer Widen",
		Node::IntegerConvertToNumber(_) => "Convert To Number",
		Node::IntegerTransmuteToNumber(_) => "Transmute To Number",
		Node::NumberNarrow(_) => "Number Narrow",
		Node::NumberWiden(_) => "Number Widen",
		Node::NumberTruncateToInteger(_) => "Truncate To Integer",
		Node::NumberTransmuteToInteger(_) => "Transmute To Integer",
		Node::MutableNew(_) => "Mutable New",
		Node::MutableGet(_) => "Mutable Get",
		Node::MutableSet(_) => "Mutable Set",
		Node::TableNew(_) => "Table New",
		Node::TableGet(_) => "Table Get",
		Node::TableSet(_) => "Table Set",
		Node::TableSize(_) => "Table Size",
		Node::TableGrow(_) => "Table Grow",
		Node::TableFill(_) => "Table Fill",
		Node::TableCopy(_) => "Table Copy",
		Node::TableDrop(_) => "Table Drop",
		Node::MemoryNew(_) => "Memory New",
		Node::MemoryLoad(_) => "Memory Load",
		Node::MemoryStore(_) => "Memory Store",
		Node::MemorySize(_) => "Memory Size",
		Node::MemoryGrow(_) => "Memory Grow",
		Node::MemoryFill(_) => "Memory Fill",
		Node::MemoryCopy(_) => "Memory Copy",
		Node::MemoryDrop(_) => "Memory Drop",
	};

	Some(name)
}

const fn extend_type_name(kind: ExtendType) -> &'static str {
	match kind {
		ExtendType::I32_S8 => "S8 to I32",
		ExtendType::I32_S16 => "S16 to I32",
		ExtendType::I64_S8 => "S8 to I64",
		ExtendType::I64_S16 => "S16 to I64",
		ExtendType::I64_S32 => "S32 to I64",
	}
}

fn write_integer_sign_extend(node: IntegerSignExtend, out: &mut dyn Write) -> Result<()> {
	write!(out, "Sign Extend {}", extend_type_name(node.kind))
}

const fn integer_type_name(kind: integer::Type) -> &'static str {
	match kind {
		integer::Type::I32 => "I32",
		integer::Type::I64 => "I64",
	}
}

const fn integer_unary_operator_name(operator: integer::UnaryOperator) -> &'static str {
	match operator {
		integer::UnaryOperator::CountOnes => "Count Ones",
		integer::UnaryOperator::LeadingZeroes => "Leading Zeroes",
		integer::UnaryOperator::TrailingZeroes => "Trailing Zeroes",
	}
}

fn write_integer_unary_operation(node: integer::UnaryOperation, out: &mut dyn Write) -> Result<()> {
	write!(
		out,
		"{} {}",
		integer_type_name(node.kind),
		integer_unary_operator_name(node.operator)
	)
}

const fn integer_binary_operator_name(operator: integer::BinaryOperator) -> &'static str {
	match operator {
		integer::BinaryOperator::Add => "+",
		integer::BinaryOperator::Subtract => "-",
		integer::BinaryOperator::Multiply => "*",
		integer::BinaryOperator::Divide { signed: false } => "u/",
		integer::BinaryOperator::Divide { signed: true } => "s/",
		integer::BinaryOperator::Remainder { signed: false } => "u%",
		integer::BinaryOperator::Remainder { signed: true } => "s%",
		integer::BinaryOperator::And => "&",
		integer::BinaryOperator::Or => "|",
		integer::BinaryOperator::ExclusiveOr => "^",
		integer::BinaryOperator::ShiftLeft => "<<",
		integer::BinaryOperator::ShiftRight { signed: false } => "u>>",
		integer::BinaryOperator::ShiftRight { signed: true } => "s>>",
		integer::BinaryOperator::RotateLeft => "^<<",
		integer::BinaryOperator::RotateRight => ">>^",
	}
}

fn write_integer_binary_operation(
	node: integer::BinaryOperation,
	out: &mut dyn Write,
) -> Result<()> {
	write!(
		out,
		"{} {}",
		integer_type_name(node.kind),
		integer_binary_operator_name(node.operator)
	)
}

const fn integer_compare_operator_name(operator: integer::CompareOperator) -> &'static str {
	match operator {
		integer::CompareOperator::Equal => "==",
		integer::CompareOperator::NotEqual => "!=",
		integer::CompareOperator::LessThan { signed: false } => "u<",
		integer::CompareOperator::LessThan { signed: true } => "s<",
		integer::CompareOperator::GreaterThan { signed: false } => "u>",
		integer::CompareOperator::GreaterThan { signed: true } => "s>",
		integer::CompareOperator::LessThanEqual { signed: false } => "u<=",
		integer::CompareOperator::LessThanEqual { signed: true } => "s<=",
		integer::CompareOperator::GreaterThanEqual { signed: false } => "u>=",
		integer::CompareOperator::GreaterThanEqual { signed: true } => "s>=",
	}
}

fn write_integer_compare_operation(
	node: integer::CompareOperation,
	out: &mut dyn Write,
) -> Result<()> {
	write!(
		out,
		"{} {}",
		integer_type_name(node.kind),
		integer_compare_operator_name(node.operator)
	)
}

const fn number_type_name(kind: number::Type) -> &'static str {
	match kind {
		number::Type::F32 => "F32",
		number::Type::F64 => "F64",
	}
}

const fn number_unary_operator_name(operator: number::UnaryOperator) -> &'static str {
	match operator {
		number::UnaryOperator::Absolute => "Absolute",
		number::UnaryOperator::Negate => "-",
		number::UnaryOperator::SquareRoot => "Square Root",
		number::UnaryOperator::RoundUp => "Round Up",
		number::UnaryOperator::RoundDown => "Round Down",
		number::UnaryOperator::Truncate => "Truncate",
		number::UnaryOperator::Nearest => "Nearest",
	}
}

fn write_number_unary_operation(node: number::UnaryOperation, out: &mut dyn Write) -> Result<()> {
	write!(
		out,
		"{} {}",
		number_type_name(node.kind),
		number_unary_operator_name(node.operator)
	)
}

const fn number_binary_operator_name(operator: number::BinaryOperator) -> &'static str {
	match operator {
		number::BinaryOperator::Add => "+",
		number::BinaryOperator::Subtract => "-",
		number::BinaryOperator::Multiply => "*",
		number::BinaryOperator::Divide => "/",
		number::BinaryOperator::Minimum => "Minimum",
		number::BinaryOperator::Maximum => "Maximum",
		number::BinaryOperator::CopySign => "Copy Sign",
	}
}

fn write_number_binary_operation(node: number::BinaryOperation, out: &mut dyn Write) -> Result<()> {
	write!(
		out,
		"{} {}",
		number_type_name(node.kind),
		number_binary_operator_name(node.operator)
	)
}

const fn number_compare_operator_name(operator: number::CompareOperator) -> &'static str {
	match operator {
		number::CompareOperator::Equal => "==",
		number::CompareOperator::NotEqual => "!=",
		number::CompareOperator::LessThan => "<",
		number::CompareOperator::GreaterThan => ">",
		number::CompareOperator::LessThanEqual => "<=",
		number::CompareOperator::GreaterThanEqual => ">=",
	}
}

fn write_number_compare_operation(
	node: number::CompareOperation,
	out: &mut dyn Write,
) -> Result<()> {
	write!(
		out,
		"{} {}",
		number_type_name(node.kind),
		number_compare_operator_name(node.operator)
	)
}

#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
pub fn write(node: &Node, out: &mut dyn Write) -> Result<()> {
	match *node {
		Node::Function(_)
		| Node::Match(_)
		| Node::Repeat(_)
		| Node::ModuleArguments(_)
		| Node::ModuleResults(_)
		| Node::FunctionCaptures(_)
		| Node::FunctionArguments(_)
		| Node::FunctionResults(_)
		| Node::BranchArguments(_)
		| Node::BranchResults(_)
		| Node::RepeatArguments(_)
		| Node::RepeatResults(_)
		| Node::Foreign(_)
		| Node::Trap
		| Node::Null
		| Node::Identity(_)
		| Node::Fence(_)
		| Node::Apply(_)
		| Node::RefIsNull(_)
		| Node::IntegerNarrow(_)
		| Node::IntegerWiden(_)
		| Node::IntegerConvertToNumber(_)
		| Node::IntegerTransmuteToNumber(_)
		| Node::NumberNarrow(_)
		| Node::NumberWiden(_)
		| Node::NumberTruncateToInteger(_)
		| Node::NumberTransmuteToInteger(_)
		| Node::MutableNew(_)
		| Node::MutableGet(_)
		| Node::MutableSet(_)
		| Node::TableNew(_)
		| Node::TableGet(_)
		| Node::TableSet(_)
		| Node::TableSize(_)
		| Node::TableGrow(_)
		| Node::TableFill(_)
		| Node::TableCopy(_)
		| Node::TableDrop(_)
		| Node::MemoryNew(_)
		| Node::MemoryLoad(_)
		| Node::MemoryStore(_)
		| Node::MemorySize(_)
		| Node::MemoryGrow(_)
		| Node::MemoryFill(_)
		| Node::MemoryCopy(_)
		| Node::MemoryDrop(_) => Err(Error::other("node does not have a dynamic name")),

		Node::I32(i32) => write!(out, "{i32}_i32"),
		Node::I64(i64) => write!(out, "{i64}_i64"),
		Node::F32(f32) => write!(out, "{f32:e}_f32"),
		Node::F64(f64) => write!(out, "{f64:e}_f64"),
		Node::IntegerUnaryOperation(node) => write_integer_unary_operation(node, out),
		Node::IntegerBinaryOperation(node) => write_integer_binary_operation(node, out),
		Node::IntegerCompareOperation(node) => write_integer_compare_operation(node, out),
		Node::IntegerSignExtend(node) => write_integer_sign_extend(node, out),
		Node::NumberUnaryOperation(node) => write_number_unary_operation(node, out),
		Node::NumberBinaryOperation(node) => write_number_binary_operation(node, out),
		Node::NumberCompareOperation(node) => write_number_compare_operation(node, out),
	}
}
