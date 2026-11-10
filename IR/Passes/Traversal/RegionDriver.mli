(** Regions are closed, so a step unsettles only the regions it edits or makes. *)

open IR
open Definitions

val run_fixpoint : ('x node option -> 'x region -> unit -> 'x Revisit.t) -> 'x RegionSet.t -> 'x region list -> unit
