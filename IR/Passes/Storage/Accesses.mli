open IR
open Definitions

type 'x access = { reference : 'x link; index : 'x link; offset : int64; width : int64; value_type : Type.t }

val read : 'x type_arena -> 'x node -> 'x access option
val write : 'x type_arena -> 'x node -> ('x access * 'x link) option
val bulk_write : 'x node -> ('x link * 'x link) option
val root : 'x type_arena -> 'x link -> 'x link

(** Unlike [root], it stops at a fence, since what the fence waits for may have changed what the reference holds. *)
val unwritten : 'x type_arena -> 'x link -> 'x link

val creation : 'x link -> 'x node option
val low_word : int64
val same_index : 'x access -> 'x access -> bool option

type overlap = Inside of int64 | Outside | Unknown

val overlap : 'x access -> 'x link -> 'x link -> overlap
val changes : 'x type_arena -> 'x access -> 'x node -> bool
val writes_anything : 'x type_arena -> 'x node -> bool
