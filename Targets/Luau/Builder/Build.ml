(* Luau: typed, with bodies in constant tables its compiler resolves statically. *)
open IR
open Emission
open Tree

(* Luau: typed, with bodies in constant tables its compiler resolves statically. *)
open LuauExtension

let runtime = RuntimeLibrary.of_resources "-- needs " Resources.resources
let whole_runtime () = List.map (fun code -> Statement.Source code) (RuntimeLibrary.sources runtime)

(* Luau: a function has 255 registers. *)

(** Below Luau's 200 locals, leaving room under its 255 registers for what expressions need. *)
let maximum_locals = 180

(* Luau: typed, with bodies in constant tables its compiler resolves statically. *)
let table_type (context : Context.t) members =
  LuauType.table
    (List.map
       (fun (name, (definition : x Region.t), _) -> (name, LuauTypes.leaf_type context.luau definition.region_type))
       members)

let define_table table members : Statement.t =
  let type_name = LuauType.Named (Context.type_name table) in
  match table with
  | Assigned -> Const (Context.table_name table, None, Cast (Table ([], []), type_name))
  | Constant _ ->
      Const
        (Context.table_name table, Some type_name, Record (List.map (fun (name, _, defined) -> (name, defined)) members))

let assign (name, _, defined) : Statement.t = Assign ([ Field (Name (Context.table_name Assigned), name) ], [ defined ])

let items_within maximum_locals options labels compilation_unit =
  Lowering.run options compilation_unit;
  (* Luau: typed, with bodies in constant tables its compiler resolves statically. *)
  let tables = Tables.group compilation_unit.root in
  let context = Context.create compilation_unit.types runtime tables labels in
  let start = Context.function_of context compilation_unit.root in
  let imports, exports = ("Imports", "Exports") in
  RuntimeLibrary.NeededHelpers.need context.helpers imports;
  RuntimeLibrary.NeededHelpers.need context.helpers exports;
  let functions =
    FunctionQueue.build context.functions (fun definition name ->
        (* Luau: typed, with bodies in constant tables its compiler resolves statically. *)
        (name, definition, Functions.build context maximum_locals definition))
  in
  let table_of (_, definition, _) = RegionTable.find tables definition in
  (* [Assigned], the first case, sorts first, as bodies in the constant tables may name it. *)
  let grouped = Tables.group_by table_of functions in
  let assigned = List.filter (fun member -> table_of member = Tables.Assigned) functions in
  let sources = List.map (fun code -> Statement.Source code) (RuntimeLibrary.NeededHelpers.sources context.helpers) in
  let definitions = LuauTypes.definitions context.luau in
  let table_types =
    List.map (fun (table, members) -> Statement.Type (Context.type_name table, table_type context members)) grouped
  in
  let defined_tables = List.map (fun (table, members) -> define_table table members) grouped in
  sources @ definitions @ table_types @ defined_tables @ List.map assign assigned
  @ [
      Return
        [
          Record
            [ (Text "imports", Name (Helper imports)); (Text "exports", Name (Helper exports)); (Text "start", start) ];
        ];
    ]

let labelled options labels compilation_unit = items_within maximum_locals options labels compilation_unit
