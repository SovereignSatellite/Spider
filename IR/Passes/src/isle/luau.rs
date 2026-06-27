//! Downcasting of Luau foreign nodes for the ISLE peephole engine.

use core::any::Any;

use ir_graph::Link;
use luau_foreign::{
	Bit32And, Bit32ArShift, Bit32CountLz, Bit32CountRz, Bit32LRotate, Bit32LShift, Bit32Or,
	Bit32RRotate, Bit32RShift, Bit32Xor, BooleanToInteger, FlipMostSignificant, LuauAdd, LuauAnd,
	LuauDivide, LuauEqual, LuauFloorDivide, LuauLessThan, LuauLessThanEqual, LuauModulo,
	LuauMultiply, LuauNegate, LuauNotEqual, LuauOr, LuauSubtract, MathAbs, MathCeil, MathFloor,
	MathFmod, MathMax, MathMin, MathModf, MathSqrt,
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

/// The operator carried by a Luau unary value node.
#[derive(Clone, Copy)]
pub enum LuauUnaryOperator {
	/// Arithmetic negation.
	Negate,
	/// `math.abs`.
	Absolute,
	/// `math.sqrt`.
	SquareRoot,
	/// `math.floor`.
	RoundDown,
	/// `math.ceil`.
	RoundUp,
	/// `math.modf`, taking the truncated integral part.
	RoundToZero,
	/// The i64 sign-bit flip.
	FlipMostSignificant,
}

/// The operator carried by a Luau arithmetic value node.
#[derive(Clone, Copy)]
pub enum LuauArithmeticOperator {
	/// The Lua `+` operator.
	Add,
	/// The Lua `-` operator.
	Subtract,
	/// The Lua `*` operator.
	Multiply,
	/// The Lua `/` operator.
	Divide,
	/// The Lua `//` floor-division operator.
	FloorDivide,
	/// The Lua `%` operator.
	Modulo,
}

/// The operator carried by a Luau binary value node.
#[derive(Clone, Copy)]
pub enum LuauBinaryOperator {
	/// `math.min`.
	Minimum,
	/// `math.max`.
	Maximum,
	/// `math.fmod`.
	FloatModulo,
	/// The Lua `and` short-circuit operator.
	And,
	/// The Lua `or` short-circuit operator.
	Or,
}

/// The operator carried by a Luau comparison value node.
#[derive(Clone, Copy)]
pub enum LuauCompareOperator {
	/// The Lua `==` operator.
	Equal,
	/// The Lua `~=` operator.
	NotEqual,
	/// The Lua `<` operator.
	LessThan,
	/// The Lua `<=` operator.
	LessThanEqual,
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

pub fn luau_unary_operation(any: &dyn Any) -> Option<(Link, LuauUnaryOperator)> {
	downcast_operation!(any,
		LuauNegate => (source) => LuauUnaryOperator::Negate,
		MathAbs => (source) => LuauUnaryOperator::Absolute,
		MathSqrt => (source) => LuauUnaryOperator::SquareRoot,
		MathFloor => (source) => LuauUnaryOperator::RoundDown,
		MathCeil => (source) => LuauUnaryOperator::RoundUp,
		MathModf => (source) => LuauUnaryOperator::RoundToZero,
		FlipMostSignificant => (source) => LuauUnaryOperator::FlipMostSignificant,
	)
}

pub fn luau_arithmetic_operation(any: &dyn Any) -> Option<(Link, Link, LuauArithmeticOperator)> {
	downcast_operation!(any,
		LuauAdd => (lhs, rhs) => LuauArithmeticOperator::Add,
		LuauSubtract => (lhs, rhs) => LuauArithmeticOperator::Subtract,
		LuauMultiply => (lhs, rhs) => LuauArithmeticOperator::Multiply,
		LuauDivide => (lhs, rhs) => LuauArithmeticOperator::Divide,
		LuauFloorDivide => (lhs, rhs) => LuauArithmeticOperator::FloorDivide,
		LuauModulo => (lhs, rhs) => LuauArithmeticOperator::Modulo,
	)
}

pub fn luau_binary_operation(any: &dyn Any) -> Option<(Link, Link, LuauBinaryOperator)> {
	downcast_operation!(any,
		MathMin => (lhs, rhs) => LuauBinaryOperator::Minimum,
		MathMax => (lhs, rhs) => LuauBinaryOperator::Maximum,
		MathFmod => (lhs, rhs) => LuauBinaryOperator::FloatModulo,
		LuauAnd => (lhs, rhs) => LuauBinaryOperator::And,
		LuauOr => (lhs, rhs) => LuauBinaryOperator::Or,
	)
}

pub fn luau_compare_operation(any: &dyn Any) -> Option<(Link, Link, LuauCompareOperator)> {
	downcast_operation!(any,
		LuauEqual => (lhs, rhs) => LuauCompareOperator::Equal,
		LuauNotEqual => (lhs, rhs) => LuauCompareOperator::NotEqual,
		LuauLessThan => (lhs, rhs) => LuauCompareOperator::LessThan,
		LuauLessThanEqual => (lhs, rhs) => LuauCompareOperator::LessThanEqual,
	)
}

pub fn boolean_to_integer(any: &dyn Any) -> Option<Link> {
	let &BooleanToInteger { source } = any.downcast_ref::<BooleanToInteger>()?;

	Some(source)
}
