//! Common node elimination.

use core::any::{Any, TypeId};

use hashbrown::HashMap;

use ir_graph::{
	Link, Node,
	foreign::Foreign,
	operation::{
		ExtendType, Identity,
		integer::{self, BinaryOperator, CompareOperator},
		number,
	},
};
use luau_foreign::{
	Bit32And, Bit32ArShift, Bit32CountLz, Bit32CountRz, Bit32LRotate, Bit32LShift, Bit32Or,
	Bit32RRotate, Bit32RShift, Bit32Xor, BooleanToInteger, FlipMostSignificant, FromBitsF32,
	FromBitsI64, IntoBitsF32, IntoBitsI64, IsPositive, LuauAdd, LuauDivide, LuauEqual,
	LuauFloorDivide, LuauLessThan, LuauLessThanEqual, LuauModulo, LuauMultiply, LuauNegate,
	LuauNotEqual, LuauSubtract, MathAbs, MathCeil, MathFloor, MathFmod, MathMax, MathMin, MathModf,
	MathSqrt, VectorCreate, VectorX,
};

macro_rules! foreign_unary_signatures {
	($any:expr, $($operation:path),+ $(,)?) => {
		$(
			if let Some(&$operation { source }) = $any.downcast_ref::<$operation>() {
				return Some(Signature::ForeignUnary(TypeId::of::<$operation>(), source));
			}
		)+
	};
}

macro_rules! foreign_binary_signatures {
	($any:expr, $is_commutative:literal, $($operation:path),+ $(,)?) => {
		$(
			if let Some(&$operation { lhs, rhs }) = $any.downcast_ref::<$operation>() {
				let (lhs, rhs) = if $is_commutative {
					Signature::ordered(lhs, rhs)
				} else {
					(lhs, rhs)
				};

				return Some(Signature::ForeignBinary(TypeId::of::<$operation>(), lhs, rhs));
			}
		)+
	};
}

/// A congruence class key: two nodes with equal signatures always compute the
/// same value, because they perform the same operation on the same operands.
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum Signature {
	RefIsNull(Link),
	IntegerUnary(integer::UnaryOperator, integer::Type, Link),
	IntegerBinary(BinaryOperator, integer::Type, Link, Link),
	IntegerCompare(CompareOperator, integer::Type, Link, Link),
	IntegerNarrow(Link),
	IntegerWiden(Link),
	IntegerSignExtend(ExtendType, Link),
	IntegerConvertToNumber(bool, number::Type, integer::Type, Link),
	IntegerTransmuteToNumber(integer::Type, Link),
	NumberUnary(number::UnaryOperator, number::Type, Link),
	NumberBinary(number::BinaryOperator, number::Type, Link, Link),
	NumberCompare(number::CompareOperator, number::Type, Link, Link),
	NumberNarrow(Link),
	NumberWiden(Link),
	NumberTruncateToInteger(bool, bool, integer::Type, number::Type, Link),
	NumberTransmuteToInteger(number::Type, Link),
	Extract(u32, Link),
	ForeignUnary(TypeId, Link),
	ForeignBinary(TypeId, Link, Link),
}

impl Signature {
	fn ordered(lhs: Link, rhs: Link) -> (Link, Link) {
		if rhs < lhs { (rhs, lhs) } else { (lhs, rhs) }
	}

	const fn is_commutative_binary(operator: BinaryOperator) -> bool {
		matches!(
			operator,
			BinaryOperator::Add
				| BinaryOperator::Multiply
				| BinaryOperator::And
				| BinaryOperator::Or
				| BinaryOperator::ExclusiveOr
		)
	}

	const fn is_commutative_compare(operator: CompareOperator) -> bool {
		matches!(operator, CompareOperator::Equal | CompareOperator::NotEqual)
	}

	fn integer_binary(operation: &integer::BinaryOperation) -> Self {
		let (lhs, rhs) = if Self::is_commutative_binary(operation.operator) {
			Self::ordered(operation.lhs, operation.rhs)
		} else {
			(operation.lhs, operation.rhs)
		};

		Self::IntegerBinary(operation.operator, operation.kind, lhs, rhs)
	}

	fn integer_compare(operation: &integer::CompareOperation) -> Self {
		let (lhs, rhs) = if Self::is_commutative_compare(operation.operator) {
			Self::ordered(operation.lhs, operation.rhs)
		} else {
			(operation.lhs, operation.rhs)
		};

		Self::IntegerCompare(operation.operator, operation.kind, lhs, rhs)
	}

	fn foreign(foreign: &dyn Foreign) -> Option<Self> {
		let any: &dyn Any = foreign;

		Self::foreign_unary(any).or_else(|| Self::foreign_binary(any))
	}

	fn foreign_unary(any: &dyn Any) -> Option<Self> {
		foreign_unary_signatures!(
			any,
			Bit32CountLz,
			Bit32CountRz,
			MathAbs,
			MathSqrt,
			MathCeil,
			MathFloor,
			MathModf,
			LuauNegate,
			BooleanToInteger,
			FlipMostSignificant,
			FromBitsF32,
			IntoBitsF32,
			IsPositive,
			FromBitsI64,
			VectorCreate,
			VectorX,
		);

		None
	}

	// Operand sorting is licensed only where the operation commutes for every
	// value the operands can carry: `Luau.Add` and `Luau.Multiply` see floats
	// whose NaN-payload asymmetry the bit transmutes observe, and `Math.Min`
	// and `Math.Max` propagate whichever NaN arrives first.
	#[expect(
		clippy::cognitive_complexity,
		reason = "a flat downcast chain over every pure binary foreign"
	)]
	fn foreign_binary(any: &dyn Any) -> Option<Self> {
		foreign_binary_signatures!(
			any,
			true,
			Bit32And,
			Bit32Or,
			Bit32Xor,
			LuauEqual,
			LuauNotEqual,
		);
		foreign_binary_signatures!(
			any,
			false,
			Bit32LShift,
			Bit32RShift,
			Bit32ArShift,
			Bit32LRotate,
			Bit32RRotate,
			MathMin,
			MathMax,
			MathFmod,
			LuauAdd,
			LuauSubtract,
			LuauMultiply,
			LuauDivide,
			LuauFloorDivide,
			LuauModulo,
			LuauLessThan,
			LuauLessThanEqual,
			IntoBitsI64,
		);

		None
	}

	#[expect(clippy::too_many_lines, reason = "exhaustive match over node variants")]
	fn of(node: &Node) -> Option<Self> {
		match node {
			Node::Function(_)
			| Node::Match(_)
			| Node::Repeat(_)
			| Node::FunctionArguments(_)
			| Node::FunctionResults(_)
			| Node::BranchArguments(_)
			| Node::BranchResults(_)
			| Node::RepeatArguments(_)
			| Node::RepeatResults(_)
			| Node::Import(_)
			| Node::Export(_)
			| Node::Trap
			| Node::Null
			| Node::I32(_)
			| Node::I64(_)
			| Node::F32(_)
			| Node::F64(_)
			| Node::Identity(_)
			| Node::Fence(_)
			| Node::Apply(_)
			| Node::MutableNew(_)
			| Node::MutableGet(_)
			| Node::MutableSet(_)
			| Node::Aggregate(_)
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
			| Node::MemoryDrop(_) => None,

			Node::Foreign(foreign) => Self::foreign(foreign.as_ref()),
			Node::RefIsNull(operation) => Some(Self::RefIsNull(operation.source)),
			Node::IntegerUnaryOperation(operation) => Some(Self::IntegerUnary(
				operation.operator,
				operation.kind,
				operation.source,
			)),
			Node::IntegerBinaryOperation(operation) => Some(Self::integer_binary(operation)),
			Node::IntegerCompareOperation(operation) => Some(Self::integer_compare(operation)),
			Node::IntegerNarrow(operation) => Some(Self::IntegerNarrow(operation.source)),
			Node::IntegerWiden(operation) => Some(Self::IntegerWiden(operation.source)),
			Node::IntegerSignExtend(operation) => {
				Some(Self::IntegerSignExtend(operation.kind, operation.source))
			}
			Node::IntegerConvertToNumber(operation) => Some(Self::IntegerConvertToNumber(
				operation.is_signed,
				operation.to,
				operation.from,
				operation.source,
			)),
			Node::IntegerTransmuteToNumber(operation) => Some(Self::IntegerTransmuteToNumber(
				operation.from,
				operation.source,
			)),
			Node::NumberUnaryOperation(operation) => Some(Self::NumberUnary(
				operation.operator,
				operation.kind,
				operation.source,
			)),
			Node::NumberBinaryOperation(operation) => Some(Self::NumberBinary(
				operation.operator,
				operation.kind,
				operation.lhs,
				operation.rhs,
			)),
			Node::NumberCompareOperation(operation) => Some(Self::NumberCompare(
				operation.operator,
				operation.kind,
				operation.lhs,
				operation.rhs,
			)),
			Node::NumberNarrow(operation) => Some(Self::NumberNarrow(operation.source)),
			Node::NumberWiden(operation) => Some(Self::NumberWiden(operation.source)),
			Node::NumberTruncateToInteger(operation) => Some(Self::NumberTruncateToInteger(
				operation.is_signed,
				operation.is_saturating,
				operation.to,
				operation.from,
				operation.source,
			)),
			Node::NumberTransmuteToInteger(operation) => Some(Self::NumberTransmuteToInteger(
				operation.from,
				operation.source,
			)),
			Node::Extract(operation) => Some(Self::Extract(operation.index, operation.source)),
		}
	}
}

/// Merges congruent value operations within a region by diverting every
/// duplicate to one representative.
pub struct CommonNodeEliminator {
	representatives: HashMap<Signature, u32>,
}

impl CommonNodeEliminator {
	/// Creates a new common node eliminator.
	#[must_use]
	pub fn new() -> Self {
		Self {
			representatives: HashMap::new(),
		}
	}

	fn process(&mut self, nodes: &mut [Node], index: usize) -> bool {
		let Some(signature) = Signature::of(&nodes[index]) else {
			return false;
		};

		let Ok(id) = u32::try_from(index) else {
			unreachable!()
		};

		if let Some(&representative) = self.representatives.get(&signature) {
			let sources = (0..nodes[index].result_count())
				.map(|port| Link(representative, port))
				.collect();

			nodes[index] = Node::Identity(Identity { sources });

			true
		} else {
			self.representatives.insert(signature, id);

			false
		}
	}

	/// Runs common node elimination on the region, reporting whether any node
	/// was merged.
	pub fn run(&mut self, nodes: &mut [Node]) -> bool {
		self.representatives.clear();

		let mut merged = false;

		for index in 0..nodes.len() {
			merged |= self.process(nodes, index);
		}

		merged
	}
}

impl Default for CommonNodeEliminator {
	fn default() -> Self {
		Self::new()
	}
}
