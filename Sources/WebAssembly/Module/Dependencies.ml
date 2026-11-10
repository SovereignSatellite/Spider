open Wasm.Source
open Types
open Entities

let rec fold visit state code =
  List.fold_left
    (fun state (instruction : Wasm.Ast.instr) ->
      let state = visit state instruction.it in
      match instruction.it with
      | Block (_, body) | Loop (_, body) | TryTable (_, _, body) -> fold visit state body
      | If (_, taken, otherwise) -> fold visit (fold visit state taken) otherwise
      | _ -> state)
    state code

let body (code : Wasm.Ast.func) =
  let (Func (_, _, body)) = code.it in
  body

let exported (export : Wasm.Ast.export) : Field.t =
  let (Export (_, exported)) = export.it in
  match exported.it with
  | FuncX function_index -> Function (index function_index)
  | TableX table -> Table (index table)
  | MemoryX memory -> Memory (index memory)
  | TagX tag -> Tag (index tag)
  | GlobalX global -> Global (index global)

let including accesses field access =
  match Fields.find_opt field accesses with
  | Some held when held >= access -> accesses
  | _ -> Fields.add field access accesses

type imported = { functions : int; memories : int; tables : int; globals : int; tags : int }

let imports (definition : Wasm.Ast.module_) =
  List.fold_left_map
    (fun counts (import : Wasm.Ast.import) : (imported * Field.t) ->
      let (Import (_, _, kind)) = import.it in
      match kind with
      | ExternFuncT _ -> ({ counts with functions = counts.functions + 1 }, Function counts.functions)
      | ExternMemoryT _ -> ({ counts with memories = counts.memories + 1 }, Memory counts.memories)
      | ExternTableT _ -> ({ counts with tables = counts.tables + 1 }, Table counts.tables)
      | ExternGlobalT _ -> ({ counts with globals = counts.globals + 1 }, Global counts.globals)
      | ExternTagT _ -> ({ counts with tags = counts.tags + 1 }, Tag counts.tags))
    { functions = 0; memories = 0; tables = 0; globals = 0; tags = 0 }
    definition.it.imports

let host_references (signatures : signatures) interface_fields =
  List.exists
    (fun (field : Field.t) ->
      match field with
      | Function index ->
          let signature = signatures.functions.(index) in
          List.mem signatures.reference (signature.parameters @ signature.results)
      | Global index -> signatures.globals.(index) = signatures.reference
      | Table index -> signatures.tables.(index) = signatures.reference
      | Memory _ | Data _ | Element _ | Tag _ -> false)
    interface_fields

let referenced (definition : Wasm.Ast.module_) =
  let visit found (instruction : Wasm.Ast.instr') =
    match instruction with RefFunc function_index -> index function_index :: found | _ -> found
  in
  let expression found constant = fold visit found constant.it in
  let initial found (declared : Wasm.Ast.global) =
    let (Global (_, initial)) = declared.it in
    expression found initial
  in
  let table found (table : Wasm.Ast.table) =
    let (Table (_, initial)) = table.it in
    expression found initial
  in
  let items found (element : Wasm.Ast.elem) =
    let (Elem (_, items, _)) = element.it in
    List.fold_left expression found items
  in
  let found = List.fold_left (fun found code -> fold visit found (body code)) [] definition.it.funcs in
  let found = List.fold_left initial found definition.it.globals in
  let found = List.fold_left table found definition.it.tables in
  Array.of_list (distinct (List.rev (List.fold_left items found definition.it.elems)))

let accessed code =
  fold
    (fun accesses instruction ->
      List.fold_left
        (fun accesses (field, access) -> including accesses field access)
        accesses (Operations.accesses instruction))
    Fields.empty (body code)

let calls (signatures : signatures) code =
  let visit callees (instruction : Wasm.Ast.instr') =
    match instruction with
    | Call called -> Callee.Function (index called) :: callees
    | CallIndirect (_, type_index) | CallRef type_index ->
        Indirect (Lazy.force signatures.types.(index type_index)) :: callees
    | _ -> callees
  in
  distinct (List.rev (fold visit [] (body code)))

let shared signatures interface_fields =
  List.fold_left
    (fun accesses (field : Field.t) ->
      match field with
      | Function _ | Tag _ -> accesses
      | Global index when not signatures.mutable_.(index) -> accesses
      | Global _ -> Fields.add field Access.Writes accesses
      | Memory _ | Data _ | Table _ | Element _ -> Fields.add field Access.Replaces accesses)
    Fields.empty interface_fields

let catches (definition : Wasm.Ast.module_) =
  let visit found (instruction : Wasm.Ast.instr') = found || match instruction with TryTable _ -> true | _ -> false in
  List.exists (fun code -> fold visit false (body code)) definition.it.funcs

let indirect_callees (signatures : signatures) referenced signature =
  List.filter_map
    (fun index -> if signatures.functions.(index) = signature then Some (Callee.Function index) else None)
    (Array.to_list referenced)

let host_callees exports referenced host =
  List.filter_map
    (fun (field : Field.t) ->
      match field with
      | Function index -> Some (Callee.Function index)
      | Memory _ | Data _ | Table _ | Element _ | Global _ | Tag _ -> None)
    exports
  @ if host then List.map (fun index -> Callee.Function index) (Array.to_list referenced) else []

(** Each callee reachable from a function, with what it accesses itself and who calls it. *)
let discovered function_count own =
  let effects = Hashtbl.create 64 in
  let callers = Hashtbl.create 64 in
  let rec discover (callee : Callee.t) =
    if not (Hashtbl.mem callers callee) then begin
      Hashtbl.replace callers callee (Dynarray.create ());
      let accesses, callees = own callee in
      Hashtbl.replace effects callee accesses;
      List.iter
        (fun called ->
          discover called;
          Dynarray.add_last (Hashtbl.find callers called) callee)
        callees
    end
  in
  for index = 0 to function_count - 1 do
    discover (Function index)
  done;
  (effects, callers)

let written effects =
  Hashtbl.fold
    (fun _ accesses written ->
      Fields.fold
        (fun field access written -> if access >= Access.Writes then Fields.add field () written else written)
        accesses written)
    effects Fields.empty

(** Where a module catches, a call may throw to a clause that reads what was written: it waits for every write before
    it, and every write after it waits for it. *)
let keep_ordered effects catches =
  let written = written effects in
  List.iter
    (fun callee ->
      let accessed = Fields.filter (fun field _ -> Fields.mem field written) (Hashtbl.find effects callee) in
      Hashtbl.replace effects callee
        (if catches then Fields.fold (fun field () accesses -> including accesses field Access.Reads) written accessed
         else accessed))
    (List.of_seq (Hashtbl.to_seq_keys effects))

let propagate effects callers =
  let changed = Queue.of_seq (Hashtbl.to_seq_keys effects) in
  while not (Queue.is_empty changed) do
    let callee = Queue.pop changed in
    Dynarray.iter
      (fun caller ->
        let held = Hashtbl.find effects caller in
        let merged =
          Fields.fold (fun field access merged -> including merged field access) (Hashtbl.find effects callee) held
        in
        if merged != held then begin
          Hashtbl.replace effects caller merged;
          Queue.add caller changed
        end)
      (Hashtbl.find callers callee)
  done

let effects signatures definition =
  let imported_count = List.length (imported imported_function definition) in
  let codes = Array.of_list definition.it.funcs in
  let exports = List.map exported definition.it.exports in
  let interface_fields = snd (imports definition) @ exports in
  let host = host_references signatures interface_fields in
  let referenced = referenced definition in
  let own (callee : Callee.t) : Access.t Fields.t * Callee.t list =
    match callee with
    | Function index when index < imported_count -> (Fields.empty, [ Host ])
    | Function index ->
        let code = codes.(index - imported_count) in
        (accessed code, calls signatures code)
    | Indirect _ when host -> (Fields.empty, [ Host ])
    | Indirect signature -> (Fields.empty, indirect_callees signatures referenced signature)
    | Host -> (shared signatures interface_fields, host_callees exports referenced host)
  in
  let effects, callers = discovered (Array.length signatures.functions) own in
  keep_ordered effects (catches definition);
  propagate effects callers;
  effects
