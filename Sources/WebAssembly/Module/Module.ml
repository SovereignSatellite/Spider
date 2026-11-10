open IR
open Wasm.Source
open Types
open Entities

let memories definition =
  imported imported_memory definition
  @ List.map
      (fun (memory : Wasm.Ast.memory) ->
        let (Memory memory_type) = memory.it in
        memory_type)
      definition.it.memories

let refuse definition =
  let memories = memories definition in
  let tables = table_types definition in
  let refuse_if applies feature = if applies then raise (Unsupported feature) in
  refuse_if (List.exists (fun (Wasm.Types.MemoryT (address, _)) -> address = I64AT) memories) "64-bit memories";
  refuse_if
    (List.exists (fun (Wasm.Types.MemoryT (_, limits)) -> limits.min > Int64.of_int Instantiation.most_pages) memories)
    "memories of 65536 pages";
  refuse_if (List.exists (fun (Wasm.Types.TableT (address, _, _)) -> address = I64AT) tables) "64-bit tables"

let context types definition captures =
  let boxes = Hashtbl.create 16 in
  let box_of contents =
    match Hashtbl.find_opt boxes contents with
    | Some existing -> existing
    | None ->
        let created = TypeArena.create_box types (fun _ -> contents) in
        Hashtbl.add boxes contents created;
        created
  in
  let structure members = TypeArena.intern types (Sum [ members ]) in
  let erased = box_of Type.unit in
  let erased_function = Context.erased_function types in
  let reference = TypeArena.intern types (Sum [ []; [ erased_function; erased ] ]) in
  let signatures_with exception_box =
    signatures reference
      (TypeArena.intern types (Sum [ []; [ exception_box ] ]))
      (TypeArena.intern types (Sum [ []; [ Type.opaque ] ]))
      definition
  in
  let tag_box = TypeArena.create_box types (fun _ -> Type.unit) in
  (* A tag's parameters may be exception references, so the exception box's contents hold the box itself. *)
  let exception_box =
    TypeArena.create_box types (fun exception_box ->
        TypeArena.intern types
          (Sum
             (List.map
                (fun parameters -> tag_box :: parameters)
                (distinct (Array.to_list (signatures_with exception_box).tags)))))
  in
  let signatures = signatures_with exception_box in
  let imported_count = List.length (imported imported_function definition) in
  let declare signature =
    Region.create types ((erased :: signature.parameters) @ [ Type.unit ]) (signature.results @ [ Type.unit ])
  in
  let bodies =
    Array.map declare
      (Array.sub signatures.functions imported_count (Array.length signatures.functions - imported_count))
  in
  let effects = Dependencies.effects signatures definition in
  let context =
    {
      types;
      signatures;
      erased;
      imported_functions = imported_count;
      bodies;
      tag_box;
      exception_box;
      boxes = box_of;
      effects;
    }
  in
  (context, List.map (fun fields -> box_of (structure (List.map (Context.field_type context) fields))) captures)

let rejecting read =
  try read ()
  with Wasm.Decode.Code (at, message) | Wasm.Valid.Invalid (at, message) ->
    raise (Invalid (string_of_region at ^ ": " ^ message))

let lifted types definition : 'x CompilationUnit.t * 'x context =
  rejecting (fun () -> ignore (Wasm.Valid.check_module definition));
  refuse definition;
  let captures = Environments.captures definition in
  let context, environments = context types definition captures in
  List.iteri
    (fun offset (code, (environment, fields)) ->
      let body = context.bodies.(offset) in
      Region.define body (Lifting.lift context (context.imported_functions + offset) fields environment code body))
    (List.combine definition.it.funcs (List.combine environments captures));
  let root =
    Region.create types [ Type.unit ] [ Type.unit ] ~build:(fun arguments ->
        Instantiation.instantiate context definition captures environments (Region.parameter arguments 0))
  in
  ({ types; root }, context)

let lift types definition = fst (lifted types definition)

let lift_labelled types binary =
  let definition, customs = rejecting (fun () -> Wasm.Decode.decode_with_custom "module" binary) in
  let compilation_unit, context = lifted types definition in
  let labels = RegionTable.create (Array.length context.bodies) in
  let label = Names.label definition binary customs context.imported_functions in
  Array.iteri (fun offset body -> RegionTable.replace labels body (label offset)) context.bodies;
  (compilation_unit, labels)
