use alloc::{sync::Arc, vec::Vec};

pub use control_flow_graph::instruction::{
	ExtendType, IntegerBinaryOperator, IntegerCompareOperator, IntegerType, IntegerUnaryOperator,
	LoadType, NumberBinaryOperator, NumberCompareOperator, NumberType, NumberUnaryOperator,
	StoreType,
};
use list::resizable::Resizable;

use crate::Link;

pub trait Host {
	fn identifier(&self) -> &'static str;

	fn for_each_id(&self, handler: &mut dyn FnMut(u32)) {
		let _handler = handler;
	}

	fn for_each_mut_id(&mut self, handler: &mut dyn FnMut(&mut u32)) {
		let _handler = handler;
	}

	fn for_each_argument(&self, handler: &mut dyn FnMut(Link)) {
		let _handler = handler;
	}

	fn for_each_mut_argument(&mut self, handler: &mut dyn FnMut(&mut Link)) {
		let _handler = handler;
	}
}

pub struct Identity {
	pub sources: Resizable<Link, 4>,
}

pub struct Merge {
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

#[derive(Clone, Copy)]
pub struct IntegerUnaryOperation {
	pub source: Link,
	pub r#type: IntegerType,
	pub operator: IntegerUnaryOperator,
}

#[derive(Clone, Copy)]
pub struct IntegerBinaryOperation {
	pub lhs: Link,
	pub rhs: Link,
	pub r#type: IntegerType,
	pub operator: IntegerBinaryOperator,
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

#[derive(Clone, Copy)]
pub struct NumberUnaryOperation {
	pub source: Link,
	pub r#type: NumberType,
	pub operator: NumberUnaryOperator,
}

#[derive(Clone, Copy)]
pub struct NumberBinaryOperation {
	pub lhs: Link,
	pub rhs: Link,
	pub r#type: NumberType,
	pub operator: NumberBinaryOperator,
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

#[derive(Clone, Copy)]
pub struct MemoryLoad {
	pub source: Location,
	pub r#type: LoadType,
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
