//! Defines core operation nodes and their constructors.

mod conversion;
mod host;
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
	host::{Export, Import},
	memory::{
		LoadType, Location, MemoryCopy, MemoryDrop, MemoryFill, MemoryLoad, MemoryNew, MemoryStore,
		StoreType,
	},
	mutable::{MutableGet, MutableNew, MutableSet},
	plumbing::{Apply, Fence, Identity, RefIsNull},
	structured::{Aggregate, Extract},
	table::{TableCopy, TableDrop, TableFill, TableGet, TableGrow, TableNew, TableSet, TableSize},
};
