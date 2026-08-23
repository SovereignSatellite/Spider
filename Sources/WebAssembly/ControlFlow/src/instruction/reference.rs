/// A reference null check instruction.
#[derive(Clone, Copy, Debug)]
pub struct RefIsNull {
	/// The destination register.
	pub destination: u16,
	/// The source register.
	pub source: u16,
}

/// A null reference constant assignment.
#[derive(Clone, Copy, Debug)]
pub struct RefNull {
	/// The destination register.
	pub destination: u16,
}

/// A function reference constant assignment.
#[derive(Clone, Copy, Debug)]
pub struct RefFunction {
	/// The destination register.
	pub destination: u16,
	/// The function index.
	pub function: u16,
}

/// A global variable read instruction.
#[derive(Clone, Copy, Debug)]
pub struct GlobalGet {
	/// The destination register.
	pub destination: u16,
	/// The global index.
	pub source: u16,
}

/// A global variable write instruction.
#[derive(Clone, Copy, Debug)]
pub struct GlobalSet {
	/// The global index.
	pub destination: u16,
	/// The source register.
	pub source: u16,
}

/// The type of an external reference.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ReferenceType {
	/// A function reference.
	Function,

	/// A global variable reference.
	Global,
	/// A table reference.
	Table,
	/// An element segment reference.
	Elements,
	/// A linear memory reference.
	Memory,
	/// A data segment reference.
	Data,
}

impl ReferenceType {
	/// Returns whether this reference type is mutable.
	#[must_use]
	pub const fn is_mutable(self) -> bool {
		matches!(
			self,
			Self::Global | Self::Table | Self::Elements | Self::Memory | Self::Data
		)
	}
}

/// An external reference used by an instruction.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Reference {
	/// The reference type.
	pub kind: ReferenceType,
	/// The reference index.
	pub id: u16,
}
