use alloc::{sync::Arc, vec::Vec};
use list::resizable::Resizable;

use crate::Link;

pub trait Host {
	fn identifier(&self) -> &'static str;

	fn for_each_id(&self, handler: &mut dyn FnMut(u32)) {
		let _ = handler;
	}

	fn for_each_mut_id(&mut self, handler: &mut dyn FnMut(&mut u32)) {
		let _ = handler;
	}

	fn for_each_argument(&self, handler: &mut dyn FnMut(Link)) {
		let _ = handler;
	}

	fn for_each_mut_argument(&mut self, handler: &mut dyn FnMut(&mut Link)) {
		let _ = handler;
	}
}

pub struct Identity {
	pub sources: Resizable<Link, 4>,
}

pub struct Fence {
	pub sources: Resizable<Link, 4>,
}

pub struct Apply {
	pub function: Link,
	pub arguments: Vec<Link>,
	pub results: u16,
	pub states: u16,
}

#[derive(Clone, Copy)]
pub struct RefIsNull {
	pub source: Link,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum IntegerType {
	I32,
	I64,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum IntegerUnaryOperator {
	CountOnes,
	LeadingZeroes,
	TrailingZeroes,
}

#[derive(Clone, Copy)]
pub struct IntegerUnaryOperation {
	pub source: Link,
	pub r#type: IntegerType,
	pub operator: IntegerUnaryOperator,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum IntegerBinaryOperator {
	Add,
	Subtract,
	Multiply,
	Divide { signed: bool },
	Remainder { signed: bool },
	And,
	Or,
	ExclusiveOr,
	ShiftLeft,
	ShiftRight { signed: bool },
	RotateLeft,
	RotateRight,
}

#[derive(Clone, Copy)]
pub struct IntegerBinaryOperation {
	pub lhs: Link,
	pub rhs: Link,
	pub r#type: IntegerType,
	pub operator: IntegerBinaryOperator,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum IntegerCompareOperator {
	Equal,
	NotEqual,
	LessThan { signed: bool },
	GreaterThan { signed: bool },
	LessThanEqual { signed: bool },
	GreaterThanEqual { signed: bool },
}

#[derive(Clone, Copy)]
pub struct IntegerCompareOperation {
	pub lhs: Link,
	pub rhs: Link,
	pub r#type: IntegerType,
	pub operator: IntegerCompareOperator,
}

#[derive(Clone, Copy)]
pub struct IntegerNarrow {
	pub source: Link,
}

#[derive(Clone, Copy)]
pub struct IntegerWiden {
	pub source: Link,
}

#[expect(non_camel_case_types)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ExtendType {
	I32_S8,
	I32_S16,

	I64_S8,
	I64_S16,
	I64_S32,
}

#[derive(Clone, Copy)]
pub struct IntegerExtend {
	pub source: Link,
	pub r#type: ExtendType,
}

#[derive(Clone, Copy)]
pub struct IntegerConvertToNumber {
	pub source: Link,
	pub signed: bool,
	pub to: NumberType,
	pub from: IntegerType,
}

#[derive(Clone, Copy)]
pub struct IntegerTransmuteToNumber {
	pub source: Link,
	pub from: IntegerType,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum NumberType {
	F32,
	F64,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum NumberUnaryOperator {
	Absolute,
	Negate,
	SquareRoot,
	RoundUp,
	RoundDown,
	Truncate,
	Nearest,
}

#[derive(Clone, Copy)]
pub struct NumberUnaryOperation {
	pub source: Link,
	pub r#type: NumberType,
	pub operator: NumberUnaryOperator,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum NumberBinaryOperator {
	Add,
	Subtract,
	Multiply,
	Divide,
	Minimum,
	Maximum,
	CopySign,
}

#[derive(Clone, Copy)]
pub struct NumberBinaryOperation {
	pub lhs: Link,
	pub rhs: Link,
	pub r#type: NumberType,
	pub operator: NumberBinaryOperator,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum NumberCompareOperator {
	Equal,
	NotEqual,
	LessThan,
	GreaterThan,
	LessThanEqual,
	GreaterThanEqual,
}

#[derive(Clone, Copy)]
pub struct NumberCompareOperation {
	pub lhs: Link,
	pub rhs: Link,
	pub r#type: NumberType,
	pub operator: NumberCompareOperator,
}

#[derive(Clone, Copy)]
pub struct NumberTruncateToInteger {
	pub source: Link,
	pub signed: bool,
	pub saturate: bool,
	pub to: IntegerType,
	pub from: NumberType,
}

#[derive(Clone, Copy)]
pub struct NumberTransmuteToInteger {
	pub source: Link,
	pub from: NumberType,
}

#[derive(Clone, Copy)]
pub struct NumberNarrow {
	pub source: Link,
}

#[derive(Clone, Copy)]
pub struct NumberWiden {
	pub source: Link,
}

#[derive(Clone, Copy)]
pub struct Location {
	pub reference: Link,
	pub offset: Link,
}

#[derive(Clone, Copy)]
pub struct GlobalNew {
	pub initializer: Link,
}

#[derive(Clone, Copy)]
pub struct GlobalGet {
	pub source: Link,
}

#[derive(Clone, Copy)]
pub struct GlobalSet {
	pub destination: Link,
	pub source: Link,
}

pub struct TableNew {
	pub initializer: Vec<(Link, u32)>,
	pub minimum: u32,
	pub maximum: u32,
}

#[derive(Clone, Copy)]
pub struct TableGet {
	pub source: Location,
}

#[derive(Clone, Copy)]
pub struct TableSet {
	pub destination: Location,
	pub source: Link,
}

#[derive(Clone, Copy)]
pub struct TableSize {
	pub source: Link,
}

#[derive(Clone, Copy)]
pub struct TableGrow {
	pub destination: Link,
	pub initializer: Link,
	pub size: Link,
}

#[derive(Clone, Copy)]
pub struct TableFill {
	pub destination: Location,
	pub source: Link,
	pub size: Link,
}

#[derive(Clone, Copy)]
pub struct TableCopy {
	pub destination: Location,
	pub source: Location,
	pub size: Link,
}

#[derive(Clone, Copy)]
pub struct TableDrop {
	pub source: Link,
}

#[derive(Clone)]
pub struct MemoryNew {
	pub initializer: Vec<(Arc<[u8]>, u32)>,
	pub minimum: u32,
	pub maximum: u32,
}

#[expect(non_camel_case_types)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum LoadType {
	I32_S8,
	I32_U8,
	I32_S16,
	I32_U16,
	I32,

	I64_S8,
	I64_U8,
	I64_S16,
	I64_U16,
	I64_S32,
	I64_U32,
	I64,

	F32,
	F64,
}

#[derive(Clone, Copy)]
pub struct MemoryLoad {
	pub source: Location,
	pub r#type: LoadType,
}

#[expect(non_camel_case_types)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum StoreType {
	I32_I8,
	I32_I16,
	I32,

	I64_I8,
	I64_I16,
	I64_I32,
	I64,

	F32,
	F64,
}

#[derive(Clone, Copy)]
pub struct MemoryStore {
	pub destination: Location,
	pub source: Link,
	pub r#type: StoreType,
}

#[derive(Clone, Copy)]
pub struct MemorySize {
	pub source: Link,
}

#[derive(Clone, Copy)]
pub struct MemoryGrow {
	pub destination: Link,
	pub size: Link,
}

#[derive(Clone, Copy)]
pub struct MemoryFill {
	pub destination: Location,
	pub byte: Link,
	pub size: Link,
}

#[derive(Clone, Copy)]
pub struct MemoryCopy {
	pub destination: Location,
	pub source: Location,
	pub size: Link,
}

#[derive(Clone, Copy)]
pub struct MemoryDrop {
	pub source: Link,
}
