/// Reserved local-variable slots used as branch conditions and assignment scratch.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ReservedLocal {
	/// Select a branch arm.
	BranchSelector = 0,
	/// Carry a repeat-region condition.
	RepeatCondition = 1,
	/// Select a repeat-region path.
	RepeatSelector = 2,
	/// Hold decoder scratch values.
	DecoderScratch = 3,
}

impl ReservedLocal {
	/// The number of reserved local-variable slots.
	pub const COUNT: u16 = Self::DecoderScratch as u16 + 1;
}

/// A local variable assignment.
#[derive(Clone, Copy, Debug)]
pub struct LocalSet {
	/// The destination register.
	pub destination: u16,
	/// The source register.
	pub source: u16,
}

/// A conditional branch on a local variable.
#[derive(Clone, Copy, Debug)]
pub struct LocalBranch {
	/// The source register.
	pub source: u16,
}

/// A function call instruction.
#[derive(Clone, Copy, Debug)]
pub struct Call {
	/// The destination register range.
	pub destinations: (u16, u16),
	/// The source register range.
	pub sources: (u16, u16),
	/// The function register.
	pub function: u16,
}
