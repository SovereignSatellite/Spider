open IR

(** A load reads the value's bytes at the address plus the offset, least significant first; a store writes them there. A
    range out of bounds is undefined. *)
type load = [ `MemoryLoadI8 of int | `MemoryLoadI16 of int | `MemoryLoadI32 of int | `MemoryLoadI64 of int ]

type store = [ `MemoryStoreI8 of int | `MemoryStoreI16 of int | `MemoryStoreI32 of int | `MemoryStoreI64 of int ]
type t = [ load | store ]

val name : t -> string
val offset : [< t ] -> int
val output_count : 'x TypeArena.t -> 'x Node.t -> t -> int
val output_type : 'x TypeArena.t -> 'x Node.t -> t -> int -> Type.t
val input_type : 'x TypeArena.t -> 'x Node.t -> t -> int -> Type.t
val base : t -> Definitions.base
val holds : t -> int
val commutative : t -> bool
val unrepeatable : t -> bool
val array_access : t -> Definitions.array_access option
val extension : t Definitions.extension
