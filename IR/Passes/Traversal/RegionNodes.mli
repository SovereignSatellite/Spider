open IR
open Definitions

val ordered_with : 'x NodeSet.t -> 'x node Dynarray.t -> 'x region -> 'x node array
val ordered : 'x region -> 'x node array
val crossing : ('x node -> bool) -> 'x node array -> 'x link array
val any_runs : ('x node -> bool) -> 'x node -> bool
