(* Luau: Luau is typed. *)
open IR
open Allocator
open Emission
open Tree
open LuauExtension

type t = {
  types : x TypeArena.t;
  layouts : x Layouts.t;
  helpers : RuntimeLibrary.NeededHelpers.t;
  leaf_types : (Type.t, LuauType.t) Hashtbl.t;
  box_names : LuauType.t BoxNames.t;
}

let create types layouts helpers =
  { types; layouts; helpers; leaf_types = Hashtbl.create 64; box_names = BoxNames.create () }

let rec leaf_type luau leaf =
  match Hashtbl.find_opt luau.leaf_types leaf with
  | Some known -> known
  | None ->
      let created = create_leaf_type luau leaf in
      Hashtbl.replace luau.leaf_types leaf created;
      created

and create_leaf_type luau leaf =
  match TypeArena.definition luau.types leaf with
  | Structural (Integer _ | Number _) -> Named (Text "number")
  | Structural (Function (parameters, results)) -> function_type luau parameters results
  | Structural (Array element) -> array_type luau element
  | Structural Opaque -> Named (Text "unknown")
  | Box _ -> Named (BoxNames.name luau.box_names leaf (definition_of luau))
  | Structural (Sum _) -> decomposed luau leaf
  | Extension IR_Luau.Thrown -> Named (Text "any")
  | Extension _ -> invalid_arg ("Luau's extension defines no type such as " ^ TypeArena.describe luau.types leaf ^ ".")

and decomposed luau value_type =
  invalid_arg (TypeArena.describe luau.types value_type ^ " is decomposed, so it is never one leaf.")

and function_type luau parameters results =
  let parameter_types = List.map (leaf_type luau) (Layouts.leaves luau.layouts parameters) in
  Function (parameter_types, List.map (leaf_type luau) (Layouts.leaves luau.layouts results))

and array_type luau element =
  if element = Type.i8 then Named (Text "buffer")
  else begin
    RuntimeLibrary.NeededHelpers.need luau.helpers "Array";
    Generic (Text "Array", [ element_type luau element ])
  end

and element_type luau element =
  match Layouts.leaves_of luau.layouts element with [| leaf |] -> leaf_type luau leaf | _ -> decomposed luau element

and definition_of luau box =
  match Array.to_list (Layouts.leaves_of luau.layouts (TypeArena.contents luau.types box)) with
  | [] -> Table []
  | leaves -> Sequence (LuauType.union (List.map (leaf_type luau) leaves))

let tag luau leaf =
  match TypeArena.definition luau.types leaf with
  | Structural (Integer _ | Number _) -> "number"
  | Structural (Function _) -> "function"
  | Structural (Array element) when element = Type.i8 -> "buffer"
  | Structural Opaque -> "unknown"
  | Structural (Array _ | Sum _) | Box _ | Extension _ -> "table"

let definitions luau =
  List.map (fun (name, definition) -> Statement.Type (name, definition)) (BoxNames.definitions luau.box_names)
