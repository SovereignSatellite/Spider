use ir_graph::Node;

#[derive(Clone, Copy)]
pub enum Color {
	Blue,
	Green,
	Red,
	Brown,
	Yellow,
}

impl Color {
	#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
	pub const fn from_reference(node: &Node) -> Self {
		match node {
			Node::Function(_) | Node::FunctionArguments(_) | Node::FunctionResults(_) => Self::Blue,
			Node::Match(_) | Node::BranchArguments(_) | Node::BranchResults(_) => Self::Green,
			Node::Repeat(_) | Node::RepeatArguments(_) | Node::RepeatResults(_) => Self::Red,
			Node::ModuleArguments(_) | Node::ModuleResults(_) => Self::Brown,

			Node::Foreign(_)
			| Node::Trap
			| Node::Null
			| Node::I32(_)
			| Node::I64(_)
			| Node::F32(_)
			| Node::F64(_)
			| Node::Identity(_)
			| Node::Fence(_)
			| Node::Apply(_)
			| Node::RefIsNull(_)
			| Node::IntegerUnaryOperation(_)
			| Node::IntegerBinaryOperation(_)
			| Node::IntegerCompareOperation(_)
			| Node::IntegerNarrow(_)
			| Node::IntegerWiden(_)
			| Node::IntegerSignExtend(_)
			| Node::IntegerConvertToNumber(_)
			| Node::IntegerTransmuteToNumber(_)
			| Node::NumberUnaryOperation(_)
			| Node::NumberBinaryOperation(_)
			| Node::NumberCompareOperation(_)
			| Node::NumberNarrow(_)
			| Node::NumberWiden(_)
			| Node::NumberTruncateToInteger(_)
			| Node::NumberTransmuteToInteger(_)
			| Node::MutableNew(_)
			| Node::MutableGet(_)
			| Node::MutableSet(_)
			| Node::Aggregate(_)
			| Node::Extract(_)
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
			| Node::MemoryDrop(_) => Self::Yellow,
		}
	}

	pub const fn as_string(self) -> &'static str {
		match self {
			Self::Blue => "#8BB1F9",
			Self::Green => "#A1FC8F",
			Self::Red => "#E07E7C",
			Self::Brown => "#A99D94",
			Self::Yellow => "#FFFF93",
		}
	}
}
