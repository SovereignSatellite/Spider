(** A Repeat's body runs at least once, so its work reading only what the loop carries unchanged moves before the loop
    and the region of its step's one exit after it. A node only some branches of a Match read already runs only where a
    branch reads it, so it moves into those branches. *)

val optimization : Options.optimization

(** A Repeat or Match moving into a Match moves whole, so what crosses its own boundaries waits for that branch's round.
*)
val run : 'x Readers.table -> bool
