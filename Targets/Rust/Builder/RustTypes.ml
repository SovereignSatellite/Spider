open IR
open Allocator
open Emission
open Tree
open RustExtension

type t = {
  types : x TypeArena.t;
  layouts : x Layouts.t;
  helpers : RuntimeLibrary.NeededHelpers.t;
  leaf_types : (Type.t, RustType.t) Hashtbl.t;
  box_names : (Name.t * RustType.t) list BoxNames.t;
}

let leaf_field leaf : Name.t = Numbered ("leaf_", leaf)

let create types layouts helpers =
  { types; layouts; helpers; leaf_types = Hashtbl.create 64; box_names = BoxNames.create () }

let rec leaf_type rust leaf =
  match Hashtbl.find_opt rust.leaf_types leaf with
  | Some known -> known
  | None ->
      let created = create_leaf_type rust leaf in
      Hashtbl.replace rust.leaf_types leaf created;
      created

and create_leaf_type rust leaf =
  match TypeArena.definition rust.types leaf with
  | Structural (Integer width) -> Named (Numbered ("u", width))
  | Structural (Number width) -> Named (Numbered ("f", width))
  | Structural (Function (parameters, results)) -> function_type rust parameters results
  | Structural (Array element) -> array_type rust element
  | Structural Opaque -> Pointer (Tuple [])
  | Box _ -> Pointer (Named (BoxNames.name rust.box_names leaf (definition_of rust)))
  | Structural (Sum _) -> decomposed rust leaf
  | Extension (IR_Rust.Signed width) -> Named (Numbered ("i", width))
  | Extension IR_Rust.Usize -> Named (Text "usize")
  | Extension (IR_Rust.Outcome results) ->
      let outcome = result_type rust results in
      Generic (Text "Result", [ outcome; Pointer (leaf_type rust Type.i8) ])
  | Extension _ -> invalid_arg ("Rust's extension defines no type such as " ^ TypeArena.describe rust.types leaf ^ ".")

and decomposed rust value_type =
  invalid_arg (TypeArena.describe rust.types value_type ^ " is decomposed, so it is never one leaf.")

and function_type rust parameters results =
  let parameter_types = List.map (leaf_type rust) (Layouts.leaves rust.layouts parameters) in
  Function (parameter_types, result_type rust results)

and result_type rust results =
  match List.map (leaf_type rust) (Layouts.leaves rust.layouts results) with [ leaf ] -> leaf | leaves -> Tuple leaves

and array_type rust element = Pointer (Slice (element_type rust element))

and element_type rust element =
  match Layouts.leaves_of rust.layouts element with [| leaf |] -> leaf_type rust leaf | _ -> decomposed rust element

and definition_of rust box =
  List.mapi
    (fun index leaf -> (leaf_field index, leaf_type rust leaf))
    (Array.to_list (Layouts.leaves_of rust.layouts (TypeArena.contents rust.types box)))

let path name : Expression.t = Name (Text name)
let null_pointer : Expression.t = Call (path "core::ptr::null_mut", [])

(** Rust's function pointers are never null, so a function type's zero is a function no defined program calls. *)
let unreachable rust leaf : Expression.t =
  RuntimeLibrary.NeededHelpers.need rust.helpers "Unreachable";
  let any = RustType.Function ([], Tuple []) in
  Transmute (Cast (Name (Helper "Unreachable"), any), leaf_type rust leaf)

let zero rust leaf : Expression.t =
  match TypeArena.definition rust.types leaf with
  | Structural (Number width) -> Integer (0L, if width = 32 then "f32" else "f64")
  | Structural (Function _) -> unreachable rust leaf
  | Structural (Array _) -> Call (path "core::ptr::slice_from_raw_parts_mut", [ null_pointer; Integer (0L, "") ])
  | Structural Opaque | Box _ -> null_pointer
  | Extension (IR_Rust.Outcome _) -> Call (path "Err", [ null_pointer ])
  | Structural (Integer _ | Sum _) | Extension _ -> Integer (0L, "")

let element_pointer rust array : RustType.t = Pointer (element_type rust (TypeArena.element rust.types array))
let box_name rust box = BoxNames.name rust.box_names box (definition_of rust)

let definitions rust =
  List.map (fun (name, definition) -> Item.Type (name, definition)) (BoxNames.definitions rust.box_names)
