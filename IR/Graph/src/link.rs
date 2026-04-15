/// A region-local link from one node's output port to another node's input.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Link(
	/// The node identifier.
	pub u32,
	/// The port index.
	pub u16,
);

impl Link {
	/// A sentinel value representing a dangling link.
	pub const DANGLING: Self = Self(u32::MAX, u16::MAX);
}
