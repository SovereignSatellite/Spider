(* Luau: bodies sit in constant tables, which the Luau compiler resolves statically. *)
open IR
open LuauExtension

type table = Assigned | Constant of int

(** The groups in increasing order of key, each holding its items in their order. *)
val group_by : ('item -> 'key) -> 'item list -> ('key * 'item list) list

val group : x Region.t -> (x, table) RegionTable.t
