open IR
open Definitions

val optimization : Options.optimization

(** Callees settle before their callers, so a body is judged at its final size; a callee still settling when its caller
    is reached reaches that caller, so it is recursive and stays a function. *)
val run : 'x region list -> 'x RegionSet.t
