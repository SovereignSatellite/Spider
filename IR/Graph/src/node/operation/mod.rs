//! Core operation node types.
//!
//! Each concept file contains the node's struct definition, its `add_into`
//! constructor, and its visitor impl colocated for SRP at the file level.

mod conversion;
mod memory;
mod mutable;
mod plumbing;
mod structured;
mod table;

pub mod integer;
pub mod number;

pub use self::{
	conversion::{
		ExtendType, IntegerConvertToNumber, IntegerNarrow, IntegerSignExtend,
		IntegerTransmuteToNumber, IntegerWiden, NumberNarrow, NumberTransmuteToInteger,
		NumberTruncateToInteger, NumberWiden,
	},
	memory::{
		LoadType, Location, MemoryCopy, MemoryDrop, MemoryFill, MemoryGrow, MemoryLoad, MemoryNew,
		MemorySize, MemoryStore, StoreType,
	},
	mutable::{MutableGet, MutableNew, MutableSet},
	plumbing::{Apply, Fence, Identity, RefIsNull},
	structured::{Aggregate, Extract},
	table::{TableCopy, TableDrop, TableFill, TableGet, TableGrow, TableNew, TableSet, TableSize},
};
