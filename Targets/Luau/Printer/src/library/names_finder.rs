use std::ops::ControlFlow;

use luau_tree::{
	expression::{
		DataNew, ElementsNew, Expression, ExtendType, GlobalGet, GlobalNew, IntegerBinaryOperation,
		IntegerBinaryOperator, IntegerCompareOperation, IntegerCompareOperator,
		IntegerConvertToNumber, IntegerExtend, IntegerNarrow, IntegerTransmuteToNumber,
		IntegerType, IntegerUnaryOperation, IntegerUnaryOperator, IntegerWiden, LoadType,
		MemoryGrow, MemoryLoad, MemoryNew, MemorySize, NumberBinaryOperation, NumberBinaryOperator,
		NumberCompareOperation, NumberCompareOperator, NumberNarrow, NumberTransmuteToInteger,
		NumberTruncateToInteger, NumberType, NumberUnaryOperation, NumberUnaryOperator,
		NumberWiden, TableGet, TableGrow, TableNew, TableSize,
	},
	statement::{
		DataDrop, ElementsDrop, GlobalSet, MemoryCopy, MemoryFill, MemoryInit, MemoryStore,
		Statement, StoreType, TableCopy, TableFill, TableInit, TableSet,
	},
	visitor::Visitor,
	LuauTree,
};

pub trait NeedsName {
	fn needs_name(&self) -> &'static str;
}

impl NeedsName for i32 {
	fn needs_name(&self) -> &'static str {
		""
	}
}

impl NeedsName for i64 {
	fn needs_name(&self) -> &'static str {
		"create_i64_from_u32"
	}
}

impl NeedsName for f32 {
	fn needs_name(&self) -> &'static str {
		if self.is_finite() {
			"vector_create"
		} else {
			"transmute_i32_to_f32"
		}
	}
}

impl NeedsName for f64 {
	fn needs_name(&self) -> &'static str {
		if self.is_finite() {
			""
		} else {
			"create_f64_from_u32"
		}
	}
}

impl NeedsName for IntegerUnaryOperation {
	fn needs_name(&self) -> &'static str {
		let Self {
			r#type, operator, ..
		} = *self;

		match (r#type, operator) {
			(IntegerType::I32, IntegerUnaryOperator::CountOnes) => "count_ones_i32",
			(IntegerType::I32, IntegerUnaryOperator::LeadingZeroes) => "leading_zeroes_i32",
			(IntegerType::I32, IntegerUnaryOperator::TrailingZeroes) => "trailing_zeroes_i32",
			(IntegerType::I64, IntegerUnaryOperator::CountOnes) => "count_ones_i64",
			(IntegerType::I64, IntegerUnaryOperator::LeadingZeroes) => "leading_zeroes_i64",
			(IntegerType::I64, IntegerUnaryOperator::TrailingZeroes) => "trailing_zeroes_i64",
		}
	}
}

impl NeedsName for IntegerBinaryOperation {
	fn needs_name(&self) -> &'static str {
		let Self {
			r#type, operator, ..
		} = *self;

		match (r#type, operator) {
			(IntegerType::I32, IntegerBinaryOperator::Add) => "add_i32",
			(IntegerType::I32, IntegerBinaryOperator::Subtract) => "subtract_i32",
			(IntegerType::I32, IntegerBinaryOperator::Multiply) => "multiply_i32",
			(IntegerType::I32, IntegerBinaryOperator::Divide { signed: true }) => "divide_s32",
			(IntegerType::I32, IntegerBinaryOperator::Divide { signed: false }) => "divide_u32",
			(IntegerType::I32, IntegerBinaryOperator::Remainder { signed: true }) => {
				"remainder_s32"
			}
			(IntegerType::I32, IntegerBinaryOperator::Remainder { signed: false }) => {
				"remainder_u32"
			}
			(IntegerType::I32, IntegerBinaryOperator::And) => "and_i32",
			(IntegerType::I32, IntegerBinaryOperator::Or) => "or_i32",
			(IntegerType::I32, IntegerBinaryOperator::ExclusiveOr) => "exclusive_or_i32",
			(IntegerType::I32, IntegerBinaryOperator::ShiftLeft) => "shift_left_i32",
			(IntegerType::I32, IntegerBinaryOperator::ShiftRight { signed: true }) => {
				"shift_right_s32"
			}
			(IntegerType::I32, IntegerBinaryOperator::ShiftRight { signed: false }) => {
				"shift_right_u32"
			}
			(IntegerType::I32, IntegerBinaryOperator::RotateLeft) => "rotate_left_i32",
			(IntegerType::I32, IntegerBinaryOperator::RotateRight) => "rotate_right_i32",
			(IntegerType::I64, IntegerBinaryOperator::Add) => "add_i64",
			(IntegerType::I64, IntegerBinaryOperator::Subtract) => "subtract_i64",
			(IntegerType::I64, IntegerBinaryOperator::Multiply) => "multiply_i64",
			(IntegerType::I64, IntegerBinaryOperator::Divide { signed: true }) => "divide_s64",
			(IntegerType::I64, IntegerBinaryOperator::Divide { signed: false }) => "divide_u64",
			(IntegerType::I64, IntegerBinaryOperator::Remainder { signed: true }) => {
				"remainder_s64"
			}
			(IntegerType::I64, IntegerBinaryOperator::Remainder { signed: false }) => {
				"remainder_u64"
			}
			(IntegerType::I64, IntegerBinaryOperator::And) => "and_i64",
			(IntegerType::I64, IntegerBinaryOperator::Or) => "or_i64",
			(IntegerType::I64, IntegerBinaryOperator::ExclusiveOr) => "exclusive_or_i64",
			(IntegerType::I64, IntegerBinaryOperator::ShiftLeft) => "shift_left_i64",
			(IntegerType::I64, IntegerBinaryOperator::ShiftRight { signed: true }) => {
				"shift_right_s64"
			}
			(IntegerType::I64, IntegerBinaryOperator::ShiftRight { signed: false }) => {
				"shift_right_u64"
			}
			(IntegerType::I64, IntegerBinaryOperator::RotateLeft) => "rotate_left_i64",
			(IntegerType::I64, IntegerBinaryOperator::RotateRight) => "rotate_right_i64",
		}
	}
}

impl NeedsName for IntegerCompareOperation {
	fn needs_name(&self) -> &'static str {
		let Self {
			r#type, operator, ..
		} = *self;

		match (r#type, operator) {
			(IntegerType::I32, IntegerCompareOperator::Equal) => "equal_i32",
			(IntegerType::I32, IntegerCompareOperator::NotEqual) => "not_equal_i32",
			(IntegerType::I32, IntegerCompareOperator::LessThan { signed: true }) => {
				"less_than_s32"
			}
			(IntegerType::I32, IntegerCompareOperator::LessThan { signed: false }) => {
				"less_than_u32"
			}
			(IntegerType::I32, IntegerCompareOperator::GreaterThan { signed: true }) => {
				"greater_than_s32"
			}
			(IntegerType::I32, IntegerCompareOperator::GreaterThan { signed: false }) => {
				"greater_than_u32"
			}
			(IntegerType::I32, IntegerCompareOperator::LessThanEqual { signed: true }) => {
				"less_than_equal_s32"
			}
			(IntegerType::I32, IntegerCompareOperator::LessThanEqual { signed: false }) => {
				"less_than_equal_u32"
			}
			(IntegerType::I32, IntegerCompareOperator::GreaterThanEqual { signed: true }) => {
				"greater_than_equal_s32"
			}
			(IntegerType::I32, IntegerCompareOperator::GreaterThanEqual { signed: false }) => {
				"greater_than_equal_u32"
			}
			(IntegerType::I64, IntegerCompareOperator::Equal) => "equal_i64",
			(IntegerType::I64, IntegerCompareOperator::NotEqual) => "not_equal_i64",
			(IntegerType::I64, IntegerCompareOperator::LessThan { signed: true }) => {
				"less_than_s64"
			}
			(IntegerType::I64, IntegerCompareOperator::LessThan { signed: false }) => {
				"less_than_u64"
			}
			(IntegerType::I64, IntegerCompareOperator::GreaterThan { signed: true }) => {
				"greater_than_s64"
			}
			(IntegerType::I64, IntegerCompareOperator::GreaterThan { signed: false }) => {
				"greater_than_u64"
			}
			(IntegerType::I64, IntegerCompareOperator::LessThanEqual { signed: true }) => {
				"less_than_equal_s64"
			}
			(IntegerType::I64, IntegerCompareOperator::LessThanEqual { signed: false }) => {
				"less_than_equal_u64"
			}
			(IntegerType::I64, IntegerCompareOperator::GreaterThanEqual { signed: true }) => {
				"greater_than_equal_s64"
			}
			(IntegerType::I64, IntegerCompareOperator::GreaterThanEqual { signed: false }) => {
				"greater_than_equal_u64"
			}
		}
	}
}

impl NeedsName for IntegerNarrow {
	fn needs_name(&self) -> &'static str {
		"narrow_i64"
	}
}

impl NeedsName for IntegerWiden {
	fn needs_name(&self) -> &'static str {
		"widen_i32"
	}
}

impl NeedsName for IntegerExtend {
	fn needs_name(&self) -> &'static str {
		let Self { r#type, .. } = *self;

		match r#type {
			ExtendType::I32_S8 => "extend_s8_to_i32",
			ExtendType::I32_S16 => "extend_s16_to_i32",
			ExtendType::I64_S8 => "extend_s8_to_i64",
			ExtendType::I64_S16 => "extend_s16_to_i64",
			ExtendType::I64_S32 => "extend_s32_to_i64",
		}
	}
}

impl NeedsName for IntegerConvertToNumber {
	fn needs_name(&self) -> &'static str {
		let Self {
			signed, to, from, ..
		} = *self;

		match (from, to, signed) {
			(IntegerType::I32, NumberType::F32, true) => "convert_s32_to_f32",
			(IntegerType::I32, NumberType::F32, false) => "convert_u32_to_f32",
			(IntegerType::I32, NumberType::F64, true) => "convert_s32_to_f64",
			(IntegerType::I32, NumberType::F64, false) => "convert_u32_to_f64",
			(IntegerType::I64, NumberType::F32, true) => "convert_s64_to_f32",
			(IntegerType::I64, NumberType::F32, false) => "convert_u64_to_f32",
			(IntegerType::I64, NumberType::F64, true) => "convert_s64_to_f64",
			(IntegerType::I64, NumberType::F64, false) => "convert_u64_to_f64",
		}
	}
}

impl NeedsName for IntegerTransmuteToNumber {
	fn needs_name(&self) -> &'static str {
		let Self { from, .. } = *self;

		match from {
			IntegerType::I32 => "transmute_i32_to_f32",
			IntegerType::I64 => "transmute_i64_to_f64",
		}
	}
}

impl NeedsName for NumberUnaryOperation {
	fn needs_name(&self) -> &'static str {
		let Self {
			r#type, operator, ..
		} = *self;

		match (r#type, operator) {
			(NumberType::F32, NumberUnaryOperator::Absolute) => "absolute_f32",
			(NumberType::F32, NumberUnaryOperator::Negate) => "negate_f32",
			(NumberType::F32, NumberUnaryOperator::SquareRoot) => "square_root_f32",
			(NumberType::F32, NumberUnaryOperator::RoundUp) => "round_up_f32",
			(NumberType::F32, NumberUnaryOperator::RoundDown) => "round_down_f32",
			(NumberType::F32, NumberUnaryOperator::Truncate) => "truncate_f32",
			(NumberType::F32, NumberUnaryOperator::Nearest) => "nearest_f32",
			(NumberType::F64, NumberUnaryOperator::Absolute) => "absolute_f64",
			(NumberType::F64, NumberUnaryOperator::Negate) => "negate_f64",
			(NumberType::F64, NumberUnaryOperator::SquareRoot) => "square_root_f64",
			(NumberType::F64, NumberUnaryOperator::RoundUp) => "round_up_f64",
			(NumberType::F64, NumberUnaryOperator::RoundDown) => "round_down_f64",
			(NumberType::F64, NumberUnaryOperator::Truncate) => "truncate_f64",
			(NumberType::F64, NumberUnaryOperator::Nearest) => "nearest_f64",
		}
	}
}

impl NeedsName for NumberBinaryOperation {
	fn needs_name(&self) -> &'static str {
		let Self {
			r#type, operator, ..
		} = *self;

		match (r#type, operator) {
			(NumberType::F32, NumberBinaryOperator::Add) => "add_f32",
			(NumberType::F32, NumberBinaryOperator::Subtract) => "subtract_f32",
			(NumberType::F32, NumberBinaryOperator::Multiply) => "multiply_f32",
			(NumberType::F32, NumberBinaryOperator::Divide) => "divide_f32",
			(NumberType::F32, NumberBinaryOperator::Minimum) => "minimum_f32",
			(NumberType::F32, NumberBinaryOperator::Maximum) => "maximum_f32",
			(NumberType::F32, NumberBinaryOperator::CopySign) => "copy_sign_f32",
			(NumberType::F64, NumberBinaryOperator::Add) => "add_f64",
			(NumberType::F64, NumberBinaryOperator::Subtract) => "subtract_f64",
			(NumberType::F64, NumberBinaryOperator::Multiply) => "multiply_f64",
			(NumberType::F64, NumberBinaryOperator::Divide) => "divide_f64",
			(NumberType::F64, NumberBinaryOperator::Minimum) => "minimum_f64",
			(NumberType::F64, NumberBinaryOperator::Maximum) => "maximum_f64",
			(NumberType::F64, NumberBinaryOperator::CopySign) => "copy_sign_f64",
		}
	}
}

impl NeedsName for NumberCompareOperation {
	fn needs_name(&self) -> &'static str {
		let Self {
			r#type, operator, ..
		} = *self;

		match (r#type, operator) {
			(NumberType::F32, NumberCompareOperator::Equal) => "equal_f32",
			(NumberType::F32, NumberCompareOperator::NotEqual) => "not_equal_f32",
			(NumberType::F32, NumberCompareOperator::LessThan) => "less_than_f32",
			(NumberType::F32, NumberCompareOperator::GreaterThan) => "greater_than_f32",
			(NumberType::F32, NumberCompareOperator::LessThanEqual) => "less_than_equal_f32",
			(NumberType::F32, NumberCompareOperator::GreaterThanEqual) => "greater_than_equal_f32",
			(NumberType::F64, NumberCompareOperator::Equal) => "equal_f64",
			(NumberType::F64, NumberCompareOperator::NotEqual) => "not_equal_f64",
			(NumberType::F64, NumberCompareOperator::LessThan) => "less_than_f64",
			(NumberType::F64, NumberCompareOperator::GreaterThan) => "greater_than_f64",
			(NumberType::F64, NumberCompareOperator::LessThanEqual) => "less_than_equal_f64",
			(NumberType::F64, NumberCompareOperator::GreaterThanEqual) => "greater_than_equal_f64",
		}
	}
}

impl NeedsName for NumberNarrow {
	fn needs_name(&self) -> &'static str {
		"narrow_f64"
	}
}

impl NeedsName for NumberWiden {
	fn needs_name(&self) -> &'static str {
		"widen_f32"
	}
}

impl NeedsName for NumberTruncateToInteger {
	fn needs_name(&self) -> &'static str {
		let Self {
			signed,
			saturate,
			to,
			from,
			..
		} = *self;

		match (from, to, signed, saturate) {
			(NumberType::F32, IntegerType::I32, true, true) => "saturate_f32_to_s32",
			(NumberType::F32, IntegerType::I32, true, false) => "truncate_f32_to_s32",
			(NumberType::F32, IntegerType::I32, false, true) => "saturate_f32_to_u32",
			(NumberType::F32, IntegerType::I32, false, false) => "truncate_f32_to_u32",
			(NumberType::F32, IntegerType::I64, true, true) => "saturate_f32_to_s64",
			(NumberType::F32, IntegerType::I64, true, false) => "truncate_f32_to_s64",
			(NumberType::F32, IntegerType::I64, false, true) => "saturate_f32_to_u64",
			(NumberType::F32, IntegerType::I64, false, false) => "truncate_f32_to_u64",
			(NumberType::F64, IntegerType::I32, true, true) => "saturate_f64_to_s32",
			(NumberType::F64, IntegerType::I32, true, false) => "truncate_f64_to_s32",
			(NumberType::F64, IntegerType::I32, false, true) => "saturate_f64_to_u32",
			(NumberType::F64, IntegerType::I32, false, false) => "truncate_f64_to_u32",
			(NumberType::F64, IntegerType::I64, true, true) => "saturate_f64_to_s64",
			(NumberType::F64, IntegerType::I64, true, false) => "truncate_f64_to_s64",
			(NumberType::F64, IntegerType::I64, false, true) => "saturate_f64_to_u64",
			(NumberType::F64, IntegerType::I64, false, false) => "truncate_f64_to_u64",
		}
	}
}

impl NeedsName for NumberTransmuteToInteger {
	fn needs_name(&self) -> &'static str {
		let Self { from, .. } = *self;

		match from {
			NumberType::F32 => "transmute_f32_to_i32",
			NumberType::F64 => "transmute_f64_to_i64",
		}
	}
}

impl NeedsName for GlobalNew {
	fn needs_name(&self) -> &'static str {
		""
	}
}

impl NeedsName for GlobalGet {
	fn needs_name(&self) -> &'static str {
		""
	}
}

impl NeedsName for TableNew {
	fn needs_name(&self) -> &'static str {
		"table_new"
	}
}

impl NeedsName for TableGet {
	fn needs_name(&self) -> &'static str {
		"table_get"
	}
}

impl NeedsName for TableSize {
	fn needs_name(&self) -> &'static str {
		"table_size"
	}
}

impl NeedsName for TableGrow {
	fn needs_name(&self) -> &'static str {
		"table_grow"
	}
}

impl NeedsName for ElementsNew {
	fn needs_name(&self) -> &'static str {
		""
	}
}

impl NeedsName for MemoryNew {
	fn needs_name(&self) -> &'static str {
		"memory_new"
	}
}

impl NeedsName for MemoryLoad {
	fn needs_name(&self) -> &'static str {
		let Self { r#type, .. } = *self;

		match r#type {
			LoadType::I32_S8 => "load_i32_from_s8",
			LoadType::I32_U8 => "load_i32_from_u8",
			LoadType::I32_S16 => "load_i32_from_s16",
			LoadType::I32_U16 => "load_i32_from_u16",
			LoadType::I32 => "load_i32",
			LoadType::I64_S8 => "load_i64_from_s8",
			LoadType::I64_U8 => "load_i64_from_u8",
			LoadType::I64_S16 => "load_i64_from_s16",
			LoadType::I64_U16 => "load_i64_from_u16",
			LoadType::I64_S32 => "load_i64_from_s32",
			LoadType::I64_U32 => "load_i64_from_u32",
			LoadType::I64 => "load_i64",
			LoadType::F32 => "load_f32",
			LoadType::F64 => "load_f64",
		}
	}
}

impl NeedsName for MemorySize {
	fn needs_name(&self) -> &'static str {
		"memory_size"
	}
}

impl NeedsName for MemoryGrow {
	fn needs_name(&self) -> &'static str {
		"memory_grow"
	}
}

impl NeedsName for DataNew {
	fn needs_name(&self) -> &'static str {
		""
	}
}

impl NeedsName for Expression {
	fn needs_name(&self) -> &'static str {
		match self {
			Expression::Function(_)
			| Expression::Scoped(_)
			| Expression::Match(_)
			| Expression::Import(_)
			| Expression::Trap
			| Expression::Null
			| Expression::Local(_)
			| Expression::Call(_)
			| Expression::RefIsNull(_) => "",

			Expression::I32(i32) => i32.needs_name(),
			Expression::I64(i64) => i64.needs_name(),
			Expression::F32(f32) => f32.needs_name(),
			Expression::F64(f64) => f64.needs_name(),

			Expression::IntegerUnaryOperation(integer_unary_operation) => {
				integer_unary_operation.needs_name()
			}
			Expression::IntegerBinaryOperation(integer_binary_operation) => {
				integer_binary_operation.needs_name()
			}
			Expression::IntegerCompareOperation(integer_compare_operation) => {
				integer_compare_operation.needs_name()
			}
			Expression::IntegerNarrow(integer_narrow) => integer_narrow.needs_name(),
			Expression::IntegerWiden(integer_widen) => integer_widen.needs_name(),
			Expression::IntegerExtend(integer_extend) => integer_extend.needs_name(),
			Expression::IntegerConvertToNumber(integer_convert_to_number) => {
				integer_convert_to_number.needs_name()
			}
			Expression::IntegerTransmuteToNumber(integer_transmute_to_number) => {
				integer_transmute_to_number.needs_name()
			}
			Expression::NumberUnaryOperation(number_unary_operation) => {
				number_unary_operation.needs_name()
			}
			Expression::NumberBinaryOperation(number_binary_operation) => {
				number_binary_operation.needs_name()
			}
			Expression::NumberCompareOperation(number_compare_operation) => {
				number_compare_operation.needs_name()
			}
			Expression::NumberNarrow(number_narrow) => number_narrow.needs_name(),
			Expression::NumberWiden(number_widen) => number_widen.needs_name(),
			Expression::NumberTruncateToInteger(number_truncate_to_integer) => {
				number_truncate_to_integer.needs_name()
			}
			Expression::NumberTransmuteToInteger(number_transmute_to_integer) => {
				number_transmute_to_integer.needs_name()
			}
			Expression::GlobalNew(global_new) => global_new.needs_name(),
			Expression::GlobalGet(global_get) => global_get.needs_name(),
			Expression::TableNew(table_new) => table_new.needs_name(),
			Expression::TableGet(table_get) => table_get.needs_name(),
			Expression::TableSize(table_size) => table_size.needs_name(),
			Expression::TableGrow(table_grow) => table_grow.needs_name(),
			Expression::ElementsNew(elements_new) => elements_new.needs_name(),
			Expression::MemoryNew(memory_new) => memory_new.needs_name(),
			Expression::MemoryLoad(memory_load) => memory_load.needs_name(),
			Expression::MemorySize(memory_size) => memory_size.needs_name(),
			Expression::MemoryGrow(memory_grow) => memory_grow.needs_name(),
			Expression::DataNew(data_new) => data_new.needs_name(),
		}
	}
}

impl NeedsName for GlobalSet {
	fn needs_name(&self) -> &'static str {
		""
	}
}

impl NeedsName for TableSet {
	fn needs_name(&self) -> &'static str {
		"table_set"
	}
}

impl NeedsName for TableFill {
	fn needs_name(&self) -> &'static str {
		"table_fill"
	}
}

impl NeedsName for TableCopy {
	fn needs_name(&self) -> &'static str {
		"table_copy"
	}
}

impl NeedsName for TableInit {
	fn needs_name(&self) -> &'static str {
		"table_init"
	}
}

impl NeedsName for ElementsDrop {
	fn needs_name(&self) -> &'static str {
		"elements_drop"
	}
}

impl NeedsName for MemoryStore {
	fn needs_name(&self) -> &'static str {
		let Self { r#type, .. } = *self;

		match r#type {
			StoreType::I32_I8 => "store_i32_into_i8",
			StoreType::I32_I16 => "store_i32_into_i16",
			StoreType::I32 => "store_i32",
			StoreType::I64_I8 => "store_i64_into_i8",
			StoreType::I64_I16 => "store_i64_into_i16",
			StoreType::I64_I32 => "store_i64_into_i32",
			StoreType::I64 => "store_i64",
			StoreType::F32 => "store_f32",
			StoreType::F64 => "store_f64",
		}
	}
}

impl NeedsName for MemoryFill {
	fn needs_name(&self) -> &'static str {
		"memory_fill"
	}
}

impl NeedsName for MemoryCopy {
	fn needs_name(&self) -> &'static str {
		"memory_copy"
	}
}

impl NeedsName for MemoryInit {
	fn needs_name(&self) -> &'static str {
		"memory_init"
	}
}

impl NeedsName for DataDrop {
	fn needs_name(&self) -> &'static str {
		"data_drop"
	}
}

impl NeedsName for Statement {
	fn needs_name(&self) -> &'static str {
		match self {
			Statement::Match(_)
			| Statement::Repeat(_)
			| Statement::FastDefine(_)
			| Statement::SlowDefine(_)
			| Statement::Assign(_)
			| Statement::AssignAll(_)
			| Statement::Call(_) => "",

			Statement::GlobalSet(global_set) => global_set.needs_name(),
			Statement::TableSet(table_set) => table_set.needs_name(),
			Statement::TableFill(table_fill) => table_fill.needs_name(),
			Statement::TableCopy(table_copy) => table_copy.needs_name(),
			Statement::TableInit(table_init) => table_init.needs_name(),
			Statement::ElementsDrop(elements_drop) => elements_drop.needs_name(),
			Statement::MemoryStore(memory_store) => memory_store.needs_name(),
			Statement::MemoryFill(memory_fill) => memory_fill.needs_name(),
			Statement::MemoryCopy(memory_copy) => memory_copy.needs_name(),
			Statement::MemoryInit(memory_init) => memory_init.needs_name(),
			Statement::DataDrop(data_drop) => data_drop.needs_name(),
		}
	}
}

pub struct NamesFinder<'names> {
	names: &'names mut Vec<&'static str>,
}

impl<'names> NamesFinder<'names> {
	pub const fn new(names: &'names mut Vec<&'static str>) -> Self {
		Self { names }
	}

	pub fn run(&mut self, tree: &LuauTree) {
		tree.accept(self)
			.continue_value()
			.expect("names finder must not fail");
	}
}

impl Visitor for NamesFinder<'_> {
	type Output = ();

	fn visit_expression(&mut self, expression: &Expression) -> ControlFlow<Self::Output> {
		let name = expression.needs_name();

		if !name.is_empty() {
			self.names.push(name);
		}

		ControlFlow::Continue(())
	}

	fn visit_statement(&mut self, statement: &Statement) -> ControlFlow<Self::Output> {
		let name = statement.needs_name();

		if !name.is_empty() {
			self.names.push(name);
		}

		ControlFlow::Continue(())
	}
}
