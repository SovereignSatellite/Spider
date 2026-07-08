//! The allocatable value: an exact live interval and kind mask.

/// The id of a value with no register partner.
pub const NONE: u32 = u32::MAX;

/// One allocatable value with an exact live interval.
#[derive(Clone, Copy)]
pub struct Value {
	/// The first time point the value is live.
	pub start: u32,
	/// The last time point the value is live.
	pub end: u32,
	/// The kind mask the value requires.
	pub kind: u8,
}

impl Value {
	/// Creates a value whose interval starts and ends at the given time.
	#[must_use]
	pub const fn at(time: u32, kind: u8) -> Self {
		Self {
			start: time,
			end: time,
			kind,
		}
	}
}
