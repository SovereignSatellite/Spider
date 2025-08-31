use alloc::{boxed::Box, sync::Arc, vec::Vec};
use list::resizable::Resizable;

use super::link::Link;

#[derive(Clone, Copy)]
pub enum ValueType {
	I32,
	I64,
	F32,
	F64,

	Reference,
}

#[derive(Clone)]
pub struct FunctionType {
	pub arguments: Resizable<ValueType, 15>,
	pub results: Resizable<ValueType, 15>,
}

#[derive(Clone)]
pub struct LambdaIn {
	pub output: u32,
	pub r#type: Box<FunctionType>,
	pub dependencies: Vec<Link>,
}

impl LambdaIn {
	#[must_use]
	pub fn dependency_ports(&self) -> core::ops::Range<u16> {
		0..self.dependencies.len().try_into().unwrap()
	}

	#[must_use]
	pub fn argument_ports(&self) -> core::ops::Range<u16> {
		let dependencies: u16 = self.dependencies.len().try_into().unwrap();
		let arguments: u16 = self.r#type.arguments.len().try_into().unwrap();

		dependencies..dependencies + arguments
	}

	#[must_use]
	pub fn output_ports(&self) -> core::ops::Range<u16> {
		let dependencies: u16 = self.dependencies.len().try_into().unwrap();
		let arguments: u16 = self.r#type.arguments.len().try_into().unwrap();

		0..dependencies + arguments
	}
}

#[derive(Clone)]
pub struct LambdaOut {
	pub input: u32,
	pub results: Vec<Link>,
}

#[derive(Clone)]
pub struct RegionIn {
	pub input: u32,
	pub output: u32,
}

#[derive(Clone)]
pub struct RegionOut {
	pub input: u32,
	pub output: u32,
	pub results: Vec<Link>,
}

#[derive(Clone)]
pub struct GammaIn {
	pub output: u32,
	pub condition: Link,
	pub arguments: Vec<Link>,
}

#[derive(Clone)]
pub struct GammaOut {
	pub input: u32,
	pub regions: Vec<u32>,
}

#[derive(Clone)]
pub struct ThetaIn {
	pub output: u32,
	pub arguments: Vec<Link>,
}

#[derive(Clone)]
pub struct ThetaOut {
	pub input: u32,
	pub condition: Link,
	pub results: Vec<Link>,
}

#[derive(Clone)]
pub struct Import {
	pub environment: Link,
	pub namespace: Arc<str>,
	pub identifier: Arc<str>,
}

#[derive(Clone)]
pub struct OmegaIn {
	pub output: u32,
}

#[derive(Clone)]
pub struct Export {
	pub identifier: Arc<str>,
	pub reference: Link,
}

#[derive(Clone)]
pub struct OmegaOut {
	pub input: u32,

	pub state: Link,
	pub exports: Vec<Export>,
}
