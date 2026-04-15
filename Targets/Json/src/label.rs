use std::io::{Error, Result, Write};

use ir_graph::{
	Node,
	control::Import,
	simple::{
		ExtendType, IntegerBinaryOperation, IntegerBinaryOperator, IntegerCompareOperation,
		IntegerCompareOperator, IntegerExtend, IntegerType, IntegerUnaryOperation,
		IntegerUnaryOperator, NumberBinaryOperation, NumberBinaryOperator, NumberCompareOperation,
		NumberCompareOperator, NumberType, NumberUnaryOperation, NumberUnaryOperator,
	},
};

#[must_use]
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

		Node::Import(_)
		| Node::I32(_)
		| Node::I64(_)
		| Node::F32(_)
		| Node::F64(_)
		| Node::IntegerUnaryOperation(_)
		| Node::IntegerBinaryOperation(_)
		| Node::IntegerCompareOperation(_)
		| Node::IntegerExtend(_)
		| Node::NumberUnaryOperation(_)
		| Node::NumberBinaryOperation(_)
		| Node::NumberCompareOperation(_) => return None,

		Node::Host(host) => host.identifier(),
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
		Node::GlobalNew(_) => "Global New",
		Node::GlobalGet(_) => "Global Get",
		Node::GlobalSet(_) => "Global Set",
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

fn write_import(node: &Import, out: &mut dyn Write) -> Result<()> {
	let namespace = node.namespace.as_bytes().escape_ascii();
	let identifier = node.identifier.as_bytes().escape_ascii();

	write!(out, "Import \"{namespace}\" \"{identifier}\"")
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

fn write_integer_extend(node: IntegerExtend, out: &mut dyn Write) -> Result<()> {
	write!(out, "Extend {}", extend_type_name(node.kind))
}

const fn integer_type_name(kind: IntegerType) -> &'static str {
	match kind {
		IntegerType::I32 => "I32",
		IntegerType::I64 => "I64",
	}
}

const fn integer_unary_operator_name(operator: IntegerUnaryOperator) -> &'static str {
	match operator {
		IntegerUnaryOperator::CountOnes => "Count Ones",
		IntegerUnaryOperator::LeadingZeroes => "Leading Zeroes",
		IntegerUnaryOperator::TrailingZeroes => "Trailing Zeroes",
	}
}

fn write_integer_unary_operation(node: IntegerUnaryOperation, out: &mut dyn Write) -> Result<()> {
	write!(
		out,
		"{} {}",
		integer_type_name(node.kind),
		integer_unary_operator_name(node.operator)
	)
}

const fn integer_binary_operator_name(operator: IntegerBinaryOperator) -> &'static str {
	match operator {
		IntegerBinaryOperator::Add => "+",
		IntegerBinaryOperator::Subtract => "-",
		IntegerBinaryOperator::Multiply => "*",
		IntegerBinaryOperator::Divide { signed: false } => "u/",
		IntegerBinaryOperator::Divide { signed: true } => "s/",
		IntegerBinaryOperator::Remainder { signed: false } => "u%",
		IntegerBinaryOperator::Remainder { signed: true } => "s%",
		IntegerBinaryOperator::And => "&",
		IntegerBinaryOperator::Or => "|",
		IntegerBinaryOperator::ExclusiveOr => "^",
		IntegerBinaryOperator::ShiftLeft => "<<",
		IntegerBinaryOperator::ShiftRight { signed: false } => "u>>",
		IntegerBinaryOperator::ShiftRight { signed: true } => "s>>",
		IntegerBinaryOperator::RotateLeft => "^<<",
		IntegerBinaryOperator::RotateRight => ">>^",
	}
}

fn write_integer_binary_operation(node: IntegerBinaryOperation, out: &mut dyn Write) -> Result<()> {
	write!(
		out,
		"{} {}",
		integer_type_name(node.kind),
		integer_binary_operator_name(node.operator)
	)
}

const fn integer_compare_operator_name(operator: IntegerCompareOperator) -> &'static str {
	match operator {
		IntegerCompareOperator::Equal => "==",
		IntegerCompareOperator::NotEqual => "!=",
		IntegerCompareOperator::LessThan { signed: false } => "u<",
		IntegerCompareOperator::LessThan { signed: true } => "s<",
		IntegerCompareOperator::GreaterThan { signed: false } => "u>",
		IntegerCompareOperator::GreaterThan { signed: true } => "s>",
		IntegerCompareOperator::LessThanEqual { signed: false } => "u<=",
		IntegerCompareOperator::LessThanEqual { signed: true } => "s<=",
		IntegerCompareOperator::GreaterThanEqual { signed: false } => "u>=",
		IntegerCompareOperator::GreaterThanEqual { signed: true } => "s>=",
	}
}

fn write_integer_compare_operation(
	node: IntegerCompareOperation,
	out: &mut dyn Write,
) -> Result<()> {
	write!(
		out,
		"{} {}",
		integer_type_name(node.kind),
		integer_compare_operator_name(node.operator)
	)
}

const fn number_type_name(kind: NumberType) -> &'static str {
	match kind {
		NumberType::F32 => "F32",
		NumberType::F64 => "F64",
	}
}

const fn number_unary_operator_name(operator: NumberUnaryOperator) -> &'static str {
	match operator {
		NumberUnaryOperator::Absolute => "Absolute",
		NumberUnaryOperator::Negate => "-",
		NumberUnaryOperator::SquareRoot => "Square Root",
		NumberUnaryOperator::RoundUp => "Round Up",
		NumberUnaryOperator::RoundDown => "Round Down",
		NumberUnaryOperator::Truncate => "Truncate",
		NumberUnaryOperator::Nearest => "Nearest",
	}
}

fn write_number_unary_operation(node: NumberUnaryOperation, out: &mut dyn Write) -> Result<()> {
	write!(
		out,
		"{} {}",
		number_type_name(node.kind),
		number_unary_operator_name(node.operator)
	)
}

const fn number_binary_operator_name(operator: NumberBinaryOperator) -> &'static str {
	match operator {
		NumberBinaryOperator::Add => "+",
		NumberBinaryOperator::Subtract => "-",
		NumberBinaryOperator::Multiply => "*",
		NumberBinaryOperator::Divide => "/",
		NumberBinaryOperator::Minimum => "Minimum",
		NumberBinaryOperator::Maximum => "Maximum",
		NumberBinaryOperator::CopySign => "Copy Sign",
	}
}

fn write_number_binary_operation(node: NumberBinaryOperation, out: &mut dyn Write) -> Result<()> {
	write!(
		out,
		"{} {}",
		number_type_name(node.kind),
		number_binary_operator_name(node.operator)
	)
}

const fn number_compare_operator_name(operator: NumberCompareOperator) -> &'static str {
	match operator {
		NumberCompareOperator::Equal => "==",
		NumberCompareOperator::NotEqual => "!=",
		NumberCompareOperator::LessThan => "<",
		NumberCompareOperator::GreaterThan => ">",
		NumberCompareOperator::LessThanEqual => "<=",
		NumberCompareOperator::GreaterThanEqual => ">=",
	}
}

fn write_number_compare_operation(node: NumberCompareOperation, out: &mut dyn Write) -> Result<()> {
	write!(
		out,
		"{} {}",
		number_type_name(node.kind),
		number_compare_operator_name(node.operator)
	)
}

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
		| Node::Host(_)
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
		| Node::GlobalNew(_)
		| Node::GlobalGet(_)
		| Node::GlobalSet(_)
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

		Node::Import(ref node) => write_import(node, out),
		Node::I32(i32) => write!(out, "{i32}_i32"),
		Node::I64(i64) => write!(out, "{i64}_i64"),
		Node::F32(f32) => write!(out, "{f32:e}_f32"),
		Node::F64(f64) => write!(out, "{f64:e}_f64"),
		Node::IntegerUnaryOperation(node) => write_integer_unary_operation(node, out),
		Node::IntegerBinaryOperation(node) => write_integer_binary_operation(node, out),
		Node::IntegerCompareOperation(node) => write_integer_compare_operation(node, out),
		Node::IntegerExtend(node) => write_integer_extend(node, out),
		Node::NumberUnaryOperation(node) => write_number_unary_operation(node, out),
		Node::NumberBinaryOperation(node) => write_number_binary_operation(node, out),
		Node::NumberCompareOperation(node) => write_number_compare_operation(node, out),
	}
}
