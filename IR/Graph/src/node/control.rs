//! Control flow node types.

use alloc::{boxed::Box, sync::Arc, vec::Vec};
use list::resizable::Resizable;

use crate::Link;

/// Value types for function signatures.
#[derive(Clone, Copy)]
pub enum ValueType {
	/// A 32-bit integer.
	I32,
	/// A 64-bit integer.
	I64,
	/// A 32-bit float.
	F32,
	/// A 64-bit float.
	F64,

	/// A reference.
	Reference,
}

/// A function type with argument and result types.
#[derive(Clone)]
pub struct FunctionType {
	/// The argument types.
	pub arguments: Resizable<ValueType, 15>,
	/// The result types.
	pub results: Resizable<ValueType, 15>,
}

/// A lambda (function) input node.
#[derive(Clone)]
pub struct LambdaIn {
	/// The paired output node.
	pub output: u32,
	/// The function type signature.
	pub kind: Box<FunctionType>,
	/// The closure dependencies.
	pub dependencies: Vec<Link>,
}

/// A lambda (function) output node.
#[derive(Clone)]
pub struct LambdaOut {
	/// The paired input node.
	pub input: u32,
	/// The result links.
	pub results: Vec<Link>,
}

/// A region (scope) input node.
#[derive(Clone, Copy)]
pub struct RegionIn {
	/// The parent gamma input node.
	pub input: u32,
	/// The paired output node.
	pub output: u32,
}

/// A region (scope) output node.
#[derive(Clone)]
pub struct RegionOut {
	/// The paired input node.
	pub input: u32,
	/// The parent gamma output node.
	pub output: u32,
	/// The result links.
	pub results: Vec<Link>,
}

/// A gamma (conditional) input node.
#[derive(Clone)]
pub struct GammaIn {
	/// The paired output node.
	pub output: u32,
	/// The argument links.
	pub arguments: Vec<Link>,
	/// The condition link.
	pub condition: Link,
}

/// A gamma (conditional) output node.
#[derive(Clone)]
pub struct GammaOut {
	/// The paired input node.
	pub input: u32,
	/// The region output nodes for each branch.
	pub regions: Vec<u32>,
}

/// A theta (loop) input node.
#[derive(Clone)]
pub struct ThetaIn {
	/// The paired output node.
	pub output: u32,
	/// The argument links.
	pub arguments: Vec<Link>,
}

/// A theta (loop) output node.
#[derive(Clone)]
pub struct ThetaOut {
	/// The paired input node.
	pub input: u32,
	/// The result links fed back to the loop or out.
	pub results: Vec<Link>,
	/// The loop continuation condition.
	pub condition: Link,
}

/// An external import node.
#[derive(Clone)]
pub struct Import {
	/// The environment link.
	pub environment: Link,
	/// The import namespace.
	pub namespace: Arc<str>,
	/// The import name.
	pub identifier: Arc<str>,
}

/// An omega (program) input node.
#[derive(Clone, Copy)]
pub struct OmegaIn {
	/// The paired output node.
	pub output: u32,
}

/// An exported symbol.
#[derive(Clone)]
pub struct Export {
	/// The export name.
	pub identifier: Arc<str>,
	/// The exported value link.
	pub reference: Link,
}

/// An omega (program) output node.
#[derive(Clone)]
pub struct OmegaOut {
	/// The paired input node.
	pub input: u32,

	/// The final state link.
	pub state: Link,
	/// The exported symbols.
	pub exports: Vec<Export>,
}
