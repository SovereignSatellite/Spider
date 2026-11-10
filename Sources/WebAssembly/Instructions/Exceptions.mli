open IR
open Definitions
open Entities

val exception_ : 'x context -> 'x State.t -> int -> 'x link list -> 'x link
val catch : 'x context -> 'x State.t -> int -> 'x link -> 'x link list
val raise : 'x State.t -> 'x link -> 'x link list * 'x State.t
