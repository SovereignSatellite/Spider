open IR
open Definitions

type 'x reads
type 'x settling

val settling : 'x node option -> 'x settling

(** The regions passes noted as edited since the settling began. *)
val edits : 'x settling -> 'x region Dynarray.t

(** Tables sharing a settling share storage for their own readers, valid only until another of them reads its own. *)
type 'x table = { region : 'x region; settling : 'x settling; nodes : 'x node array; own_reads : 'x reads Lazy.t }

val create : ?settling:'x settling -> 'x region -> 'x table
val owner : 'x table -> 'x node option
val renewed : 'x table -> 'x table

(** A pass must note each region inside the region that it changes, which drops that region's cached readers. *)
val edited : 'x table -> 'x region -> unit

val redefine_match : 'x table -> 'x node -> 'x link -> Type.t list -> (int -> 'x link array -> 'x link array) -> unit
val redefine_repeat : 'x table -> 'x node -> 'x link array -> Type.t list -> ('x link array -> 'x link) -> unit
val of_link : 'x table -> 'x region -> 'x link -> ('x node * int) list
val is_read : 'x table -> 'x region -> 'x link -> bool
val only_by : 'x table -> 'x region -> 'x node -> 'x node -> bool
