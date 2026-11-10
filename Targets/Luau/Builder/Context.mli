open IR
open Allocator
open Emission
open Tree
open LuauExtension

type t = {
  layouts : x Layouts.t;
  (* Luau: Luau is typed, and bodies sit in constant tables, which its compiler resolves statically. *)
  luau : LuauTypes.t;
  functions : x FunctionQueue.t;
  tables : (x, Tables.table) RegionTable.t;
  helpers : RuntimeLibrary.NeededHelpers.t;
  labels : (x, string) RegionTable.t;
}

(* Luau: Luau is typed, and bodies sit in constant tables, which its compiler resolves statically. *)
val create : x TypeArena.t -> RuntimeLibrary.t -> (x, Tables.table) RegionTable.t -> (x, string) RegionTable.t -> t
val table_name : Tables.table -> Name.t
val type_name : Tables.table -> Name.t
val function_of : t -> x Region.t -> Expression.t
