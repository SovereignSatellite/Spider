//! Downcasting of Luau foreign nodes for the ISLE peephole engine.

use core::any::Any;

use ir_graph::Link;
use luau_foreign::{
	Bit32And, Bit32ArShift, Bit32CountLz, Bit32CountRz, Bit32LRotate, Bit32LShift, Bit32Or,
	Bit32RRotate, Bit32RShift, Bit32Xor,
};

/// The operator carried by a Luau `bit32` binary node.
#[derive(Clone, Copy)]
pub enum Bit32BinaryOperator {
	/// `bit32.band`.
	And,
	/// `bit32.bor`.
	Or,
	/// `bit32.bxor`.
	ExclusiveOr,
	/// `bit32.lshift`.
	ShiftLeft,
	/// `bit32.rshift`.
	ShiftRightUnsigned,
	/// `bit32.arshift`.
	ShiftRightSigned,
	/// `bit32.lrotate`.
	RotateLeft,
	/// `bit32.rrotate`.
	RotateRight,
}

/// The operator carried by a Luau `bit32` unary node.
#[derive(Clone, Copy)]
pub enum Bit32UnaryOperator {
	/// `bit32.countlz`.
	CountLeadingZeros,
	/// `bit32.countrz`.
	CountTrailingZeros,
}

macro_rules! downcast_operation {
	($any:expr, $($node:path => ($($field:ident),+) => $operator:expr),+ $(,)?) => {{
		$(
			if let Some(&$node { $($field),+ }) = $any.downcast_ref::<$node>() {
				return Some(($($field,)+ $operator));
			}
		)+

		None
	}};
}

pub fn bit32_binary_operation(any: &dyn Any) -> Option<(Link, Link, Bit32BinaryOperator)> {
	downcast_operation!(any,
		Bit32And => (lhs, rhs) => Bit32BinaryOperator::And,
		Bit32Or => (lhs, rhs) => Bit32BinaryOperator::Or,
		Bit32Xor => (lhs, rhs) => Bit32BinaryOperator::ExclusiveOr,
		Bit32LShift => (lhs, rhs) => Bit32BinaryOperator::ShiftLeft,
		Bit32RShift => (lhs, rhs) => Bit32BinaryOperator::ShiftRightUnsigned,
		Bit32ArShift => (lhs, rhs) => Bit32BinaryOperator::ShiftRightSigned,
		Bit32LRotate => (lhs, rhs) => Bit32BinaryOperator::RotateLeft,
		Bit32RRotate => (lhs, rhs) => Bit32BinaryOperator::RotateRight,
	)
}

pub fn bit32_unary_operation(any: &dyn Any) -> Option<(Link, Bit32UnaryOperator)> {
	downcast_operation!(any,
		Bit32CountLz => (source) => Bit32UnaryOperator::CountLeadingZeros,
		Bit32CountRz => (source) => Bit32UnaryOperator::CountTrailingZeros,
	)
}

pub fn is_bit32_canonical(any: &dyn Any) -> bool {
	bit32_binary_operation(any).is_some() || bit32_unary_operation(any).is_some()
}
