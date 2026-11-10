open IR

type 'x t

val evaluated : 'x t -> 'x Region.t -> 'x Definitions.node list
val inlined : 'x t -> 'x Link.t -> int -> 'x Link.t option
val deciding : 'x t -> 'x Region.t -> 'x Link.t -> ('x Node.t * int) option
val following : 'x t -> 'x Node.t -> ('x Node.t * 'x Link.t) option
val register : 'x t -> 'x Link.t -> int -> Register.t option

(** The moves for the reader, made at the end of the branch for a branch's results, before the loop for a Repeat, and
    after the exit test for a Repeat body's results. *)
val moves : 'x t -> 'x Node.t -> 'x Move.t list

val registers : 'x t -> Register.t list
val leaf_types : 'x t -> Register.t -> Type.t list

(** [reads_once reader input held]: whether the reader evaluates the input once while registers hold [held] of its
    output leaves, which decides what is left to be computed where read; [computed_where_read node]: whether the node
    can be computed in the expression reading it at all. *)
val allocate :
  'x Layouts.t -> RegisterFile.t -> ('x Node.t -> int -> int -> bool) -> ('x Node.t -> bool) -> 'x Region.t -> 'x t
