//! Region types and their boundary nodes.
//!
//! Each region kind lives in its own module:
//! [`function`] / [`matcher`] + [`branch`] / [`repeat`].

use parking_lot::{ArcMutexGuard, RawMutex};

use crate::Node;

pub mod branch;
pub mod function;
pub mod matcher;
pub mod repeat;

pub use self::{branch::Branch, function::Function, matcher::Match, repeat::Repeat};

/// A locked reference to a concrete region type.
pub enum Region {
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
			Self::Function(region) => &region.nodes,
			Self::Branch(region) => &region.nodes,
			Self::Repeat(region) => &region.nodes,
		}
	}

	/// Returns a mutable reference to the region's nodes.
	pub fn nodes_mut(&mut self) -> &mut Vec<Node> {
		match self {
			Self::Function(region) => &mut region.nodes,
			Self::Branch(region) => &mut region.nodes,
			Self::Repeat(region) => &mut region.nodes,
		}
	}

	/// Returns the ids of the arguments and results boundary nodes.
	#[must_use]
	pub fn roots(&self) -> (u32, u32) {
		match self {
			Self::Function(region) => region.roots(),
			Self::Branch(region) => region.roots(),
			Self::Repeat(region) => region.roots(),
		}
	}
}
