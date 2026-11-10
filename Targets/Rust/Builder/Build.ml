open Emission
open Tree

let runtime = RuntimeLibrary.of_resources "// needs " Resources.resources
let whole_runtime () = List.map (fun code -> Item.Source code) (RuntimeLibrary.sources runtime)

let labelled options labels compilation_unit =
  Lowering.run options compilation_unit;
  let context = Context.create compilation_unit.types runtime labels in
  let start = Context.function_of context compilation_unit.root in
  RuntimeLibrary.NeededHelpers.need context.helpers "Imports";
  RuntimeLibrary.NeededHelpers.need context.helpers "Exports";
  let functions =
    FunctionQueue.build context.functions (fun definition name -> Functions.build context name definition)
  in
  let sources = List.map (fun code -> Item.Source code) (RuntimeLibrary.NeededHelpers.sources context.helpers) in
  let definitions = RustTypes.definitions context.rust in
  sources @ definitions @ functions
  @ [ Item.Function (Text "run", [], Tuple [], [ Statement.Call (Call (start, [])) ]) ]
