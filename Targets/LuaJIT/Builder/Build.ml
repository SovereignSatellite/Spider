open Emission
open Tree

let runtime = RuntimeLibrary.of_resources "-- needs " Resources.resources
let whole_runtime () = List.map (fun code -> Statement.Source code) (RuntimeLibrary.sources runtime)

(* LuaJIT: a function has 250 slots. *)

(** Below LuaJIT's 200 locals, leaving room under its 250 slots for what expressions need. *)
let maximum_locals = 180

let assign (name, defined) : Statement.t = Assign ([ Field (Name Context.table_name, name) ], [ defined ])

let items_within maximum_locals options labels compilation_unit =
  Lowering.run options compilation_unit;
  let context = Context.create compilation_unit.types runtime labels in
  let start = Context.function_of context compilation_unit.root in
  let imports, exports = ("Imports", "Exports") in
  RuntimeLibrary.NeededHelpers.need context.helpers imports;
  RuntimeLibrary.NeededHelpers.need context.helpers exports;
  let functions =
    FunctionQueue.build context.functions (fun definition name ->
        (name, Functions.build context maximum_locals definition))
  in
  let sources = List.map (fun code -> Statement.Source code) (RuntimeLibrary.NeededHelpers.sources context.helpers) in
  (* LuaJIT: untyped, with every body in one table. *)
  sources
  @ [ Statement.Local (Context.table_name, Some (Table ([], []))) ]
  @ List.map assign functions
  @ [
      Return
        [
          Record
            [ (Text "imports", Name (Helper imports)); (Text "exports", Name (Helper exports)); (Text "start", start) ];
        ];
    ]

let labelled options labels compilation_unit = items_within maximum_locals options labels compilation_unit
