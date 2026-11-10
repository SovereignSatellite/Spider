(** Where the step continues exactly when a two-way Match in the body takes variant 1, the head (what does not depend on
    the Match) runs once before the loop and skips it on variant 0, and the loop runs the rest, then the head, testing
    the head's condition last. The exiting branch and what follows it run after the loop, on the head's values at exit.
*)

val optimization : Options.optimization
val run : 'x Readers.table -> bool
