use wasmparser::{ConstExpr, Operator};

#[derive(Clone, Copy)]
pub enum ConstantExpression {
	I32(i32),
	I64(i64),
	F32(f32),
	F64(f64),
	RefNull,
	RefFunction(u32),
	GlobalGet(u32),
}

impl ConstantExpression {
	#[must_use]
	#[expect(
		clippy::wildcard_enum_match_arm,
		reason = "all remaining constant-expression operators are unsupported"
	)]
	pub fn parse(expression: &ConstExpr<'_>) -> Self {
		let mut reader = expression.get_operators_reader();

		match reader.read().unwrap() {
			Operator::GlobalGet { global_index } => Self::GlobalGet(global_index),
			Operator::I32Const { value } => Self::I32(value),
			Operator::I64Const { value } => Self::I64(value),
			Operator::F32Const { value } => Self::F32(value.into()),
			Operator::F64Const { value } => Self::F64(value.into()),
			Operator::RefNull { .. } => Self::RefNull,
			Operator::RefFunc { function_index } => Self::RefFunction(function_index),
			operator => unimplemented!("{operator:?}"),
		}
	}
}
