open IR

type 'x t

(** One walk against evaluation order sees every use of a value before its definition, since a region reads only its
    arguments and a loop's next iteration receives them only through copies. *)
val create :
  'x Schedule.t ->
  'x Ports.t ->
  'x Flows.t ->
  ('x Definitions.node -> int -> int -> bool) ->
  ('x Definitions.node -> bool) ->
  'x t

val inlined : 'x t -> int -> bool

(** Whether the node is evaluated apart from its reader, as neither a region's boundary nor a node computing nothing is.
*)
val apart : 'x t -> int -> bool

(** The Match the loop's exits run, with the step's field holding the exit value it selects on. *)
val following : 'x t -> int -> ('x Node.t * 'x Definitions.link) option

val live : 'x t -> int -> bool
val definition : 'x t -> int -> int
val last_use : 'x t -> int -> int
