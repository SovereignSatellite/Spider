(** Moves a node reading a choice (a Match's outputs, also through the sums and tags carrying them, or a sum's tag) into
    the branches where that removes its work: each branch computes nothing for it, or keeps the one branch of an inner
    Match it takes, except that one branch of a Match already there may keep the whole node, which then runs on that
    path alone. No piece of the node lands in two branches. *)

open IR
open Definitions

val optimization : Options.optimization
val run : 'x type_arena -> ('x node -> 'x link array option) -> 'x Readers.table -> bool
