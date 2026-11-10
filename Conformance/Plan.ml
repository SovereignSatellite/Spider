open IR
open Suite
open Wasm.Source

type kind = Number of Wasm.Types.numtype | Reference | External

module Kind = struct
  let width kind = match kind with Number _ -> 1 | Reference -> 3 | External -> 2
  let positioned kinds = snd (List.fold_left_map (fun start kind -> (start + width kind, (kind, start))) 0 kinds)
end

type source = Spectest | Exported of int
type binding = { module_ : string; name : string; source : source }
type instance = { number : int; line : int; definition : Wasm.Ast.module_; imports : binding list }
type expectation = Returns of value list | Traps | Finishes

module Expectation = struct
  let describe expectation =
    match expectation with
    | Returns values -> "[" ^ String.concat " " (List.map Value.describe values) ^ "]"
    | Traps -> "a trap"
    | Finishes -> "to finish"

  let values expectation = match expectation with Returns values -> values | Traps | Finishes -> []
end

type argument = Number of Wasm.Types.numtype * int64 | Null | Host of int64
type signature = { parameters : kind list; results : kind list }

type invocation = {
  line : int;
  instance : int;
  function_ : string;
  arguments : argument list;
  signature : signature;
  expectation : expectation;
}

type read = { line : int; instance : int; global : string; kind : kind; expected : value list }
type step = Instantiate of instance | Uninstantiable of instance | Invoke of invocation | Get of read
type disposition = Run of step | Refused of string | Skipped of string | Failed of string
type plan = { test : string; commands : (int * disposition) list }

let steps plan =
  List.filter_map (fun (_, disposition) -> match disposition with Run step -> Some step | _ -> None) plan.commands

let judged plan =
  List.map
    (fun step ->
      match step with
      | Invoke invocation -> (invocation.line, invocation.expectation)
      | Get read -> (read.line, Returns read.expected)
      | Uninstantiable instance -> (instance.line, Traps)
      | Instantiate instance -> (instance.line, Finishes))
    (steps plan)

let compile types options instance =
  let compilation_unit = WebAssembly.Module.lift types instance.definition in
  Passes.Optimizer.run options compilation_unit;
  compilation_unit

let verify compilation_unit =
  match WellFormedness.check compilation_unit with
  | [] -> ()
  | errors -> failwith ("invalid lowering: " ^ String.concat "; " errors)

let kind (value_type : Wasm.Types.valtype) : kind option =
  match value_type with
  | NumT number_type -> Some (Number number_type)
  | RefT (_, (FuncHT | NoFuncHT | UseHT _)) -> Some Reference
  | RefT (_, (ExternHT | NoExternHT)) -> Some External
  | _ -> None

type loaded = Lifted of instance | NotRun of disposition

module Names = Map.Make (String)
module Numbers = Map.Make (Int)
module NumberSet = Set.Make (Int)

type modules = {
  current : loaded option;
  named : loaded Names.t;
  definitions : string Names.t;
  registry : loaded Names.t;
  instances : instance Numbers.t;
  registered : NumberSet.t;
  unknown : NumberSet.t;
}

let rec is_unknown modules instance =
  NumberSet.mem instance.number modules.unknown
  || List.exists
       (fun binding ->
         match binding.source with
         | Exported exporter -> is_unknown modules (Numbers.find exporter modules.instances)
         | Spectest -> false)
       instance.imports

let signature (definition : Wasm.Ast.module_) index =
  match (WebAssembly.Types.composites definition.it.types).(index) with
  | FuncT signature -> signature
  | StructT _ | ArrayT _ -> invalid_arg "Expected a function type."

let declared (kind : Wasm.Types.externtype) =
  match kind with
  | ExternFuncT _ | ExternGlobalT (GlobalT (_, NumT _)) | ExternMemoryT _ | ExternTableT _ -> Ok ()
  | ExternGlobalT _ -> Error "imported reference globals"
  | ExternTagT _ -> Error "Tag imports"

let resolve modules (import : Wasm.Ast.import) =
  let (Import (module_name, item_name, kind)) = import.it in
  let module_ = Wasm.Utf8.encode module_name and field = Wasm.Utf8.encode item_name in
  let source =
    if module_ = "spectest" then Ok Spectest
    else
      match Names.find module_ modules.registry with
      | Lifted instance -> Ok (Exported instance.number)
      | NotRun _ -> Error "imports of a module not run"
  in
  match (source, declared kind) with
  | Ok source, Ok () -> Ok { module_; name = field; source }
  | Error reason, _ | _, Error reason -> Error reason

let resolve_instance modules line (definition : Wasm.Ast.module_) =
  let resolved = List.map (resolve modules) definition.it.imports in
  match List.find_map (function Error reason -> Some reason | Ok _ -> None) resolved with
  | Some reason -> Error (Refused reason)
  | None ->
      Ok
        {
          number = Numbers.cardinal modules.instances;
          line;
          definition;
          imports = List.filter_map Result.to_option resolved;
        }

let lift modules line path =
  try
    let definition = Wasm.Decode.decode (Filename.basename path) (In_channel.with_open_bin path In_channel.input_all) in
    match WellFormedness.check (WebAssembly.Module.lift (TypeArena.create IR_Memory.extension) definition) with
    | [] -> resolve_instance modules line definition
    | errors -> Error (Failed (String.concat "; " (List.take 3 errors)))
  with
  | WebAssembly.Types.Unsupported feature -> Error (Refused feature)
  | error -> Error (Failed (Printexc.to_string error))

let disposition lifted step = match lifted with Ok instance -> Run (step instance) | Error not_run -> not_run

(** A module that is not run may have imported and changed any registered instance, so their later results, and those of
    instances importing from them, are unknown. *)
let added modules lifted =
  match lifted with
  | Ok instance -> { modules with instances = Numbers.add instance.number instance modules.instances }
  | Error _ -> { modules with unknown = NumberSet.union modules.unknown modules.registered }

let instantiate modules line binary name =
  let lifted = match binary with Some path -> lift modules line path | None -> Error (Refused "text modules") in
  let current = match lifted with Ok instance -> Lifted instance | Error not_run -> NotRun not_run in
  let modules = added modules lifted in
  ( disposition lifted (fun instance -> Instantiate instance),
    {
      modules with
      current = Some current;
      named = (match name with Some name -> Names.add name current modules.named | None -> modules.named);
    } )

let define modules binary name =
  ( Skipped "module definitions",
    match name with
    | Some name -> { modules with definitions = Names.add name binary modules.definitions }
    | None -> modules )

let uninstantiable modules line binary =
  let lifted = lift modules line binary in
  (disposition lifted (fun instance -> Uninstantiable instance), added modules lifted)

let named modules name =
  match name with Some name -> Names.find name modules.named | None -> Option.get modules.current

let host number = Int64.succ number

(** A function reference argument is always null. *)
let argument kind value =
  match (kind, value) with
  | _, Bits (value_type, bits) -> Number (value_type, bits)
  | External, External number -> Host (host number)
  | External, _ -> Host 0L
  | _ -> Null

let unsupported values =
  List.find_map (function Unsupported value_type -> Some (value_type ^ " values") | _ -> None) values

let type_of (definition : Wasm.Ast.module_) index =
  let imported =
    List.filter_map
      (fun (import : Wasm.Ast.import) ->
        match import.it with Import (_, _, ExternFuncT (Idx type_index)) -> Some (Int32.to_int type_index) | _ -> None)
      definition.it.imports
  in
  if index < List.length imported then List.nth imported index
  else
    let (Func (type_index, _, _)) = (List.nth definition.it.funcs (index - List.length imported)).it in
    Int32.to_int type_index.it

let invoke_export instance line action expectation function_index =
  let takes, returns = signature instance.definition (type_of instance.definition function_index) in
  let kinds = List.map (fun value_type -> Option.get (kind value_type)) in
  let signature = { parameters = kinds takes; results = kinds returns } in
  Run
    (Invoke
       {
         line;
         instance = instance.number;
         function_ = action.field;
         arguments = List.map2 argument signature.parameters action.arguments;
         signature;
         expectation;
       })

let export_index instance field index_of =
  Option.get
    (List.find_map
       (fun (export : Wasm.Ast.export) ->
         let (Export (name, exported)) = export.it in
         if Wasm.Utf8.encode name = field then index_of exported else None)
       instance.definition.it.exports)

let invoke instance line (action : action) expectation =
  invoke_export instance line action expectation
    (export_index instance action.field (fun exported ->
         match exported.it with FuncX index -> Some (Int32.to_int index.it) | _ -> None))

let read instance line action expectation =
  let globals =
    List.filter_map
      (fun (import : Wasm.Ast.import) ->
        match import.it with Import (_, _, ExternGlobalT global) -> Some global | _ -> None)
      instance.definition.it.imports
    @ List.map
        (fun (global : Wasm.Ast.global) ->
          let (Global (global_type, _)) = global.it in
          global_type)
        instance.definition.it.globals
  in
  let index =
    export_index instance action.field (fun exported ->
        match exported.it with GlobalX index -> Some (Int32.to_int index.it) | _ -> None)
  in
  let (GlobalT (_, value_type)) = List.nth globals index in
  Run
    (Get
       {
         line;
         instance = instance.number;
         global = action.field;
         kind = Option.get (kind value_type);
         expected = Expectation.values expectation;
       })

let act modules line (action : action) expectation =
  match named modules action.module_ with
  | NotRun (Refused feature) -> Skipped ("refused " ^ feature)
  | NotRun _ -> Failed "its module failed to lift"
  | Lifted instance when is_unknown modules instance -> Skipped "registered for a module not run"
  | Lifted instance -> (
      match (unsupported (action.arguments @ Expectation.values expectation), action.kind) with
      | Some reason, _ -> Skipped reason
      | None, Get -> read instance line action expectation
      | None, Invoke -> invoke instance line action expectation)

let register modules name alias =
  let registered = named modules name in
  ( Skipped "register",
    {
      modules with
      registry = Names.add alias registered modules.registry;
      registered =
        (match registered with
        | Lifted instance -> NumberSet.add instance.number modules.registered
        | NotRun _ -> modules.registered);
    } )

let dispose modules (line, command) =
  let disposition, modules =
    match command with
    | Module { binary; name } -> instantiate modules line binary name
    | ModuleInstance { definition; instance } ->
        instantiate modules line (Some (Names.find definition modules.definitions)) instance
    | ModuleDefinition { binary; name } -> define modules binary name
    | Register { name; registered } -> register modules name registered
    | AssertUninstantiable binary -> uninstantiable modules line binary
    | AssertTrap (_, "indirect call type mismatch") -> (Skipped "undefined: indirect call type mismatch", modules)
    | AssertReturn (action, expected) -> (act modules line action (Returns expected), modules)
    | AssertTrap (action, _) -> (act modules line action Traps, modules)
    | Action action -> (act modules line action Finishes, modules)
    | Unplanned kind -> (Skipped kind, modules)
  in
  (modules, (line, disposition))

let create (test : test) =
  let modules =
    {
      current = None;
      named = Names.empty;
      definitions = Names.empty;
      registry = Names.empty;
      instances = Numbers.empty;
      registered = NumberSet.empty;
      unknown = NumberSet.empty;
    }
  in
  { test = test.name; commands = snd (List.fold_left_map dispose modules test.commands) }
