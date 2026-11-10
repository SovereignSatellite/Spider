open IR
open Definitions

val built : 'x node -> (int * 'x link array) list option
val builds : 'x node -> int -> bool
val producing : 'x node -> bool
val fields : 'x node -> int -> 'x link array option
val field : 'x node -> int -> int -> 'x link option
