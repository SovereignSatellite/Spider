//! Integer arithmetic and comparison operations.

#![expect(
	unused_variables,
	reason = "macro-generated visitors may not consume every field"
)]

use crate::{Link, Node};

/// Integer types.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Type {
	/// A 32-bit integer.
	I32,
	/// A 64-bit integer.
	I64,
}

/// Unary operators for integers.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum UnaryOperator {
	/// Population count.
	CountOnes,
	/// Count of leading zero bits.
	LeadingZeroes,
	/// Count of trailing zero bits.
	TrailingZeroes,
}

/// An integer unary operation node.
#[derive(Clone, Copy)]
pub struct UnaryOperation {
	/// The link to the source value.
	pub source: Link,
	/// The integer type of the operation.
	pub kind: Type,
	/// The operator to apply.
	pub operator: UnaryOperator,
}

impl UnaryOperation {
	/// Adds an integer unary operation node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		source: Link,
		kind: Type,
		operator: UnaryOperator,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
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
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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
		signed: bool,
	},
	/// Remainder.
	Remainder {
		/// Whether the operands are treated as signed.
		signed: bool,
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
		signed: bool,
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
	pub fn add_into(
		nodes: &mut Vec<Node>,
		lhs: Link,
		rhs: Link,
		kind: Type,
		operator: BinaryOperator,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
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
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CompareOperator {
	/// Equality.
	Equal,
	/// Inequality.
	NotEqual,
	/// Less than.
	LessThan {
		/// Whether the comparison is signed.
		signed: bool,
	},
	/// Greater than.
	GreaterThan {
		/// Whether the comparison is signed.
		signed: bool,
	},
	/// Less than or equal.
	LessThanEqual {
		/// Whether the comparison is signed.
		signed: bool,
	},
	/// Greater than or equal.
	GreaterThanEqual {
		/// Whether the comparison is signed.
		signed: bool,
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
	pub fn add_into(
		nodes: &mut Vec<Node>,
		lhs: Link,
		rhs: Link,
		kind: Type,
		operator: CompareOperator,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
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
