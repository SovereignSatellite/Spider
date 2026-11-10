(** On a restructured graph one backward pass over the blocks suffices without loops; with loops, a second pass carries
    each loop head's set to its latch, which covers any nesting depth. *)

val run : Graph.t -> int list -> int array array
