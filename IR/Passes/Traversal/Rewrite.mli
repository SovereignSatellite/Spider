open IR
open Definitions

val nodes : ('x node -> 'x link array option) -> 'x node array -> bool
val replace : 'x region -> 'x node -> 'x link array -> unit
val substitute : 'x region -> ('x link * 'x link) list -> unit
val splice : 'x region -> 'x link array -> 'x link array
val nested : ('x node -> 'x link array option) -> 'x RegionSet.t -> 'x region -> bool
