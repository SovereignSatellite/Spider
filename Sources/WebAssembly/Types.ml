open IR
open Wasm.Source

exception Unsupported of string
exception Invalid of string

let unsupported instruction =
  raise (Unsupported (String.trim (Wasm.Sexpr.to_string 80 (Wasm.Arrange.instr (instruction @@ no_region)))))

type signature = { parameters : Type.t list; results : Type.t list }

let index variable = Int32.to_int variable.it

let used (use : Wasm.Types.typeuse) =
  match use with Idx type_index -> Int32.to_int type_index | Rec _ | Def _ -> invalid_arg "Decoded types are indices."

let composites types =
  Array.of_list
    (List.concat_map
       (fun (group : Wasm.Ast.type_) ->
         let (RecT members) = group.it in
         List.map (fun (Wasm.Types.SubT (_, _, composite)) -> composite) members)
       types)

let unsupported_value value_type = raise (Unsupported (Wasm.Types.string_of_valtype value_type ^ " values"))

(** A reference lifts as the nullable reference to the top of its hierarchy, whichever type it names in it. *)
let value (composites : Wasm.Types.comptype array) function_reference exception_reference host_reference
    (value_type : Wasm.Types.valtype) =
  match value_type with
  | NumT I32T -> Type.i32
  | NumT I64T -> Type.i64
  | NumT F32T -> Type.f32
  | NumT F64T -> Type.f64
  | RefT (_, (FuncHT | NoFuncHT)) -> function_reference
  | RefT (_, UseHT use) -> (
      match composites.(used use) with
      | FuncT _ -> function_reference
      | StructT _ | ArrayT _ -> unsupported_value value_type)
  | RefT (_, (ExnHT | NoExnHT)) -> exception_reference
  | RefT (_, (ExternHT | NoExternHT)) -> host_reference
  | other -> unsupported_value other

let signatures_of value composites =
  Array.map
    (fun (composite : Wasm.Types.comptype) ->
      lazy
        (match composite with
        | FuncT (parameters, results) -> { parameters = List.map value parameters; results = List.map value results }
        | StructT _ -> raise (Unsupported "Struct types")
        | ArrayT _ -> raise (Unsupported "Array types")))
    composites

type signatures = {
  composites : Wasm.Types.comptype array;
  types : signature Lazy.t array;
  functions : signature array;
  globals : Type.t array;
  mutable_ : bool array;
  tags : Type.t list array;
  tables : Type.t array;
  elements : Type.t array;
  reference : Type.t;
  exception_ : Type.t;
  host : Type.t;
}

let value_type signatures value_type =
  value signatures.composites signatures.reference signatures.exception_ signatures.host value_type

let imported select (definition : Wasm.Ast.module_) =
  List.filter_map
    (fun (import : Wasm.Ast.import) ->
      let (Import (_, _, kind)) = import.it in
      select kind)
    definition.it.imports

let imported_function (kind : Wasm.Types.externtype) = match kind with ExternFuncT use -> Some (used use) | _ -> None
let imported_global (kind : Wasm.Types.externtype) = match kind with ExternGlobalT global -> Some global | _ -> None
let imported_tag (kind : Wasm.Types.externtype) = match kind with ExternTagT (TagT use) -> Some (used use) | _ -> None
let imported_memory (kind : Wasm.Types.externtype) = match kind with ExternMemoryT memory -> Some memory | _ -> None
let imported_table (kind : Wasm.Types.externtype) = match kind with ExternTableT table -> Some table | _ -> None

let table_types definition =
  imported imported_table definition
  @ List.map
      (fun (table : Wasm.Ast.table) ->
        let (Table (table_type, _)) = table.it in
        table_type)
      definition.it.tables

let tag_type (tag : Wasm.Ast.tag) =
  let (Tag (TagT use)) = tag.it in
  used use

let signatures function_reference exception_reference host_reference (definition : Wasm.Ast.module_) =
  let composites = composites definition.it.types in
  let value = value composites function_reference exception_reference host_reference in
  let types = signatures_of value composites in
  let globals =
    imported imported_global definition
    @ List.map
        (fun (declared : Wasm.Ast.global) ->
          let (Global (global_type, _)) = declared.it in
          global_type)
        definition.it.globals
  in
  {
    composites;
    types;
    functions =
      Array.of_list
        (List.map
           (fun type_index -> Lazy.force types.(type_index))
           (imported imported_function definition
           @ List.map
               (fun (code : Wasm.Ast.func) ->
                 let (Func (type_index, _, _)) = code.it in
                 index type_index)
               definition.it.funcs));
    globals = Array.of_list (List.map (fun (Wasm.Types.GlobalT (_, content)) -> value content) globals);
    mutable_ = Array.of_list (List.map (fun (Wasm.Types.GlobalT (mutability, _)) -> mutability = Var) globals);
    tags =
      Array.of_list
        (List.map
           (fun type_index -> (Lazy.force types.(type_index)).parameters)
           (imported imported_tag definition @ List.map tag_type definition.it.tags));
    tables =
      Array.of_list
        (List.map (fun (Wasm.Types.TableT (_, _, element)) -> value (RefT element)) (table_types definition));
    elements =
      Array.of_list
        (List.map
           (fun (element : Wasm.Ast.elem) ->
             let (Elem (element_type, _, _)) = element.it in
             value (RefT element_type))
           definition.it.elems);
    reference = function_reference;
    exception_ = exception_reference;
    host = host_reference;
  }
