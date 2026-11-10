(** Function bodies share no nodes or regions, so work confined to each body runs on them in parallel. OCaml's runtime
    bounds a process's domains, so only the main domain spawns helpers. *)

open IR
open Definitions

val in_parallel : ('x RegionSet.t -> 'x region -> unit) -> 'x region list -> 'x RegionSet.t
