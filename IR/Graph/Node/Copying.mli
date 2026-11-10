open Definitions

(** A node of the same kind reading the given inputs, which have this node's input types, with copies of its regions. *)
val node : 'x node -> 'x link array -> 'x node

val links : ('x link -> 'x link option) -> 'x link array -> 'x link array
val results : 'x region -> 'x link array -> 'x link array
