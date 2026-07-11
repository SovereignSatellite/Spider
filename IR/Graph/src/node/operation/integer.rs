//! Integer arithmetic and comparison operations.

#![expect(
	unused_variables,
	reason = "macro-generated visitors may not consume every field"
)]

use crate::{Link, Node};

/// Integer types.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Type {
	/// A 32-bit integer.
	I32,
	/// A 64-bit integer.
	I64,
}

/// Unary operators for integers.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum UnaryOperator {
	/// Population count.
	CountOnes,
	/// Count of leading zero bits.
	LeadingZeros,
	/// Count of trailing zero bits.
	TrailingZeros,
}

/// An integer unary operation node.
#[derive(Clone, Copy)]
pub struct UnaryOperation {
	/// The source value.
	pub source: Link,
	/// The integer type of the operation.
	pub kind: Type,
	/// The operator to apply.
	pub operator: UnaryOperator,
}

impl UnaryOperation {
	/// Adds an integer unary operation node to the graph.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(
		nodes: &mut Vec<Node>,
		source: Link,
		kind: Type,
		operator: UnaryOperator,
	) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::IntegerUnaryOperation(Self {
			source,
			kind,
			operator,
		});

		nodes.push(node);

		Link(id, 0)
	}

	handle_sources!((source, link), (kind, ignore), (operator, ignore));
}

/// Binary operators for integer arithmetic.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum BinaryOperator {
	/// Addition.
	Add,
	/// Subtraction.
	Subtract,
	/// Multiplication.
	Multiply,
	/// Division.
	Divide {
		/// Whether the operands are treated as signed.
		is_signed: bool,
	},
	/// Remainder.
	Remainder {
		/// Whether the operands are treated as signed.
		is_signed: bool,
	},
	/// Bitwise AND.
	And,
	/// Bitwise OR.
	Or,
	/// Bitwise exclusive OR.
	ExclusiveOr,
	/// Left shift.
	ShiftLeft,
	/// Right shift.
	ShiftRight {
		/// Whether the shift is arithmetic.
		is_signed: bool,
	},
	/// Left rotation.
	RotateLeft,
	/// Right rotation.
	RotateRight,
}

/// An integer binary operation node.
#[derive(Clone, Copy)]
pub struct BinaryOperation {
	/// The left-hand operand.
	pub lhs: Link,
	/// The right-hand operand.
	pub rhs: Link,
	/// The integer type of the operation.
	pub kind: Type,
	/// The operator to apply.
	pub operator: BinaryOperator,
}

impl BinaryOperation {
	/// Adds an integer binary operation node to the graph.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(
		nodes: &mut Vec<Node>,
		lhs: Link,
		rhs: Link,
		kind: Type,
		operator: BinaryOperator,
	) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::IntegerBinaryOperation(Self {
			lhs,
			rhs,
			kind,
			operator,
		});

		nodes.push(node);

		Link(id, 0)
	}

	handle_sources!((lhs, link), (rhs, link), (kind, ignore), (operator, ignore));
}

/// Comparison operators for integers.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum CompareOperator {
	/// Equality.
	Equal,
	/// Inequality.
	NotEqual,
	/// Less than.
	LessThan {
		/// Whether the comparison is signed.
		is_signed: bool,
	},
	/// Less than or equal.
	LessThanEqual {
		/// Whether the comparison is signed.
		is_signed: bool,
	},
}

/// An integer comparison operation node.
#[derive(Clone, Copy)]
pub struct CompareOperation {
	/// The left-hand operand.
	pub lhs: Link,
	/// The right-hand operand.
	pub rhs: Link,
	/// The integer type of the operands.
	pub kind: Type,
	/// The comparison operator.
	pub operator: CompareOperator,
}

impl CompareOperation {
	/// Adds an integer comparison node to the graph.
	#[must_use = "inserted nodes without live consumers are dead"]
	pub fn add_into(
		nodes: &mut Vec<Node>,
		lhs: Link,
		rhs: Link,
		kind: Type,
		operator: CompareOperator,
	) -> Link {
		let Ok(id) = nodes.len().try_into() else {
			unreachable!()
		};
		let node = Node::IntegerCompareOperation(Self {
			lhs,
			rhs,
			kind,
			operator,
		});

		nodes.push(node);

		Link(id, 0)
	}

	handle_sources!((lhs, link), (rhs, link), (kind, ignore), (operator, ignore));
}
