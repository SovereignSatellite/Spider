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
let create types runtime tables labels =
  let layouts = Layouts.create types in
  let helpers = RuntimeLibrary.NeededHelpers.create runtime in
  {
    layouts;
    (* Luau: Luau is typed, and bodies sit in constant tables, which its compiler resolves statically. *)
    luau = LuauTypes.create types layouts helpers;
    functions = FunctionQueue.create ();
    tables;
    helpers;
    labels;
  }

(* Luau: Luau is typed, and bodies sit in constant tables, which its compiler resolves statically. *)
let table_name (table : Tables.table) : Name.t =
  match table with Assigned -> Text "functions" | Constant index -> Numbered ("functions_", index)

let type_name (table : Tables.table) : Name.t =
  match table with Assigned -> Text "Functions" | Constant index -> Numbered ("Functions_", index)

let function_of context body : Expression.t =
  (* Luau: Luau is typed, and bodies sit in constant tables, which its compiler resolves statically. *)
  Field (Name (table_name (RegionTable.find context.tables body)), FunctionQueue.name context.functions body)
