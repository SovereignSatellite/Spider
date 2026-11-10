open IR
open Definitions
open Entities

val apply : 'x context -> 'x State.t -> Callee.t -> 'x link -> 'x link -> 'x link list -> 'x link list * 'x State.t
val function_value : 'x context -> 'x State.t -> int -> 'x link
val call : 'x context -> 'x State.t -> int -> 'x link list -> 'x link list * 'x State.t
val reference : 'x context -> 'x State.t -> int -> 'x link
