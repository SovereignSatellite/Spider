/// A link from one node's output port to another node's input.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Link(
	/// The node identifier.
	pub u32,
	/// The port index.
	pub u16,
);

impl Link {
	/// Packs this link into a `usize` for use as an index.
	#[must_use]
	pub const fn into_usize(self) -> usize {
		let [id_0, id_1, id_2, id_3] = self.0.to_le_bytes();
		let [port_0, port_1] = self.1.to_le_bytes();

		usize::from_le_bytes([id_0, id_1, port_0, id_2, port_1, id_3, 0, 0])
	}

	/// A sentinel value representing a dangling link.
	pub const DANGLING: Self = Self(u32::MAX, u16::MAX);
}
