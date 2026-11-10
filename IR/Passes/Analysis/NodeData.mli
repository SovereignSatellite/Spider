open IR
open Definitions

type 'x t

val of_node : 'x type_arena -> 'x node -> 'x t option
val equal : 'x t -> 'x t -> bool
val hash : 'x t -> int
val idempotent : 'x type_arena -> 'x node -> bool
