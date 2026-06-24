use alloc::sync::Arc;
use core::any::Any;

use hashbrown::HashMap;

use ir_allocator::DEFERRED;
use ir_graph::{
	Link,
	foreign::Foreign,
	operation::{self, ExtendType, LoadType, StoreType, integer, number},
};
use luau_foreign::BufferStore;
use luau_tree::{
	expression::{
		Aggregate, Apply, BooleanToInteger, BufferLength, Expression, Extract, GlobalGet,
		GlobalNew, Index, Infix, Local, Match, Name, Prefix, RefIsNull, TableNew, TableSize,
		VectorX,
	},
	statement::Sequence,
};
use turing_machine_foreign::Ask as TuringAsk;
use web_assembly_foreign::Import as WasmImport;

use super::policy::PHYSICAL_REGISTERS;

fn register_to_local(register: u32) -> Local {
	if register < PHYSICAL_REGISTERS {
		Local::Fast {
			name: Name { id: register },
		}
	} else {
		let offset = u16::try_from(register - PHYSICAL_REGISTERS).unwrap();

		Local::Slow { offset }
	}
}

const fn integer_unary_name(node: &integer::UnaryOperation) -> &'static str {
	match (node.kind, node.operator) {
		(integer::Type::I32, integer::UnaryOperator::CountOnes) => "rt_count_ones_i32",
		(integer::Type::I32, integer::UnaryOperator::LeadingZeros) => "rt_leading_zeros_i32",
		(integer::Type::I32, integer::UnaryOperator::TrailingZeros) => "rt_trailing_zeros_i32",
		(integer::Type::I64, integer::UnaryOperator::CountOnes) => "rt_count_ones_i64",
		(integer::Type::I64, integer::UnaryOperator::LeadingZeros) => "rt_leading_zeros_i64",
		(integer::Type::I64, integer::UnaryOperator::TrailingZeros) => "rt_trailing_zeros_i64",
	}
}

#[expect(
	clippy::too_many_lines,
	reason = "exhaustive match over integer binary operators"
)]
const fn integer_binary_name(node: &integer::BinaryOperation) -> &'static str {
	match (node.kind, node.operator) {
		(integer::Type::I32, integer::BinaryOperator::Add) => "rt_add_i32",
		(integer::Type::I32, integer::BinaryOperator::Subtract) => "rt_subtract_i32",
		(integer::Type::I32, integer::BinaryOperator::Multiply) => "rt_multiply_i32",
		(integer::Type::I32, integer::BinaryOperator::Divide { is_signed: true }) => {
			"rt_divide_s32"
		}
		(integer::Type::I32, integer::BinaryOperator::Divide { is_signed: false }) => {
			"rt_divide_u32"
		}
		(integer::Type::I32, integer::BinaryOperator::Remainder { is_signed: true }) => {
			"rt_remainder_s32"
		}
		(integer::Type::I32, integer::BinaryOperator::Remainder { is_signed: false }) => {
			"rt_remainder_u32"
		}
		(integer::Type::I32, integer::BinaryOperator::And) => "rt_and_i32",
		(integer::Type::I32, integer::BinaryOperator::Or) => "rt_or_i32",
		(integer::Type::I32, integer::BinaryOperator::ExclusiveOr) => "rt_exclusive_or_i32",
		(integer::Type::I32, integer::BinaryOperator::ShiftLeft) => "rt_shift_left_i32",
		(integer::Type::I32, integer::BinaryOperator::ShiftRight { is_signed: true }) => {
			"rt_shift_right_s32"
		}
		(integer::Type::I32, integer::BinaryOperator::ShiftRight { is_signed: false }) => {
			"rt_shift_right_u32"
		}
		(integer::Type::I32, integer::BinaryOperator::RotateLeft) => "rt_rotate_left_i32",
		(integer::Type::I32, integer::BinaryOperator::RotateRight) => "rt_rotate_right_i32",
		(integer::Type::I64, integer::BinaryOperator::Add) => "rt_add_i64",
		(integer::Type::I64, integer::BinaryOperator::Subtract) => "rt_subtract_i64",
		(integer::Type::I64, integer::BinaryOperator::Multiply) => "rt_multiply_i64",
		(integer::Type::I64, integer::BinaryOperator::Divide { is_signed: true }) => {
			"rt_divide_s64"
		}
		(integer::Type::I64, integer::BinaryOperator::Divide { is_signed: false }) => {
			"rt_divide_u64"
		}
		(integer::Type::I64, integer::BinaryOperator::Remainder { is_signed: true }) => {
			"rt_remainder_s64"
		}
		(integer::Type::I64, integer::BinaryOperator::Remainder { is_signed: false }) => {
			"rt_remainder_u64"
		}
		(integer::Type::I64, integer::BinaryOperator::And) => "rt_and_i64",
		(integer::Type::I64, integer::BinaryOperator::Or) => "rt_or_i64",
		(integer::Type::I64, integer::BinaryOperator::ExclusiveOr) => "rt_exclusive_or_i64",
		(integer::Type::I64, integer::BinaryOperator::ShiftLeft) => "rt_shift_left_i64",
		(integer::Type::I64, integer::BinaryOperator::ShiftRight { is_signed: true }) => {
			"rt_shift_right_s64"
		}
		(integer::Type::I64, integer::BinaryOperator::ShiftRight { is_signed: false }) => {
			"rt_shift_right_u64"
		}
		(integer::Type::I64, integer::BinaryOperator::RotateLeft) => "rt_rotate_left_i64",
		(integer::Type::I64, integer::BinaryOperator::RotateRight) => "rt_rotate_right_i64",
	}
}

#[expect(
	clippy::too_many_lines,
	reason = "exhaustive match over integer compare operators"
)]
const fn integer_compare_name(node: &integer::CompareOperation) -> &'static str {
	match (node.kind, node.operator) {
		(integer::Type::I32, integer::CompareOperator::Equal) => "rt_equal_i32",
		(integer::Type::I32, integer::CompareOperator::NotEqual) => "rt_not_equal_i32",
		(integer::Type::I32, integer::CompareOperator::LessThan { is_signed: true }) => {
			"rt_less_than_s32"
		}
		(integer::Type::I32, integer::CompareOperator::LessThan { is_signed: false }) => {
			"rt_less_than_u32"
		}
		(integer::Type::I32, integer::CompareOperator::GreaterThan { is_signed: true }) => {
			"rt_greater_than_s32"
		}
		(integer::Type::I32, integer::CompareOperator::GreaterThan { is_signed: false }) => {
			"rt_greater_than_u32"
		}
		(integer::Type::I32, integer::CompareOperator::LessThanEqual { is_signed: true }) => {
			"rt_less_than_equal_s32"
		}
		(integer::Type::I32, integer::CompareOperator::LessThanEqual { is_signed: false }) => {
			"rt_less_than_equal_u32"
		}
		(integer::Type::I32, integer::CompareOperator::GreaterThanEqual { is_signed: true }) => {
			"rt_greater_than_equal_s32"
		}
		(integer::Type::I32, integer::CompareOperator::GreaterThanEqual { is_signed: false }) => {
			"rt_greater_than_equal_u32"
		}
		(integer::Type::I64, integer::CompareOperator::Equal) => "rt_equal_i64",
		(integer::Type::I64, integer::CompareOperator::NotEqual) => "rt_not_equal_i64",
		(integer::Type::I64, integer::CompareOperator::LessThan { is_signed: true }) => {
			"rt_less_than_s64"
		}
		(integer::Type::I64, integer::CompareOperator::LessThan { is_signed: false }) => {
			"rt_less_than_u64"
		}
		(integer::Type::I64, integer::CompareOperator::GreaterThan { is_signed: true }) => {
			"rt_greater_than_s64"
		}
		(integer::Type::I64, integer::CompareOperator::GreaterThan { is_signed: false }) => {
			"rt_greater_than_u64"
		}
		(integer::Type::I64, integer::CompareOperator::LessThanEqual { is_signed: true }) => {
			"rt_less_than_equal_s64"
		}
		(integer::Type::I64, integer::CompareOperator::LessThanEqual { is_signed: false }) => {
			"rt_less_than_equal_u64"
		}
		(integer::Type::I64, integer::CompareOperator::GreaterThanEqual { is_signed: true }) => {
			"rt_greater_than_equal_s64"
		}
		(integer::Type::I64, integer::CompareOperator::GreaterThanEqual { is_signed: false }) => {
			"rt_greater_than_equal_u64"
		}
	}
}

const fn integer_extend_name(node: &operation::IntegerSignExtend) -> &'static str {
	match node.kind {
		ExtendType::I32_S8 => "rt_extend_s8_to_i32",
		ExtendType::I32_S16 => "rt_extend_s16_to_i32",
		ExtendType::I64_S8 => "rt_extend_s8_to_i64",
		ExtendType::I64_S16 => "rt_extend_s16_to_i64",
		ExtendType::I64_S32 => "rt_extend_s32_to_i64",
	}
}

const fn integer_convert_name(node: &operation::IntegerConvertToNumber) -> &'static str {
	match (node.from, node.to, node.is_signed) {
		(integer::Type::I32, number::Type::F32, true) => "rt_convert_s32_to_f32",
		(integer::Type::I32, number::Type::F32, false) => "rt_convert_u32_to_f32",
		(integer::Type::I32, number::Type::F64, true) => "rt_convert_s32_to_f64",
		(integer::Type::I32, number::Type::F64, false) => "rt_convert_u32_to_f64",
		(integer::Type::I64, number::Type::F32, true) => "rt_convert_s64_to_f32",
		(integer::Type::I64, number::Type::F32, false) => "rt_convert_u64_to_f32",
		(integer::Type::I64, number::Type::F64, true) => "rt_convert_s64_to_f64",
		(integer::Type::I64, number::Type::F64, false) => "rt_convert_u64_to_f64",
	}
}

const fn integer_transmute_name(node: &operation::IntegerTransmuteToNumber) -> &'static str {
	match node.from {
		integer::Type::I32 => "rt_transmute_i32_to_f32",
		integer::Type::I64 => "rt_transmute_i64_to_f64",
	}
}

const fn number_unary_name(node: &number::UnaryOperation) -> &'static str {
	match (node.kind, node.operator) {
		(number::Type::F32, number::UnaryOperator::Absolute) => "rt_absolute_f32",
		(number::Type::F32, number::UnaryOperator::Negate) => "rt_negate_f32",
		(number::Type::F32, number::UnaryOperator::SquareRoot) => "rt_square_root_f32",
		(number::Type::F32, number::UnaryOperator::RoundUp) => "rt_round_up_f32",
		(number::Type::F32, number::UnaryOperator::RoundDown) => "rt_round_down_f32",
		(number::Type::F32, number::UnaryOperator::Truncate) => "rt_truncate_f32",
		(number::Type::F32, number::UnaryOperator::Nearest) => "rt_nearest_f32",
		(number::Type::F64, number::UnaryOperator::Absolute) => "rt_absolute_f64",
		(number::Type::F64, number::UnaryOperator::Negate) => "rt_negate_f64",
		(number::Type::F64, number::UnaryOperator::SquareRoot) => "rt_square_root_f64",
		(number::Type::F64, number::UnaryOperator::RoundUp) => "rt_round_up_f64",
		(number::Type::F64, number::UnaryOperator::RoundDown) => "rt_round_down_f64",
		(number::Type::F64, number::UnaryOperator::Truncate) => "rt_truncate_f64",
		(number::Type::F64, number::UnaryOperator::Nearest) => "rt_nearest_f64",
	}
}

const fn number_binary_name(node: &number::BinaryOperation) -> &'static str {
	match (node.kind, node.operator) {
		(number::Type::F32, number::BinaryOperator::Add) => "rt_add_f32",
		(number::Type::F32, number::BinaryOperator::Subtract) => "rt_subtract_f32",
		(number::Type::F32, number::BinaryOperator::Multiply) => "rt_multiply_f32",
		(number::Type::F32, number::BinaryOperator::Divide) => "rt_divide_f32",
		(number::Type::F32, number::BinaryOperator::Minimum) => "rt_minimum_f32",
		(number::Type::F32, number::BinaryOperator::Maximum) => "rt_maximum_f32",
		(number::Type::F32, number::BinaryOperator::CopySign) => "rt_copy_sign_f32",
		(number::Type::F64, number::BinaryOperator::Add) => "rt_add_f64",
		(number::Type::F64, number::BinaryOperator::Subtract) => "rt_subtract_f64",
		(number::Type::F64, number::BinaryOperator::Multiply) => "rt_multiply_f64",
		(number::Type::F64, number::BinaryOperator::Divide) => "rt_divide_f64",
		(number::Type::F64, number::BinaryOperator::Minimum) => "rt_minimum_f64",
		(number::Type::F64, number::BinaryOperator::Maximum) => "rt_maximum_f64",
		(number::Type::F64, number::BinaryOperator::CopySign) => "rt_copy_sign_f64",
	}
}

const fn number_compare_name(node: &number::CompareOperation) -> &'static str {
	match (node.kind, node.operator) {
		(number::Type::F32, number::CompareOperator::Equal) => "rt_equal_f32",
		(number::Type::F32, number::CompareOperator::NotEqual) => "rt_not_equal_f32",
		(number::Type::F32, number::CompareOperator::LessThan) => "rt_less_than_f32",
		(number::Type::F32, number::CompareOperator::GreaterThan) => "rt_greater_than_f32",
		(number::Type::F32, number::CompareOperator::LessThanEqual) => "rt_less_than_equal_f32",
		(number::Type::F32, number::CompareOperator::GreaterThanEqual) => {
			"rt_greater_than_equal_f32"
		}
		(number::Type::F64, number::CompareOperator::Equal) => "rt_equal_f64",
		(number::Type::F64, number::CompareOperator::NotEqual) => "rt_not_equal_f64",
		(number::Type::F64, number::CompareOperator::LessThan) => "rt_less_than_f64",
		(number::Type::F64, number::CompareOperator::GreaterThan) => "rt_greater_than_f64",
		(number::Type::F64, number::CompareOperator::LessThanEqual) => "rt_less_than_equal_f64",
		(number::Type::F64, number::CompareOperator::GreaterThanEqual) => {
			"rt_greater_than_equal_f64"
		}
	}
}

const fn number_truncate_name(node: &operation::NumberTruncateToInteger) -> &'static str {
	match (node.from, node.to, node.is_signed, node.is_saturating) {
		(number::Type::F32, integer::Type::I32, true, true) => "rt_saturate_f32_to_s32",
		(number::Type::F32, integer::Type::I32, true, false) => "rt_truncate_f32_to_s32",
		(number::Type::F32, integer::Type::I32, false, true) => "rt_saturate_f32_to_u32",
		(number::Type::F32, integer::Type::I32, false, false) => "rt_truncate_f32_to_u32",
		(number::Type::F32, integer::Type::I64, true, true) => "rt_saturate_f32_to_s64",
		(number::Type::F32, integer::Type::I64, true, false) => "rt_truncate_f32_to_s64",
		(number::Type::F32, integer::Type::I64, false, true) => "rt_saturate_f32_to_u64",
		(number::Type::F32, integer::Type::I64, false, false) => "rt_truncate_f32_to_u64",
		(number::Type::F64, integer::Type::I32, true, true) => "rt_saturate_f64_to_s32",
		(number::Type::F64, integer::Type::I32, true, false) => "rt_truncate_f64_to_s32",
		(number::Type::F64, integer::Type::I32, false, true) => "rt_saturate_f64_to_u32",
		(number::Type::F64, integer::Type::I32, false, false) => "rt_truncate_f64_to_u32",
		(number::Type::F64, integer::Type::I64, true, true) => "rt_saturate_f64_to_s64",
		(number::Type::F64, integer::Type::I64, true, false) => "rt_truncate_f64_to_s64",
		(number::Type::F64, integer::Type::I64, false, true) => "rt_saturate_f64_to_u64",
		(number::Type::F64, integer::Type::I64, false, false) => "rt_truncate_f64_to_u64",
	}
}

const fn number_transmute_name(node: &operation::NumberTransmuteToInteger) -> &'static str {
	match node.from {
		number::Type::F32 => "rt_transmute_f32_to_i32",
		number::Type::F64 => "rt_transmute_f64_to_i64",
	}
}

const fn memory_load_name(kind: LoadType) -> &'static str {
	match kind {
		LoadType::I32_S8 => "rt_load_i32_from_s8",
		LoadType::I32_U8 => "rt_load_i32_from_u8",
		LoadType::I32_S16 => "rt_load_i32_from_s16",
		LoadType::I32_U16 => "rt_load_i32_from_u16",
		LoadType::I32 => "rt_load_i32",
		LoadType::I64_S8 => "rt_load_i64_from_s8",
		LoadType::I64_U8 => "rt_load_i64_from_u8",
		LoadType::I64_S16 => "rt_load_i64_from_s16",
		LoadType::I64_U16 => "rt_load_i64_from_u16",
		LoadType::I64_S32 => "rt_load_i64_from_s32",
		LoadType::I64_U32 => "rt_load_i64_from_u32",
		LoadType::I64 => "rt_load_i64",
		LoadType::F32 => "rt_load_f32",
		LoadType::F64 => "rt_load_f64",
	}
}

pub const fn memory_store_name(kind: StoreType) -> &'static str {
	match kind {
		StoreType::I32_I8 => "rt_store_i32_into_i8",
		StoreType::I32_I16 => "rt_store_i32_into_i16",
		StoreType::I32 => "rt_store_i32",
		StoreType::I64_I8 => "rt_store_i64_into_i8",
		StoreType::I64_I16 => "rt_store_i64_into_i16",
		StoreType::I64_I32 => "rt_store_i64_into_i32",
		StoreType::I64 => "rt_store_i64",
		StoreType::F32 => "rt_store_f32",
		StoreType::F64 => "rt_store_f64",
	}
}

pub fn build_match_expression(condition: Expression, branches: Vec<Sequence>) -> Expression {
	let branches = branches
		.into_iter()
		.map(Sequence::into_assign_source)
		.collect();

	Expression::Match(
		Match {
			branches,
			condition,
		}
		.into(),
	)
}

pub fn build_foreign(foreign: &dyn Foreign) -> Expression {
	let any: &dyn Any = foreign;

	if let Some(node) = any.downcast_ref::<WasmImport>() {
		let arguments = [
			Expression::String(Arc::clone(&node.namespace)),
			Expression::String(Arc::clone(&node.identifier)),
		];

		return Expression::Apply2Arguments(
			Apply {
				name: "rt_import",
				arguments,
			}
			.into(),
		);
	}

	if any.downcast_ref::<TuringAsk>().is_some() {
		return Expression::Apply0Arguments(
			Apply {
				name: "rt_turing_ask",
				arguments: [],
			}
			.into(),
		);
	}

	unimplemented!("`{}` has no expression form", foreign.identifier())
}

pub struct DataHandler {
	arena: ir_allocator::Arena,
	deferred_expressions: HashMap<(u32, u32), Expression>,
}

impl DataHandler {
	#[must_use]
	pub fn new() -> Self {
		Self {
			arena: ir_allocator::Arena::new(),
			deferred_expressions: HashMap::new(),
		}
	}

	pub fn install(&mut self, arena: ir_allocator::Arena) {
		self.arena = arena;
	}

	#[must_use]
	pub fn node_count(&self, region: u32) -> usize {
		self.arena.node_count(region)
	}

	#[must_use]
	pub fn is_deferred(&self, region: u32, id: u32) -> bool {
		self.arena.register(region, Link(id, 0)) == DEFERRED
	}

	pub fn store(&mut self, region: u32, id: u32, expression: Expression) {
		self.deferred_expressions.insert((region, id), expression);
	}

	#[must_use]
	pub fn local_of(&self, region: u32, link: Link) -> Local {
		register_to_local(self.arena.register(region, link))
	}

	#[must_use]
	pub fn port_locals(&self, region: u32, id: u32, count: u16) -> Vec<Local> {
		(0..count)
			.map(|port| self.local_of(region, Link(id, port)))
			.collect()
	}

	pub fn load(&mut self, region: u32, link: Link) -> Expression {
		let register = self.arena.register(region, link);

		if register == DEFERRED {
			return self
				.deferred_expressions
				.remove(&(region, link.0))
				.expect("a deferred port must have a stored expression");
		}

		Expression::Local(register_to_local(register))
	}

	pub fn load_all(&mut self, region: u32, sources: &[Link]) -> Vec<Expression> {
		sources
			.iter()
			.map(|&link| self.load(region, link))
			.collect()
	}

	fn load_each<const N: usize>(&mut self, region: u32, sources: [Link; N]) -> [Expression; N] {
		sources.map(|link| self.load(region, link))
	}

	pub fn build_apply_1(
		&mut self,
		region: u32,
		name: &'static str,
		sources: [Link; 1],
	) -> Expression {
		let arguments = self.load_each(region, sources);
		let expression = Apply { name, arguments };

		Expression::Apply1Argument(expression.into())
	}

	pub fn build_apply_2(
		&mut self,
		region: u32,
		name: &'static str,
		sources: [Link; 2],
	) -> Expression {
		let arguments = self.load_each(region, sources);
		let expression = Apply { name, arguments };

		Expression::Apply2Arguments(expression.into())
	}

	pub fn build_apply_3(
		&mut self,
		region: u32,
		name: &'static str,
		sources: [Link; 3],
	) -> Expression {
		let arguments = self.load_each(region, sources);
		let expression = Apply { name, arguments };

		Expression::Apply3Arguments(expression.into())
	}

	pub fn build_apply_4(
		&mut self,
		region: u32,
		name: &'static str,
		sources: [Link; 4],
	) -> Expression {
		let arguments = self.load_each(region, sources);
		let expression = Apply { name, arguments };

		Expression::Apply4Arguments(expression.into())
	}

	pub fn build_apply_5(
		&mut self,
		region: u32,
		name: &'static str,
		sources: [Link; 5],
	) -> Expression {
		let arguments = self.load_each(region, sources);
		let expression = Apply { name, arguments };

		Expression::Apply5Arguments(expression.into())
	}

	pub fn build_buffer_store(
		&mut self,
		region: u32,
		reference: Link,
		node: BufferStore,
	) -> Expression {
		let buffer = self.build_extract(
			region,
			operation::Extract {
				source: reference,
				index: 0,
			},
		);
		let arguments = [
			buffer,
			self.load(region, node.offset),
			self.load(region, node.value),
		];
		let expression = Apply {
			name: node.name,
			arguments,
		};

		Expression::Apply3Arguments(expression.into())
	}

	pub fn build_buffer_length(&mut self, region: u32, source: Link) -> Expression {
		let source = self.load(region, source);

		Expression::BufferLength(BufferLength { source }.into())
	}

	pub fn build_vector_create(&mut self, region: u32, source: Link) -> Expression {
		let arguments = [
			self.load(region, source),
			Expression::I32(0),
			Expression::I32(0),
		];
		let expression = Apply {
			name: "vector_create",
			arguments,
		};

		Expression::Apply3Arguments(expression.into())
	}

	pub fn build_vector_x(&mut self, region: u32, source: Link) -> Expression {
		let source = self.load(region, source);

		Expression::VectorX(VectorX { source }.into())
	}

	pub fn build_infix(
		&mut self,
		region: u32,
		operator: &'static str,
		lhs: Link,
		rhs: Link,
	) -> Expression {
		let expression = Infix {
			operator,
			lhs: self.load(region, lhs),
			rhs: self.load(region, rhs),
		};

		Expression::Infix(expression.into())
	}

	pub fn build_prefix(
		&mut self,
		region: u32,
		operator: &'static str,
		source: Link,
	) -> Expression {
		let expression = Prefix {
			operator,
			source: self.load(region, source),
		};

		Expression::Prefix(expression.into())
	}

	fn wrap_boolean(source: Expression) -> Expression {
		let expression = BooleanToInteger { source };

		Expression::BooleanToInteger(expression.into())
	}

	pub fn build_boolean_to_integer(&mut self, region: u32, source: Link) -> Expression {
		let source = self.load(region, source);

		Self::wrap_boolean(source)
	}

	pub fn build_ref_is_null(&mut self, region: u32, node: operation::RefIsNull) -> Expression {
		let expression = RefIsNull {
			source: self.load(region, node.source),
		};

		Self::wrap_boolean(Expression::RefIsNull(expression.into()))
	}

	pub fn build_integer_unary_operation(
		&mut self,
		region: u32,
		node: integer::UnaryOperation,
	) -> Expression {
		self.build_apply_1(region, integer_unary_name(&node), [node.source])
	}

	pub fn build_integer_binary_operation(
		&mut self,
		region: u32,
		node: integer::BinaryOperation,
	) -> Expression {
		self.build_apply_2(region, integer_binary_name(&node), [node.lhs, node.rhs])
	}

	pub fn build_integer_compare_operation(
		&mut self,
		region: u32,
		node: integer::CompareOperation,
	) -> Expression {
		let inner = self.build_apply_2(region, integer_compare_name(&node), [node.lhs, node.rhs]);

		Self::wrap_boolean(inner)
	}

	pub fn build_integer_narrow(
		&mut self,
		region: u32,
		node: operation::IntegerNarrow,
	) -> Expression {
		self.build_apply_1(region, "rt_narrow_i64", [node.source])
	}

	pub fn build_integer_widen(
		&mut self,
		region: u32,
		node: operation::IntegerWiden,
	) -> Expression {
		self.build_apply_1(region, "rt_widen_i32", [node.source])
	}

	pub fn build_integer_sign_extend(
		&mut self,
		region: u32,
		node: operation::IntegerSignExtend,
	) -> Expression {
		self.build_apply_1(region, integer_extend_name(&node), [node.source])
	}

	pub fn build_integer_convert_to_number(
		&mut self,
		region: u32,
		node: operation::IntegerConvertToNumber,
	) -> Expression {
		self.build_apply_1(region, integer_convert_name(&node), [node.source])
	}

	pub fn build_integer_transmute_to_number(
		&mut self,
		region: u32,
		node: operation::IntegerTransmuteToNumber,
	) -> Expression {
		self.build_apply_1(region, integer_transmute_name(&node), [node.source])
	}

	pub fn build_number_unary_operation(
		&mut self,
		region: u32,
		node: number::UnaryOperation,
	) -> Expression {
		self.build_apply_1(region, number_unary_name(&node), [node.source])
	}

	pub fn build_number_binary_operation(
		&mut self,
		region: u32,
		node: number::BinaryOperation,
	) -> Expression {
		self.build_apply_2(region, number_binary_name(&node), [node.lhs, node.rhs])
	}

	pub fn build_number_compare_operation(
		&mut self,
		region: u32,
		node: number::CompareOperation,
	) -> Expression {
		let inner = self.build_apply_2(region, number_compare_name(&node), [node.lhs, node.rhs]);

		Self::wrap_boolean(inner)
	}

	pub fn build_number_narrow(
		&mut self,
		region: u32,
		node: operation::NumberNarrow,
	) -> Expression {
		self.build_apply_1(region, "rt_narrow_f64", [node.source])
	}

	pub fn build_number_widen(&mut self, region: u32, node: operation::NumberWiden) -> Expression {
		self.build_apply_1(region, "rt_widen_f32", [node.source])
	}

	pub fn build_number_truncate_to_integer(
		&mut self,
		region: u32,
		node: operation::NumberTruncateToInteger,
	) -> Expression {
		self.build_apply_1(region, number_truncate_name(&node), [node.source])
	}

	pub fn build_number_transmute_to_integer(
		&mut self,
		region: u32,
		node: operation::NumberTransmuteToInteger,
	) -> Expression {
		self.build_apply_1(region, number_transmute_name(&node), [node.source])
	}

	pub fn build_mutable_new(&mut self, region: u32, node: operation::MutableNew) -> Expression {
		let expression = GlobalNew {
			initializer: self.load(region, node.initializer),
		};

		Expression::GlobalNew(expression.into())
	}

	pub fn build_mutable_get(&mut self, region: u32, reference: Link) -> Expression {
		let expression = GlobalGet {
			source: self.load(region, reference),
		};

		Expression::GlobalGet(expression.into())
	}

	pub fn build_aggregate(&mut self, region: u32, node: &operation::Aggregate) -> Expression {
		let fields = node
			.fields
			.iter()
			.map(|&link| self.load(region, link))
			.collect();
		let expression = Aggregate { fields };

		Expression::Aggregate(expression.into())
	}

	pub fn build_extract(&mut self, region: u32, node: operation::Extract) -> Expression {
		let expression = Extract {
			source: self.load(region, node.source),
			index: node.index,
		};

		Expression::Extract(expression.into())
	}

	pub fn build_table_new(&mut self, region: u32, node: &operation::TableNew) -> Expression {
		let initializer = node
			.initializer
			.iter()
			.map(|&(link, offset)| (self.load(region, link), offset))
			.collect();
		let expression = TableNew {
			initializer,
			minimum: node.minimum,
			maximum: node.maximum,
		};

		Expression::TableNew(expression.into())
	}

	pub fn build_table_get(&mut self, region: u32, reference: Link, offset: Link) -> Expression {
		self.build_apply_2(region, "rt_table_get", [reference, offset])
	}

	pub fn build_table_size(&mut self, region: u32, reference: Link) -> Expression {
		self.build_table_length(region, reference)
	}

	pub fn build_table_length(&mut self, region: u32, source: Link) -> Expression {
		let source = self.load(region, source);

		Expression::TableSize(TableSize { source }.into())
	}

	pub fn build_index(&mut self, region: u32, source: Link, offset: Link) -> Expression {
		let [source, offset] = self.load_each(region, [source, offset]);

		Expression::Index(Index { source, offset }.into())
	}

	pub fn build_table_grow(
		&mut self,
		region: u32,
		reference: Link,
		initializer: Link,
		size: Link,
	) -> Expression {
		self.build_apply_3(region, "rt_table_grow", [reference, initializer, size])
	}

	pub fn build_memory_load(
		&mut self,
		region: u32,
		reference: Link,
		offset: Link,
		kind: LoadType,
	) -> Expression {
		self.build_apply_2(region, memory_load_name(kind), [reference, offset])
	}

	pub fn build_memory_size(&mut self, region: u32, reference: Link) -> Expression {
		self.build_apply_1(region, "rt_memory_size", [reference])
	}

	pub fn build_memory_grow(&mut self, region: u32, reference: Link, size: Link) -> Expression {
		self.build_apply_2(region, "rt_memory_grow", [reference, size])
	}
}

impl Default for DataHandler {
	fn default() -> Self {
		Self::new()
	}
}
