//! Lowerings for memory loads and stores.

use ir_graph::{
	Link, Node,
	operation::{LoadType, Location, MemoryLoad, MemoryStore, StoreType},
	region::Match,
};
use luajit_foreign::{
	BooleanToInteger, CastAnyPointer, CastI64, CastU8Pointer, LuaJITAdd, LuaJITLessThan,
	MemoryData, MemorySize, PointerLoad, PointerStore,
};

use crate::{boolean::either, replace};

#[derive(Clone, Copy)]
struct Access {
	field: &'static str,
	width: i32,
}

/// Lowers a memory load or store in place, reporting whether one matched.
#[must_use = "propagate whether this pass changed the graph"]
pub fn lower(nodes: &mut Vec<Node>, id: u32) -> bool {
	let index = usize::try_from(id).unwrap();

	if let &Node::MemoryLoad(MemoryLoad { source, kind }) = &nodes[index] {
		load(nodes, id, source, kind);

		return true;
	}

	if let &Node::MemoryStore(MemoryStore {
		destination,
		source,
		kind,
	}) = &nodes[index]
	{
		store(nodes, id, destination, source, kind);

		return true;
	}

	false
}

fn load(nodes: &mut Vec<Node>, id: u32, source: Location, kind: LoadType) {
	let condition = out_of_bounds(nodes, source, load_access(kind).width);
	let matcher = Match::add_if_into(
		nodes,
		vec![source.reference, source.offset],
		condition,
		move |nodes, arguments| {
			vec![load_value(
				nodes,
				Location {
					reference: Link(arguments, 0),
					offset: Link(arguments, 1),
				},
				kind,
			)]
		},
		|nodes, _arguments| vec![Node::add_trap_into(nodes)],
	);
	let value = Link(matcher, 0);

	replace::replace_read(nodes, id, value, source.reference, &[value]);
}

const fn load_access(kind: LoadType) -> Access {
	match kind {
		LoadType::I32_S8 | LoadType::I64_S8 => Access {
			field: "i8",
			width: 1,
		},
		LoadType::I32_U8 | LoadType::I64_U8 => Access {
			field: "u8",
			width: 1,
		},
		LoadType::I32_S16 | LoadType::I64_S16 => Access {
			field: "i16",
			width: 2,
		},
		LoadType::I32_U16 | LoadType::I64_U16 => Access {
			field: "u16",
			width: 2,
		},
		LoadType::I32 | LoadType::I64_S32 | LoadType::F32 => Access {
			field: "i32",
			width: 4,
		},
		LoadType::I64_U32 => Access {
			field: "u32",
			width: 4,
		},
		LoadType::I64 | LoadType::F64 => Access {
			field: "i64",
			width: 8,
		},
	}
}

fn store(nodes: &mut Vec<Node>, id: u32, destination: Location, source: Link, kind: StoreType) {
	let access = store_access(kind);
	let condition = out_of_bounds(nodes, destination, access.width);
	let matcher = Match::add_if_into(
		nodes,
		vec![destination.reference, destination.offset, source],
		condition,
		move |nodes, arguments| {
			vec![store_value(
				nodes,
				Location {
					reference: Link(arguments, 0),
					offset: Link(arguments, 1),
				},
				Link(arguments, 2),
				access,
			)]
		},
		|nodes, _arguments| vec![Node::add_trap_into(nodes)],
	);

	replace::replace_node(nodes, id, &[Link(matcher, 0)]);
}

const fn store_access(kind: StoreType) -> Access {
	match kind {
		StoreType::I32_I8 | StoreType::I64_I8 => Access {
			field: "i8",
			width: 1,
		},
		StoreType::I32_I16 | StoreType::I64_I16 => Access {
			field: "i16",
			width: 2,
		},
		StoreType::I32 | StoreType::I64_I32 | StoreType::F32 => Access {
			field: "i32",
			width: 4,
		},
		StoreType::I64 | StoreType::F64 => Access {
			field: "i64",
			width: 8,
		},
	}
}

fn out_of_bounds(nodes: &mut Vec<Node>, location: Location, width: i32) -> Link {
	let zero = Node::add_i32_into(nodes, 0);
	let negative = LuaJITLessThan::add_into(nodes, location.offset, zero);
	let width = Node::add_i32_into(nodes, width);
	let end = LuaJITAdd::add_into(nodes, location.offset, width);
	let size = MemorySize::add_into(nodes, location.reference);
	let beyond = LuaJITLessThan::add_into(nodes, size, end);
	let outside = either(nodes, negative, beyond);

	BooleanToInteger::add_into(nodes, outside)
}

fn load_value(nodes: &mut Vec<Node>, source: Location, kind: LoadType) -> Link {
	let access = load_access(kind);
	let pointer = pointer_at(nodes, source);
	let value = PointerLoad::add_into(nodes, pointer, access.field);

	match kind {
		LoadType::I32_S8
		| LoadType::I32_U8
		| LoadType::I32_S16
		| LoadType::I32_U16
		| LoadType::I32
		| LoadType::I64
		| LoadType::F32
		| LoadType::F64 => value,
		LoadType::I64_S8
		| LoadType::I64_U8
		| LoadType::I64_S16
		| LoadType::I64_U16
		| LoadType::I64_S32
		| LoadType::I64_U32 => CastI64::add_into(nodes, value),
	}
}

fn store_value(nodes: &mut Vec<Node>, destination: Location, source: Link, access: Access) -> Link {
	let pointer = pointer_at(nodes, destination);

	PointerStore::add_into(nodes, destination.reference, pointer, access.field, source)
}

fn pointer_at(nodes: &mut Vec<Node>, location: Location) -> Link {
	let data = MemoryData::add_into(nodes, location.reference);
	let address = CastU8Pointer::add_into(nodes, data);
	let address = LuaJITAdd::add_into(nodes, address, location.offset);

	CastAnyPointer::add_into(nodes, address)
}
