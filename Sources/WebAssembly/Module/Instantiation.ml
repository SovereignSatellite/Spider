open IR
open Definitions
open Wasm.Source
open Types
open Entities
open Traps
open Accesses
open Calls

let most_pages = 65535

let maximum (limits : Wasm.Types.limits) most =
  match limits.max with Some maximum -> min (Int64.to_int maximum) most | None -> most

let most_elements limits = i32 (maximum limits 0xFFFF_FFFF)

let lift_onto context state stack instruction =
  let count = List.length (Stack.change context.signatures instruction).parameters in
  let results, _ = Operations.lift context state instruction (List.rev (List.take count stack)) in
  List.rev_append results (List.drop count stack)

let constant context state expression =
  List.hd (List.fold_left (fun stack instruction -> lift_onto context state stack instruction.it) [] expression.it)

let items context state (element : Wasm.Ast.elem) =
  let (Elem (_, items, _)) = element.it in
  List.map (constant context state) items

let initialize context state field segment offset length =
  let start = constant context state offset in
  snd (copy context state field segment [ start; i32 0; length ])

let place_elements context state index (element : Wasm.Ast.elem) =
  let dropped state = snd (Tables.drop context state index) in
  let (Elem (_, _, mode)) = element.it in
  match mode.it with
  | Active (table, offset) ->
      let length = make ArraySize [| State.contents context state (Element index) Reads |] in
      dropped (initialize context state (Tables.table table) (Element index) offset length)
  | Declarative -> dropped state
  | Passive -> state

let place_data context state index (segment : Wasm.Ast.data) =
  let (Data (bytes, mode)) = segment.it in
  match mode.it with
  | Active (memory, offset) ->
      let length = i32 (String.length bytes) in
      let state = initialize context state (Memories.memory memory) (Data index) offset length in
      snd (Memories.drop context state index)
  | Passive | Declarative -> state

let import_node (import : Wasm.Ast.import) import_type =
  let (Import (module_name, item_name, _)) = import.it in
  make (Import { import_type; namespace = Wasm.Utf8.encode module_name; identifier = Wasm.Utf8.encode item_name }) [||]

let received context imports fields =
  List.map2
    (fun import field ->
      let held = Context.field_type context field in
      ( field,
        match field with
        | Function _ -> result (BoxGet.create context.types (import_node import (context.boxes held)))
        | Memory _ | Data _ | Table _ | Element _ | Global _ | Tag _ -> import_node import held ))
    imports fields

let boxed context field contents = (field, make (BoxNew (Context.field_type context field)) [| contents |])

(** Globals, tables and element segments start out zero or empty: environments hold their boxes, and their initializers,
    which may reference functions, run once the environments exist. *)
let declared context (definition : Wasm.Ast.module_) (imported : Dependencies.imported) =
  let memories =
    List.mapi
      (fun offset (memory : Wasm.Ast.memory) ->
        let (Memory (MemoryT (_, limits))) = memory.it in
        let field = Field.Memory (imported.memories + offset) in
        let bytes = allocate context (i32 (Int64.to_int limits.min * 65536)) (make (I8Constant 0) [||]) in
        boxed context field (limited context field bytes (i32 (maximum limits most_pages))))
      definition.it.memories
  in
  let data =
    List.mapi
      (fun index (segment : Wasm.Ast.data) ->
        let (Data (bytes, _)) = segment.it in
        boxed context (Data index) (result (ArrayOf.create context.types Type.i8 bytes)))
      definition.it.datas
  in
  let tables =
    List.mapi
      (fun offset (table : Wasm.Ast.table) ->
        let (Table (TableT (_, limits, _), _)) = table.it in
        let index = imported.tables + offset in
        let unset = Tables.references context context.signatures.tables.(index) [] in
        boxed context (Table index) (limited context (Table index) unset (most_elements limits)))
      definition.it.tables
  in
  let elements =
    List.mapi
      (fun index _ -> boxed context (Element index) (Tables.references context context.signatures.elements.(index) []))
      definition.it.elems
  in
  let globals =
    List.init
      (Array.length context.signatures.globals - imported.globals)
      (fun offset ->
        let index = imported.globals + offset in
        boxed context (Global index) (zero context.signatures.globals.(index)))
  in
  let tags =
    List.init
      (Array.length context.signatures.tags - imported.tags)
      (fun offset -> (Field.Tag (imported.tags + offset), make (BoxNew context.tag_box) [| zero Type.unit |]))
  in
  memories @ data @ tables @ elements @ globals @ tags

let export_value context settled (export : Wasm.Ast.export) =
  let (Export (name, _)) = export.it in
  let box =
    match Dependencies.exported export with
    | Function index ->
        let value = function_value context settled index in
        make (BoxNew (context.boxes (Link.type_ context.types value))) [| value |]
    | (Memory _ | Data _ | Table _ | Element _ | Global _ | Tag _) as field -> State.box settled field
  in
  make (Export (Wasm.Utf8.encode name)) [| box |]

let indexed step state items = Seq.fold_lefti step state (List.to_seq items)

let instantiate context definition captures environment_types start =
  let imported, fields = Dependencies.imports definition in
  let received = received context definition.it.imports fields in
  let declared = declared context definition imported in
  let entities =
    List.fold_left (fun entities (field, link) -> Fields.add field link entities) Fields.empty (received @ declared)
  in
  let values, written = Environments.build context captures environment_types entities in
  let installed =
    State.of_map context
      (fence start (Array.of_list written))
      values
      (Array.of_list
         (List.filter
            (fun (field : Field.t) ->
              match field with
              | Function _ -> false
              | Memory _ | Data _ | Table _ | Element _ | Global _ | Tag _ -> true)
            (List.map fst (Fields.bindings values))))
  in
  let globals =
    indexed
      (fun state offset (declared : Wasm.Ast.global) ->
        let (Global (_, initial)) = declared.it in
        let value = constant context state initial in
        set context state (Global (imported.globals + offset)) value)
      installed definition.it.globals
  in
  let tables =
    indexed
      (fun state offset (table : Wasm.Ast.table) ->
        let (Table (TableT (_, limits, _), initial)) = table.it in
        let field = Field.Table (imported.tables + offset) in
        let elements = allocate context (i32 (Int64.to_int limits.min)) (constant context state initial) in
        set context state field (limited context field elements (most_elements limits)))
      globals definition.it.tables
  in
  let populated =
    indexed
      (fun state index element ->
        let element_type = context.signatures.elements.(index) in
        set context state (Element index) (Tables.references context element_type (items context state element)))
      tables definition.it.elems
  in
  let placed = indexed (place_elements context) populated definition.it.elems in
  let placed = indexed (place_data context) placed definition.it.datas in
  (* A call waits only for the fields some callee writes, so the start waits for the rest it may read here. *)
  let started =
    match definition.it.start with
    | Some start ->
        let (Start function_index) = start.it in
        snd (call context (State.settle placed) (index function_index) [])
    | None -> placed
  in
  let settled = State.settle started in
  let exports = List.map (export_value context settled) definition.it.exports in
  [| fence settled.token (Array.of_list exports) |]
