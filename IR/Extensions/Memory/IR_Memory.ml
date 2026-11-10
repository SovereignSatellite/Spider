open IR

type load = [ `MemoryLoadI8 of int | `MemoryLoadI16 of int | `MemoryLoadI32 of int | `MemoryLoadI64 of int ]
type store = [ `MemoryStoreI8 of int | `MemoryStoreI16 of int | `MemoryStoreI32 of int | `MemoryStoreI64 of int ]
type t = [ load | store ]

let name node =
  match node with
  | `MemoryLoadI8 _ -> "Memory.LoadI8"
  | `MemoryLoadI16 _ -> "Memory.LoadI16"
  | `MemoryLoadI32 _ -> "Memory.LoadI32"
  | `MemoryLoadI64 _ -> "Memory.LoadI64"
  | `MemoryStoreI8 _ -> "Memory.StoreI8"
  | `MemoryStoreI16 _ -> "Memory.StoreI16"
  | `MemoryStoreI32 _ -> "Memory.StoreI32"
  | `MemoryStoreI64 _ -> "Memory.StoreI64"

let offset node =
  match node with
  | `MemoryLoadI8 offset
  | `MemoryLoadI16 offset
  | `MemoryLoadI32 offset
  | `MemoryLoadI64 offset
  | `MemoryStoreI8 offset
  | `MemoryStoreI16 offset
  | `MemoryStoreI32 offset
  | `MemoryStoreI64 offset ->
      offset

let value_type node =
  match node with
  | `MemoryLoadI8 _ | `MemoryStoreI8 _ -> Type.i8
  | `MemoryLoadI16 _ | `MemoryStoreI16 _ -> Type.i16
  | `MemoryLoadI32 _ | `MemoryStoreI32 _ -> Type.i32
  | `MemoryLoadI64 _ | `MemoryStoreI64 _ -> Type.i64

let output_count _ _ _ = 1
let output_type _ _ extended _ = match extended with #load -> value_type extended | #store -> Type.bytes
let input_type _ _ extended index = match index with 0 -> Type.bytes | 1 -> Type.i32 | _ -> value_type extended
let base node : Definitions.base = match node with #load -> Operation | #store -> Write
let holds _ = 0
let commutative _ = false
let unrepeatable _ = false

let array_access node : Definitions.array_access option =
  Some { offset = offset node; width = Type.bits (value_type node) / 8 }

let describe _ = invalid_arg "Memory defines no types."

let extension : t Definitions.extension =
  { name; output_count; output_type; input_type; base; holds; commutative; unrepeatable; array_access; describe }
