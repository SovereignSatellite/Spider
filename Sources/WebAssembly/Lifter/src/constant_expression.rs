use wasmparser::{ConstExpr, Operator};

use ir_graph::{Link, Node, operation::MutableGet};

use super::{entities::Entities, graph_builder::GraphBuilder};

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
		reason = "catch-all for operators invalid in constant expressions"
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

	pub fn emit(self, builder: &mut GraphBuilder) -> u16 {
		match self {
			Self::I32(value) => builder.emit_i32_constant(value),
			Self::I64(value) => builder.emit_i64_constant(value),
			Self::F32(value) => builder.emit_f32_constant(value),
			Self::F64(value) => builder.emit_f64_constant(value),
			Self::RefNull => builder.emit_ref_null(),
			Self::RefFunction(function) => builder.emit_ref_function(function),
			Self::GlobalGet(global) => builder.emit_global_get(global),
		}
	}

	pub fn emit_into(self, nodes: &mut Vec<Node>, entities: &Entities) -> Link {
		match self {
			Self::I32(value) => Node::add_i32_into(nodes, value),
			Self::I64(value) => Node::add_i64_into(nodes, value),
			Self::F32(value) => Node::add_f32_into(nodes, value),
			Self::F64(value) => Node::add_f64_into(nodes, value),
			Self::RefNull => Node::add_null_into(nodes),
			Self::RefFunction(function) => entities.emit_function_reference(nodes, function),
			Self::GlobalGet(global) => {
				let Ok(index) = usize::try_from(global) else {
					unreachable!()
				};

				MutableGet::add_into(nodes, entities.globals[index]).0
			}
		}
	}
}
