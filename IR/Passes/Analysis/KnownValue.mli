open IR
open Definitions

type 'x knowledge = SameAs of 'x link | Unproduced | Unknown

val across_branches : 'x type_arena -> 'x node -> int -> ('x link -> 'x knowledge) -> 'x knowledge
val field_through : 'x type_arena -> ('x link -> 'x link) -> int -> int -> 'x link -> 'x knowledge
val carried_outside : 'x type_arena -> ('x link -> 'x link) -> 'x node -> 'x link -> 'x link option
val carried_known : 'x type_arena -> 'x node -> int -> 'x link option
