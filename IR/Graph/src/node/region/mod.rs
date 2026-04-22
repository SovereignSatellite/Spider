//! Region types and their boundary nodes.
//!
//! Each region kind lives in its own module:
//! [`module`] / [`function`] / [`matcher`] + [`branch`] / [`repeat`].

use parking_lot::{ArcMutexGuard, RawMutex};

use crate::Node;

pub mod branch;
pub mod function;
pub mod matcher;
pub mod module;
pub mod repeat;

pub use self::{
	branch::Branch,
	function::{Function, ValueType},
	matcher::Match,
	module::Module,
	repeat::Repeat,
};

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

impl Region {
	/// Returns a reference to the region's nodes.
	#[must_use]
	pub fn nodes(&self) -> &[Node] {
		match self {
			Self::Module(region) => &region.nodes,
			Self::Function(region) => &region.nodes,
			Self::Branch(region) => &region.nodes,
			Self::Repeat(region) => &region.nodes,
		}
	}

	/// Returns a mutable reference to the region's nodes.
	pub fn nodes_mut(&mut self) -> &mut Vec<Node> {
		match self {
			Self::Module(region) => &mut region.nodes,
			Self::Function(region) => &mut region.nodes,
			Self::Branch(region) => &mut region.nodes,
			Self::Repeat(region) => &mut region.nodes,
		}
	}

	/// Visits each root node index in the region.
	pub fn for_each_root<H: FnMut(u32)>(&self, handler: H) {
		match self {
			Self::Module(region) => region.for_each_root(handler),
			Self::Function(region) => region.for_each_root(handler),
			Self::Branch(region) => region.for_each_root(handler),
			Self::Repeat(region) => region.for_each_root(handler),
		}
	}
}
