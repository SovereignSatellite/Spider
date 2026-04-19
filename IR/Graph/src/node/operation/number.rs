//! Floating-point arithmetic and comparison operations.

#![expect(
	unused_variables,
	reason = "macro-generated visitors may not consume every field"
)]

use crate::{Link, Node};

/// Floating-point types.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Type {
	/// A 32-bit float.
	F32,
	/// A 64-bit float.
	F64,
}

/// Unary operators for floating-point values.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum UnaryOperator {
	/// Absolute value.
	Absolute,
	/// Negation.
	Negate,
	/// Square root.
	SquareRoot,
	/// Rounds toward positive infinity.
	RoundUp,
	/// Rounds toward negative infinity.
	RoundDown,
	/// Truncates toward zero.
	Truncate,
	/// Rounds to the nearest integer.
	Nearest,
}

/// A floating-point unary operation node.
#[derive(Clone, Copy)]
pub struct UnaryOperation {
	/// The link to the source value.
	pub source: Link,
	/// The floating-point type of the operation.
	pub kind: Type,
	/// The operator to apply.
	pub operator: UnaryOperator,
}

impl UnaryOperation {
	/// Adds a floating-point unary operation node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		source: Link,
		kind: Type,
		operator: UnaryOperator,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::NumberUnaryOperation(Self {
			source,
			kind,
			operator,
		});

		nodes.push(node);

		Link(id, 0)
	}

	handle_sources!((source, link), (kind, ignore), (operator, ignore));
}

/// Binary operators for floating-point arithmetic.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BinaryOperator {
	/// Addition.
	Add,
	/// Subtraction.
	Subtract,
	/// Multiplication.
	Multiply,
	/// Division.
	Divide,
	/// Minimum.
	Minimum,
	/// Maximum.
	Maximum,
	/// Copies the sign of one operand to the other.
	CopySign,
}

/// A floating-point binary operation node.
#[derive(Clone, Copy)]
pub struct BinaryOperation {
	/// The left-hand operand.
	pub lhs: Link,
	/// The right-hand operand.
	pub rhs: Link,
	/// The floating-point type of the operation.
	pub kind: Type,
	/// The operator to apply.
	pub operator: BinaryOperator,
}

impl BinaryOperation {
	/// Adds a floating-point binary operation node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		lhs: Link,
		rhs: Link,
		kind: Type,
		operator: BinaryOperator,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::NumberBinaryOperation(Self {
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

/// Comparison operators for floating-point values.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CompareOperator {
	/// Equality.
	Equal,
	/// Inequality.
	NotEqual,
	/// Less than.
	LessThan,
	/// Greater than.
	GreaterThan,
	/// Less than or equal.
	LessThanEqual,
	/// Greater than or equal.
	GreaterThanEqual,
}

/// A floating-point comparison operation node.
#[derive(Clone, Copy)]
pub struct CompareOperation {
	/// The left-hand operand.
	pub lhs: Link,
	/// The right-hand operand.
	pub rhs: Link,
	/// The floating-point type of the operands.
	pub kind: Type,
	/// The comparison operator.
	pub operator: CompareOperator,
}

impl CompareOperation {
	/// Adds a floating-point comparison node to the graph.
	pub fn add_into(
		nodes: &mut Vec<Node>,
		lhs: Link,
		rhs: Link,
		kind: Type,
		operator: CompareOperator,
	) -> Link {
		let id = nodes.len().try_into().unwrap_or_else(|_| unreachable!());
		let node = Node::NumberCompareOperation(Self {
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
