(** After Reissmann et al., "Efficient Control Flow Restructuring for GPUs" (2016). *)

(** Numbers the result so a loop spans its head to its latch, a branch's head precedes its arms (in successor order) and
    then its continuation point, and the exit is last. *)
val run : Graph.t -> Graph.t
