use ir_graph::operation::{LoadType, StoreType};

use super::Location;

/// A memory load instruction.
#[derive(Clone, Copy, Debug)]
pub struct MemoryLoad {
	/// The destination register.
	pub destination: u16,
	/// The source location.
	pub source: Location,
	/// The load type.
	pub kind: LoadType,
}

/// A memory store instruction.
#[derive(Clone, Copy, Debug)]
pub struct MemoryStore {
	/// The destination location.
	pub destination: Location,
	/// The source register.
	pub source: u16,
	/// The store type.
	pub kind: StoreType,
}

/// A memory size query instruction.
#[derive(Clone, Copy, Debug)]
pub struct MemorySize {
	/// The destination register.
	pub destination: u16,
	/// The memory index.
	pub memory: u16,
}

impl MemorySize {
	/// The size of a memory page in bytes.
	pub const PAGE_SIZE: usize = 0x1_0000;
	/// Define the largest page count whose byte size fits in a 32-bit integer.
	pub const PAGE_LIMIT: usize = 0xFFFF;
}

/// A memory grow instruction.
#[derive(Clone, Copy, Debug)]
pub struct MemoryGrow {
	/// The destination register.
	pub destination: u16,
	/// The memory index.
	pub memory: u16,
	/// The growth size register.
	pub size: u16,
}

/// A memory fill instruction.
#[derive(Clone, Copy, Debug)]
pub struct MemoryFill {
	/// The destination location.
	pub destination: Location,
	/// The fill byte register.
	pub byte: u16,
	/// The fill size register.
	pub size: u16,
}

/// A memory copy instruction.
#[derive(Clone, Copy, Debug)]
pub struct MemoryCopy {
	/// The destination location.
	pub destination: Location,
	/// The source location.
	pub source: Location,
	/// The copy size register.
	pub size: u16,
}

/// A memory initialization instruction.
#[derive(Clone, Copy, Debug)]
pub struct MemoryInit {
	/// The destination location.
	pub destination: Location,
	/// The source location.
	pub source: Location,
	/// The initialization size register.
	pub size: u16,
}

/// A data segment drop instruction.
#[derive(Clone, Copy, Debug)]
pub struct DataDrop {
	/// The data segment index.
	pub source: u16,
}
