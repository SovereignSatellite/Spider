open IR

val literal : 'x TypeArena.t -> 'x Definitions.node -> bool

(** The Match whose branches decide the tag of [tested], a result of [region], with the output holding that tag: the
    last node the region evaluates, whose output reaches [tested] through the sums and tags carrying it. *)
val deciding : 'x Schedule.t -> 'x Flows.t -> 'x Definitions.region -> 'x Link.t -> ('x Node.t * int) option

val constant : 'x Link.t -> int option

(** Visits each test that the branches of a Match take over, a loop's step or the selector of the Match its exits run,
    each branch result tested where its branch ends in their place, and each loop with the Match its exits run, which
    decides no exit itself, as its branches then leave a loop enclosing this one. *)
val decided :
  'x Schedule.t ->
  'x Flows.t ->
  ('x Link.t -> unit) ->
  ('x Link.t -> unit) ->
  ('x Node.t -> 'x Node.t * 'x Link.t -> unit) ->
  unit
