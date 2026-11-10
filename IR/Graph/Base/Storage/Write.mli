(** A write gives its input 0, written, as an output of the type that input has when the write is made. *)

open Definitions

val array_set : 'x type_arena -> 'x link -> 'x link -> 'x link -> 'x node
val array_fill : 'x type_arena -> 'x link -> 'x link -> 'x link -> 'x link -> 'x node
val array_copy : 'x type_arena -> 'x link -> 'x link -> 'x link -> 'x link -> 'x link -> 'x node
val box_set : 'x type_arena -> 'x link -> 'x link -> 'x node
