open IR

type edge = { source : int; destination : int; copied : bool }
type 'item sole = Nothing | Sole of 'item | Several

(** A flow into an output several readers reach, as every branch reaches a Match's, is a copy at its reader; any other
    joins its two ends into one congruence. A slot of a forwarding node's tagged output that no flow reaches belongs to
    variants it does not hold, so its whole congruence is undefined. *)
type 'x t

val types : 'x t -> 'x TypeArena.t
val congruence : 'x t -> int -> int
val join : 'x t -> int -> int -> unit
val undefined : 'x t -> int -> bool

(** Valid only until coalescing joins congruences. *)
val value_of : 'x t -> int -> int

val reached : 'x t -> int -> bool
val copied : 'x t -> int -> bool
val computes_nothing : 'x t -> int -> bool

(** Whether all the node computes is its first leaf, from nothing, as a sum's tag is beside the fields flowing through.
*)
val computes_first : 'x t -> int -> bool

val edges : 'x t -> int -> edge array
val create : 'x Layouts.t -> 'x Schedule.t -> 'x Ports.t -> 'x t
