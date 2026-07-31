//! Lowerings for memory loads and stores.

use ir_graph::{
	Link, Node,
	operation::{LoadType, Location, StoreType},
};
use luau_foreign::{
	Bit32ArShift, Bit32Or, BufferLoad, BufferStore, FromBitsI64, IntoBitsI64, LuauAdd,
};

use crate::replace;

const WORD_BYTES: u32 = 4;
const SIGN_SHIFT: u32 = 31;

const READ_I8: &str = "buffer_read_i8";
const READ_U8: &str = "buffer_read_u8";
const READ_I16: &str = "buffer_read_i16";
const READ_U16: &str = "buffer_read_u16";
const READ_I32: &str = "buffer_read_i32";
const READ_U32: &str = "buffer_read_u32";
const READ_F64: &str = "buffer_read_f64";

const WRITE_U8: &str = "buffer_write_u8";
const WRITE_U16: &str = "buffer_write_u16";
const WRITE_U32: &str = "buffer_write_u32";
const WRITE_F64: &str = "buffer_write_f64";

pub fn lower_load(nodes: &mut Vec<Node>, identifier: u32, source: Location, kind: LoadType) {
	let mut reads = Vec::new();
	let value = load_value(nodes, source.reference, source.offset, kind, &mut reads);

	replace::replace_read(nodes, identifier, value, source.reference, &reads);
}

fn load_value(
	nodes: &mut Vec<Node>,
	buffer: Link,
	offset: Link,
	kind: LoadType,
	reads: &mut Vec<Link>,
) -> Link {
	match kind {
		LoadType::I32_S8 => read_signed(nodes, buffer, offset, READ_I8, reads),
		LoadType::I32_U8 => read(nodes, buffer, offset, READ_U8, reads),
		LoadType::I32_S16 => read_signed(nodes, buffer, offset, READ_I16, reads),
		LoadType::I32_U16 => read(nodes, buffer, offset, READ_U16, reads),
		LoadType::I32 | LoadType::F32 => read(nodes, buffer, offset, READ_U32, reads),
		LoadType::I64_S8 => widen_read_signed(nodes, buffer, offset, READ_I8, reads),
		LoadType::I64_U8 => widen_read_unsigned(nodes, buffer, offset, READ_U8, reads),
		LoadType::I64_S16 => widen_read_signed(nodes, buffer, offset, READ_I16, reads),
		LoadType::I64_U16 => widen_read_unsigned(nodes, buffer, offset, READ_U16, reads),
		LoadType::I64_S32 => widen_read_signed(nodes, buffer, offset, READ_I32, reads),
		LoadType::I64_U32 => widen_read_unsigned(nodes, buffer, offset, READ_U32, reads),
		LoadType::I64 => load_long(nodes, buffer, offset, reads),
		LoadType::F64 => read(nodes, buffer, offset, READ_F64, reads),
	}
}

fn read(
	nodes: &mut Vec<Node>,
	buffer: Link,
	offset: Link,
	name: &'static str,
	reads: &mut Vec<Link>,
) -> Link {
	let value = BufferLoad::add_into(nodes, name, buffer, offset);

	reads.push(value);

	value
}

// A signed-width read returns a Luau-negative number; folding it through `bit32` re-encodes
// it as the two's-complement word the integer representation expects.
fn read_signed(
	nodes: &mut Vec<Node>,
	buffer: Link,
	offset: Link,
	name: &'static str,
	reads: &mut Vec<Link>,
) -> Link {
	let value = read(nodes, buffer, offset, name, reads);

	Bit32Or::add_fast_into(nodes, value, 0)
}

fn widen_read_signed(
	nodes: &mut Vec<Node>,
	buffer: Link,
	offset: Link,
	name: &'static str,
	reads: &mut Vec<Link>,
) -> Link {
	let low = read_signed(nodes, buffer, offset, name, reads);

	widen_signed(nodes, low)
}

fn widen_read_unsigned(
	nodes: &mut Vec<Node>,
	buffer: Link,
	offset: Link,
	name: &'static str,
	reads: &mut Vec<Link>,
) -> Link {
	let low = read(nodes, buffer, offset, name, reads);

	widen_unsigned(nodes, low)
}

fn widen_unsigned(nodes: &mut Vec<Node>, low: Link) -> Link {
	let high = Node::add_i32_into(nodes, 0);

	IntoBitsI64::add_into(nodes, low, high)
}

fn widen_signed(nodes: &mut Vec<Node>, low: Link) -> Link {
	let high = Bit32ArShift::add_fast_into(nodes, low, SIGN_SHIFT);

	IntoBitsI64::add_into(nodes, low, high)
}

fn load_long(nodes: &mut Vec<Node>, buffer: Link, offset: Link, reads: &mut Vec<Link>) -> Link {
	let low = read(nodes, buffer, offset, READ_U32, reads);
	let high_offset = LuauAdd::add_fast_into(nodes, offset, WORD_BYTES);
	let high = read(nodes, buffer, high_offset, READ_U32, reads);

	IntoBitsI64::add_into(nodes, low, high)
}

pub fn lower_store(
	nodes: &mut Vec<Node>,
	identifier: u32,
	destination: Location,
	source: Link,
	kind: StoreType,
) {
	let state = store_value(nodes, destination, source, kind);

	replace::replace_node(nodes, identifier, &[state]);
}

fn store_value(
	nodes: &mut Vec<Node>,
	destination: Location,
	source: Link,
	kind: StoreType,
) -> Link {
	match kind {
		StoreType::I32_I8 => write(nodes, destination, source, WRITE_U8),
		StoreType::I32_I16 => write(nodes, destination, source, WRITE_U16),
		StoreType::I32 | StoreType::F32 => write(nodes, destination, source, WRITE_U32),
		StoreType::I64_I8 => write_low(nodes, destination, source, WRITE_U8),
		StoreType::I64_I16 => write_low(nodes, destination, source, WRITE_U16),
		StoreType::I64_I32 => write_low(nodes, destination, source, WRITE_U32),
		StoreType::I64 => store_long(nodes, destination, source),
		StoreType::F64 => write(nodes, destination, source, WRITE_F64),
	}
}

fn write(nodes: &mut Vec<Node>, destination: Location, value: Link, name: &'static str) -> Link {
	BufferStore::add_into(
		nodes,
		name,
		destination.reference,
		destination.offset,
		value,
	)
}

fn write_low(
	nodes: &mut Vec<Node>,
	destination: Location,
	source: Link,
	name: &'static str,
) -> Link {
	let (low, _) = FromBitsI64::add_into(nodes, source);

	write(nodes, destination, low, name)
}

fn store_long(nodes: &mut Vec<Node>, destination: Location, source: Link) -> Link {
	let (low, high) = FromBitsI64::add_into(nodes, source);
	let high_offset = LuauAdd::add_fast_into(nodes, destination.offset, WORD_BYTES);
	let high_write =
		BufferStore::add_into(nodes, WRITE_U32, destination.reference, high_offset, high);

	// The low write threads the high write's state token (the forwarded reference) so both
	// writes survive elimination and stay ordered one after the other.
	BufferStore::add_into(nodes, WRITE_U32, high_write, destination.offset, low)
}
