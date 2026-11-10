open IR
open Definitions

(** A region nested in an outer one through uniform Matches, with the outer link each link it maps carries; a field the
    branch knows better maps to none. *)
type 'x t = { path : ('x node * int) list; nodes : 'x node array; outer : ('x, 'x link) LinkTable.t }

(** Visits outer regions first, so a visit may map more of a region's links for those within it. *)
val visit : 'x node array -> ('x t -> unit) -> unit
