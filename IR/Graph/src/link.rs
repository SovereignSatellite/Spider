#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Link(pub u32, pub u16);

impl Link {
	#[must_use]
	pub const fn into_usize(self) -> usize {
		let [id_0, id_1, id_2, id_3] = self.0.to_le_bytes();
		let [port_0, port_1] = self.1.to_le_bytes();

		usize::from_le_bytes([id_0, id_1, port_0, id_2, port_1, id_3, 0, 0])
	}

	pub const DANGLING: Self = Self(u32::MAX, u16::MAX);
}
