//! The allocatable value: an exact live interval, affinity hint, and kind mask.

/// The affinity of a value with no preferred register partner.
pub const NONE: u32 = u32::MAX;

/// One allocatable value with an exact live interval.
#[derive(Clone, Copy)]
pub struct Value {
	/// The first time point the value is live.
	pub start: u32,
	/// The last time point the value is live.
	pub end: u32,
	/// A partner value whose register this value prefers, or `NONE`.
	pub affinity: u32,
	/// Whether another value prefers this one's register, so it is held back from
	/// first-fit until that partner can reclaim it.
	pub is_reserved: bool,
	/// The kind mask the value requires.
	pub kind: u8,
}

impl Value {
	/// Creates a value whose interval starts and ends at the given time, with no
	/// affinity.
	#[must_use]
	pub const fn at(time: u32, kind: u8) -> Self {
		Self {
			start: time,
			end: time,
			affinity: NONE,
			is_reserved: false,
			kind,
		}
	}
}
