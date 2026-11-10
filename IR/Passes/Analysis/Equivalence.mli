open IR
open Definitions

val regions : 'x type_arena -> (int -> int -> bool) -> 'x region -> 'x region -> bool
