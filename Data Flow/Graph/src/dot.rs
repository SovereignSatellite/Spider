use core::fmt::{Display, Formatter, Result};

use alloc::vec::Vec;

use crate::{
	DataFlowGraph,
	nested::Import,
	node::{
		Node,
		mvp::{
			DataNew, ExtendType, IntegerBinaryOperation, IntegerBinaryOperator,
			IntegerCompareOperation, IntegerCompareOperator, IntegerExtend, IntegerType,
			IntegerUnaryOperation, IntegerUnaryOperator, NumberBinaryOperation,
			NumberBinaryOperator, NumberCompareOperation, NumberCompareOperator, NumberType,
			NumberUnaryOperation, NumberUnaryOperator,
		},
	},
};

#[derive(PartialEq, Eq, Clone, Copy)]
enum Vertex {
	Lambda,
	Region,
	Gamma,
	Theta,
	Omega,
	Operation,
}

impl Vertex {
	const fn from_node(node: &Node) -> Self {
		match node {
			Node::LambdaIn(_) | Node::LambdaOut(_) => Self::Lambda,
			Node::RegionIn(_) | Node::RegionOut(_) => Self::Region,
			Node::GammaIn(_) | Node::GammaOut(_) => Self::Gamma,
			Node::ThetaIn(_) | Node::ThetaOut(_) => Self::Theta,
			Node::OmegaIn(_) | Node::OmegaOut(_) => Self::Omega,
			Node::Import(_)
			| Node::Host(_)
			| Node::Trap
			| Node::Null
			| Node::Identity(_)
			| Node::I32(_)
			| Node::I64(_)
			| Node::F32(_)
			| Node::F64(_)
			| Node::Call(_)
			| Node::Merge(_)
			| Node::RefIsNull(_)
			| Node::IntegerUnaryOperation(_)
			| Node::IntegerBinaryOperation(_)
			| Node::IntegerCompareOperation(_)
			| Node::IntegerNarrow(_)
			| Node::IntegerWiden(_)
			| Node::IntegerExtend(_)
			| Node::IntegerConvertToNumber(_)
			| Node::IntegerTransmuteToNumber(_)
			| Node::NumberUnaryOperation(_)
			| Node::NumberBinaryOperation(_)
			| Node::NumberCompareOperation(_)
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
			| Node::TableInit(_)
			| Node::ElementsNew(_)
			| Node::ElementsDrop(_)
			| Node::MemoryNew(_)
			| Node::MemoryLoad(_)
			| Node::MemoryStore(_)
			| Node::MemorySize(_)
			| Node::MemoryGrow(_)
			| Node::MemoryFill(_)
			| Node::MemoryCopy(_)
			| Node::MemoryInit(_)
			| Node::DataNew(_)
			| Node::DataDrop(_) => Self::Operation,
		}
	}

	const fn group(self) -> char {
		match self {
			Self::Lambda => 'A',
			Self::Region => 'B',
			Self::Gamma => 'C',
			Self::Theta => 'D',
			Self::Omega => 'E',
			Self::Operation => 'F',
		}
	}

	const fn color(self) -> &'static str {
		match self {
			Self::Lambda => "#8BB1F9",
			Self::Region => "#808026",
			Self::Gamma => "#A1FC8F",
			Self::Theta => "#E07E7C",
			Self::Omega => "#A99D94",
			Self::Operation => "#FFFF93",
		}
	}
}

impl Display for Vertex {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		writeln!(
			f,
			"\tnode [fillcolor = \"{}\", group = {}];",
			self.color(),
			self.group()
		)
	}
}

pub struct Dot<'inner> {
	inner: &'inner DataFlowGraph,
}

impl<'inner> Dot<'inner> {
	#[must_use]
	pub const fn new(inner: &'inner DataFlowGraph) -> Self {
		Self { inner }
	}

	fn try_node_name(node: &Node) -> Option<&'static str> {
		let name = match node {
			Node::LambdaIn(_) => "Lambda In",
			Node::LambdaOut(_) => "Lambda Out",
			Node::RegionIn(_) => "Region In",
			Node::RegionOut(_) => "Region Out",
			Node::GammaIn(_) => "Gamma In",
			Node::GammaOut(_) => "Gamma Out",
			Node::ThetaIn(_) => "Theta In",
			Node::ThetaOut(_) => "Theta Out",
			Node::OmegaIn(_) => "Omega In",
			Node::OmegaOut(_) => "Omega Out",

			Node::Host(host) => host.identifier(),
			Node::Trap => "Trap",
			Node::Null => "Null",
			Node::Identity(_) => "Identity",
			Node::Call(_) => "Call",
			Node::Merge(_) => "Merge",
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
			Node::TableInit(_) => "Table Init",
			Node::ElementsNew(_) => "Elements New",
			Node::ElementsDrop(_) => "Elements Drop",
			Node::MemoryNew(_) => "Memory New",
			Node::MemoryLoad(_) => "Memory Load",
			Node::MemoryStore(_) => "Memory Store",
			Node::MemorySize(_) => "Memory Size",
			Node::MemoryGrow(_) => "Memory Grow",
			Node::MemoryFill(_) => "Memory Fill",
			Node::MemoryCopy(_) => "Memory Copy",
			Node::MemoryInit(_) => "Memory Init",
			Node::DataDrop(_) => "Data Drop",

			_ => return None,
		};

		Some(name)
	}

	fn fmt_import(node: &Import, f: &mut Formatter<'_>) -> Result {
		let namespace = node.namespace.as_bytes().escape_ascii();
		let identifier = node.identifier.as_bytes().escape_ascii();

		write!(f, "Import \\\"{namespace}\\\" \\\"{identifier}\\\"")
	}

	const fn extend_type_name(r#type: ExtendType) -> &'static str {
		match r#type {
			ExtendType::I32_S8 => "S8 to I32",
			ExtendType::I32_S16 => "S16 to I32",
			ExtendType::I64_S8 => "S8 to I64",
			ExtendType::I64_S16 => "S16 to I64",
			ExtendType::I64_S32 => "S32 to I64",
		}
	}

	fn fmt_integer_extend(node: IntegerExtend, f: &mut Formatter<'_>) -> Result {
		write!(f, "Extend {}", Self::extend_type_name(node.r#type))
	}

	const fn integer_type_name(r#type: IntegerType) -> &'static str {
		match r#type {
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

	fn fmt_integer_unary_operation(node: IntegerUnaryOperation, f: &mut Formatter<'_>) -> Result {
		write!(
			f,
			"{} {}",
			Self::integer_type_name(node.r#type),
			Self::integer_unary_operator_name(node.operator)
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

	fn fmt_integer_binary_operation(node: IntegerBinaryOperation, f: &mut Formatter<'_>) -> Result {
		write!(
			f,
			"{} {}",
			Self::integer_type_name(node.r#type),
			Self::integer_binary_operator_name(node.operator)
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

	fn fmt_integer_compare_operation(
		node: IntegerCompareOperation,
		f: &mut Formatter<'_>,
	) -> Result {
		write!(
			f,
			"{} {}",
			Self::integer_type_name(node.r#type),
			Self::integer_compare_operator_name(node.operator)
		)
	}

	const fn number_type_name(r#type: NumberType) -> &'static str {
		match r#type {
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

	fn fmt_number_unary_operation(node: NumberUnaryOperation, f: &mut Formatter<'_>) -> Result {
		write!(
			f,
			"{} {}",
			Self::number_type_name(node.r#type),
			Self::number_unary_operator_name(node.operator)
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

	fn fmt_number_binary_operation(node: NumberBinaryOperation, f: &mut Formatter<'_>) -> Result {
		write!(
			f,
			"{} {}",
			Self::number_type_name(node.r#type),
			Self::number_binary_operator_name(node.operator)
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

	fn fmt_number_compare_operation(node: NumberCompareOperation, f: &mut Formatter<'_>) -> Result {
		write!(
			f,
			"{} {}",
			Self::number_type_name(node.r#type),
			Self::number_compare_operator_name(node.operator)
		)
	}

	fn fmt_data_new(node: &DataNew, f: &mut Formatter<'_>) -> Result {
		let content = &node.content[..node.content.len().min(16)];

		write!(f, "Data New {content:?}")
	}

	fn fmt_node(node: &Node, f: &mut Formatter<'_>) -> Result {
		if let Some(name) = Self::try_node_name(node) {
			return f.write_str(name);
		}

		match *node {
			Node::Import(ref import) => Self::fmt_import(import, f),
			Node::I32(i32) => write!(f, "{i32}_i32"),
			Node::I64(i64) => write!(f, "{i64}_i64"),
			Node::F32(f32) => write!(f, "{f32:e}_f32"),
			Node::F64(f64) => write!(f, "{f64:e}_f64"),
			Node::IntegerUnaryOperation(integer_unary_operation) => {
				Self::fmt_integer_unary_operation(integer_unary_operation, f)
			}
			Node::IntegerBinaryOperation(integer_binary_operation) => {
				Self::fmt_integer_binary_operation(integer_binary_operation, f)
			}
			Node::IntegerCompareOperation(integer_compare_operation) => {
				Self::fmt_integer_compare_operation(integer_compare_operation, f)
			}
			Node::IntegerExtend(integer_extend) => Self::fmt_integer_extend(integer_extend, f),
			Node::NumberUnaryOperation(number_unary_operation) => {
				Self::fmt_number_unary_operation(number_unary_operation, f)
			}
			Node::NumberBinaryOperation(number_binary_operation) => {
				Self::fmt_number_binary_operation(number_binary_operation, f)
			}
			Node::NumberCompareOperation(number_compare_operation) => {
				Self::fmt_number_compare_operation(number_compare_operation, f)
			}
			Node::DataNew(ref data_new) => Self::fmt_data_new(data_new, f),

			_ => unreachable!(),
		}
	}

	fn fmt_nodes(&self, f: &mut Formatter<'_>) -> Result {
		writeln!(f, "\tnode [shape = box, style = filled, ordering = out];")?;

		let mut last_vertex = Vertex::Omega;

		last_vertex.fmt(f)?;

		self.inner.nodes().enumerate().try_for_each(|(id, node)| {
			let vertex = Vertex::from_node(node);

			if vertex != last_vertex {
				last_vertex = vertex;

				last_vertex.fmt(f)?;
			}

			write!(f, "\tN{id} [xlabel = {id}, label = \"")?;

			Self::fmt_node(node, f)?;

			writeln!(f, "\"];")
		})
	}

	fn fmt_edges(&self, f: &mut Formatter<'_>) -> Result {
		writeln!(f, "\tedge [color = \"#444477\"];")?;

		let mut temporary = Vec::new();

		self.inner.nodes().enumerate().try_for_each(|(to, node)| {
			temporary.clear();

			node.for_each_argument(|link| temporary.push(link.0));

			temporary
				.iter()
				.try_for_each(|&from| writeln!(f, "\tN{from} -> N{to};"))?;

			temporary.clear();

			node.for_each_requirement(|id| temporary.push(id));

			temporary.iter().try_for_each(|&from| {
				writeln!(f, "\tN{from} -> N{to} [style = dotted, weight = 1024];")
			})
		})
	}
}

impl Display for Dot<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		writeln!(f, "digraph {{")?;

		self.fmt_nodes(f)?;
		self.fmt_edges(f)?;

		writeln!(f, "}}")
	}
}
