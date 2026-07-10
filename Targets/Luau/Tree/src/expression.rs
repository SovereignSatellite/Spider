//! Expression types for the Luau tree.

use alloc::sync::Arc;

use super::statement::Sequence;

/// A function definition.
pub struct Function {
	/// The argument names.
	pub arguments: Vec<Name>,
	/// The local variable names.
	pub locals: Vec<Name>,
	/// The stack size.
	pub stack: u16,
	/// The function body.
	pub code: Sequence,
	/// The return expressions.
	pub returns: Vec<Expression>,
}

/// A conditional match expression.
pub struct Match {
	/// The branch expressions.
	pub branches: Vec<Expression>,
	/// The condition expression.
	pub condition: Expression,
}

/// A call to a statically-named runtime function or alias with a fixed arity.
pub struct Apply<const N: usize> {
	/// The exact printed callee name (bare alias like `bit32_and`, or `rt_`-prefixed like `rt_add_i32`).
	pub name: &'static str,
	/// The argument expressions.
	pub arguments: [Expression; N],
}

/// A variable name identifier.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name {
	/// The name identifier.
	pub id: u32,
}

/// A local variable reference.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Local {
	/// A fast register variable.
	Fast {
		/// The variable name.
		name: Name,
	},
	/// A slow stack variable.
	Slow {
		/// The stack offset.
		offset: u16,
	},
}

/// A function call.
pub struct Call {
	/// The function expression.
	pub function: Expression,
	/// The argument expressions.
	pub arguments: Vec<Expression>,
}

/// A boolean-to-integer conversion.
pub struct BooleanToInteger {
	/// The source expression.
	pub source: Expression,
}

/// A reference null check.
pub struct RefIsNull {
	/// The source expression.
	pub source: Expression,
}

/// An aggregate construction.
pub struct Aggregate {
	/// The field values, in field order.
	pub fields: Vec<Expression>,
}

/// An aggregate field projection.
pub struct Extract {
	/// The source aggregate.
	pub source: Expression,
	/// The zero-based field index.
	pub index: u32,
}

/// A named field access: `(source).name`.
pub struct Field {
	/// The source expression.
	pub source: Expression,
	/// The accessed field name.
	pub name: &'static str,
}

/// A table creation.
pub struct TableNew {
	/// The initial elements and their offsets.
	pub initializer: Vec<(Expression, u32)>,
	/// The minimum element count.
	pub minimum: u32,
	/// The maximum element count.
	pub maximum: u32,
}

/// A table element read: `(source)[offset]`.
pub struct Index {
	/// The source table.
	pub source: Expression,
	/// The element offset.
	pub offset: Expression,
}

/// A memory creation yielding a fixed-size memory, or `nil` when the
/// allocation fails.
pub struct MemoryNew {
	/// The initial contents and their offsets.
	pub initializer: Vec<(Arc<[u8]>, u32)>,
	/// The size in bytes.
	pub size: Expression,
}

/// A binary operator application: `(lhs) operator (rhs)`.
pub struct Infix {
	/// The operator text.
	pub operator: &'static str,
	/// The left-hand operand.
	pub lhs: Expression,
	/// The right-hand operand.
	pub rhs: Expression,
}

/// A unary operator application: `operator(source)`.
pub struct Prefix {
	/// The operator text.
	pub operator: &'static str,
	/// The operand.
	pub source: Expression,
}

/// An expression node.
pub enum Expression {
	/// A function definition.
	Function(Box<Function>),
	/// A conditional match expression.
	Match(Box<Match>),

	/// An unreachable trap.
	Trap,
	/// A null reference constant.
	Null,

	/// A local variable reference.
	Local(Local),

	/// A 32-bit integer constant.
	I32(i32),
	/// A 64-bit integer constant.
	I64(i64),
	/// A 32-bit float constant.
	F32(f32),
	/// A 64-bit float constant.
	F64(f64),
	/// A string literal.
	String(Arc<str>),

	/// A function call.
	Call(Box<Call>),
	/// A zero-argument statically-named call.
	Apply0Arguments(Box<Apply<0>>),
	/// A one-argument statically-named call.
	Apply1Argument(Box<Apply<1>>),
	/// A two-argument statically-named call.
	Apply2Arguments(Box<Apply<2>>),
	/// A three-argument statically-named call.
	Apply3Arguments(Box<Apply<3>>),
	/// A four-argument statically-named call.
	Apply4Arguments(Box<Apply<4>>),
	/// A five-argument statically-named call.
	Apply5Arguments(Box<Apply<5>>),
	/// A binary operator application.
	Infix(Box<Infix>),
	/// A unary operator application.
	Prefix(Box<Prefix>),

	/// A boolean-to-integer conversion.
	BooleanToInteger(Box<BooleanToInteger>),
	/// A reference null check.
	RefIsNull(Box<RefIsNull>),

	/// An aggregate construction.
	Aggregate(Box<Aggregate>),
	/// An aggregate field projection.
	Extract(Box<Extract>),
	/// A named field access.
	Field(Box<Field>),

	/// A table creation.
	TableNew(Box<TableNew>),
	/// A table element read.
	Index(Box<Index>),

	/// A memory creation.
	MemoryNew(Box<MemoryNew>),
}

impl Expression {
	fn into_boolean_unchecked(self) -> Self {
		let operation = Infix {
			operator: "~=",
			lhs: self,
			rhs: Self::I32(0),
		};

		Self::Infix(operation.into())
	}

	/// Converts this expression into a boolean.
	#[must_use]
	pub fn into_boolean(self) -> Self {
		match self {
			Self::Function(_)
			| Self::Null
			| Self::I64(_)
			| Self::F32(_)
			| Self::F64(_)
			| Self::String(_)
			| Self::Infix(_)
			| Self::Prefix(_)
			| Self::Aggregate(_)
			| Self::TableNew(_)
			| Self::MemoryNew(_) => unreachable!("integer `Expression` expected"),

			Self::Match(_)
			| Self::Local(_)
			| Self::I32(_)
			| Self::Call(_)
			| Self::Apply0Arguments(_)
			| Self::Apply1Argument(_)
			| Self::Apply2Arguments(_)
			| Self::Apply3Arguments(_)
			| Self::Apply4Arguments(_)
			| Self::Apply5Arguments(_)
			| Self::Extract(_)
			| Self::Field(_)
			| Self::Index(_) => self.into_boolean_unchecked(),

			Self::Trap | Self::RefIsNull(_) => self,

			Self::BooleanToInteger(boolean_to_integer) => boolean_to_integer.source,
		}
	}
}
