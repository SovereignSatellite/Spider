open LiveRanges

type interval = { congruence : int; leaf : IR.Type.t; mutable ranges : segment; mutable register : Register.t option }

val start : interval -> int
val end_ : interval -> int

(** An interval may leave a gap over a sibling branch, or over a loop body before the value is first defined in it,
    since one branch runs at a time and each iteration defines the value again before reading it. *)
val build : 'x Ports.t -> 'x Flows.t -> 'x Liveness.t -> interval option array
