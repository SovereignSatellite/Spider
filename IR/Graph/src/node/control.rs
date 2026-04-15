//! Control flow node types.

use alloc::sync::{Arc, Weak};

use list::resizable::Resizable;
use parking_lot::{ArcMutexGuard, Mutex, RawMutex};

use super::Node;
use crate::Link;

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

/// An exported symbol.
#[derive(Clone)]
pub struct Export {
	/// The export name.
	pub identifier: Arc<str>,
	/// The exported value link.
	pub reference: Link,
}

/// A module region.
pub struct Module {
	/// The nodes in this region.
	pub nodes: Vec<Node>,
}

/// The boundary arguments node for a module region.
pub struct ModuleArguments {
	/// The parent module.
	pub parent: Weak<Mutex<Module>>,
}

/// The boundary results node for a module region.
pub struct ModuleResults {
	/// The parent module.
	pub parent: Weak<Mutex<Module>>,
	/// The final state link.
	pub state: Link,
	/// The exported symbols.
	pub exports: Vec<Export>,
}

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

/// A function region.
pub struct Function {
	/// The argument types.
	pub argument_types: Resizable<ValueType, 15>,
	/// The result types.
	pub result_types: Resizable<ValueType, 15>,
	/// The closure captures.
	pub captures: Vec<Link>,
	/// The nodes in this region.
	pub nodes: Vec<Node>,
}

/// The boundary captures node for a function region.
pub struct FunctionCaptures {
	/// The parent function.
	pub parent: Weak<Mutex<Function>>,
}

/// The boundary arguments node for a function region.
pub struct FunctionArguments {
	/// The parent function.
	pub parent: Weak<Mutex<Function>>,
}

/// The boundary results node for a function region.
pub struct FunctionResults {
	/// The parent function.
	pub parent: Weak<Mutex<Function>>,
	/// The result value links.
	pub sources: Vec<Link>,
}

/// A branch region within a match.
pub struct Branch {
	/// The nodes in this region.
	pub nodes: Vec<Node>,
	/// The parent match.
	pub parent: Weak<Mutex<Match>>,
}

/// The boundary arguments node for a branch region.
pub struct BranchArguments {
	/// The parent branch.
	pub parent: Weak<Mutex<Branch>>,
}

/// The boundary results node for a branch region.
pub struct BranchResults {
	/// The parent branch.
	pub parent: Weak<Mutex<Branch>>,
	/// The result value links.
	pub sources: Vec<Link>,
}

/// A match (conditional) region.
pub struct Match {
	/// The argument links.
	pub arguments: Vec<Link>,
	/// The condition link.
	pub condition: Link,
	/// The branch regions.
	pub branches: Vec<Arc<Mutex<Branch>>>,
}

/// A repeat (loop) region.
pub struct Repeat {
	/// The argument links.
	pub arguments: Vec<Link>,
	/// The nodes in this region.
	pub nodes: Vec<Node>,
}

/// The boundary arguments node for a repeat region.
pub struct RepeatArguments {
	/// The parent repeat.
	pub parent: Weak<Mutex<Repeat>>,
}

/// The boundary results node for a repeat region.
pub struct RepeatResults {
	/// The parent repeat.
	pub parent: Weak<Mutex<Repeat>>,
	/// The result value links fed back to the loop or out.
	pub sources: Vec<Link>,
	/// The loop continuation condition.
	pub condition: Link,
}

/// A locked reference to a concrete region type.
pub enum Region {
	/// A module region.
	Module(ArcMutexGuard<RawMutex, Module>),
	/// A function region.
	Function(ArcMutexGuard<RawMutex, Function>),
	/// A branch region within a match.
	Branch(ArcMutexGuard<RawMutex, Branch>),
	/// A repeat (loop) region.
	Repeat(ArcMutexGuard<RawMutex, Repeat>),
}
