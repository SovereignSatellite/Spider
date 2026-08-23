use super::Location;

/// A table element read instruction.
#[derive(Clone, Copy, Debug)]
pub struct TableGet {
	/// The destination register.
	pub destination: u16,
	/// The source location.
	pub source: Location,
}

/// A table element write instruction.
#[derive(Clone, Copy, Debug)]
pub struct TableSet {
	/// The destination location.
	pub destination: Location,
	/// The source register.
	pub source: u16,
}

/// A table size query instruction.
#[derive(Clone, Copy, Debug)]
pub struct TableSize {
	/// The destination register.
	pub destination: u16,
	/// The table index.
	pub table: u16,
}

/// A table grow instruction.
#[derive(Clone, Copy, Debug)]
pub struct TableGrow {
	/// The destination register.
	pub destination: u16,
	/// The table index.
	pub table: u16,
	/// The growth size register.
	pub size: u16,
	/// The initial value register.
	pub initializer: u16,
}

/// A table fill instruction.
#[derive(Clone, Copy, Debug)]
pub struct TableFill {
	/// The destination location.
	pub destination: Location,
	/// The fill value register.
	pub source: u16,
	/// The number of elements register.
	pub size: u16,
}

/// A table copy instruction.
#[derive(Clone, Copy, Debug)]
pub struct TableCopy {
	/// The destination location.
	pub destination: Location,
	/// The source location.
	pub source: Location,
	/// The number of elements register.
	pub size: u16,
}

/// A table initialization instruction.
#[derive(Clone, Copy, Debug)]
pub struct TableInit {
	/// The destination location.
	pub destination: Location,
	/// The source location.
	pub source: Location,
	/// The number of elements register.
	pub size: u16,
}

/// An element segment drop instruction.
#[derive(Clone, Copy, Debug)]
pub struct ElementsDrop {
	/// The element segment index.
	pub source: u16,
}
