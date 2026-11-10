open IR
open Definitions
open Types
open Entities

let captures definition =
  let imported_count = List.length (imported imported_function definition) in
  List.mapi
    (fun offset code ->
      List.map fst (Fields.bindings (Fields.remove (Function (imported_count + offset)) (Dependencies.accessed code))))
    definition.it.funcs

(** A function whose environment is [Building] is captured empty and written once its box exists. *)
type 'x environment = Unbuilt | Building | Built of 'x link

let build context captures environments entities =
  let captures = Array.of_list captures in
  let environments = Array.of_list environments in
  let empty = make (BoxNew context.erased) [| zero Type.unit |] in
  let built = Array.make (Array.length captures) Unbuilt in
  let incomplete = Dynarray.create () in
  let erase box = make (BoxRetype context.erased) [| box |] in
  let held_by offset environment =
    let values =
      List.map
        (fun (field : Field.t) ->
          match field with
          | Function index when index >= context.imported_functions ->
              Option.map erase (environment (index - context.imported_functions))
          | Function _ | Memory _ | Data _ | Table _ | Element _ | Global _ | Tag _ -> Some (Fields.find field entities))
        captures.(offset)
    in
    ( make
        (SumNew { sum_type = Context.contents context environments.(offset); variant = 0 })
        (Array.of_list (List.map (Option.value ~default:empty) values)),
      List.exists Option.is_none values )
  in
  let rec environment offset =
    match built.(offset) with Built box -> Some box | Building -> None | Unbuilt -> build_environment offset
  and build_environment offset =
    built.(offset) <- Building;
    let held, waiting = held_by offset environment in
    let box = make (BoxNew environments.(offset)) [| held |] in
    if waiting then Dynarray.add_last incomplete offset;
    built.(offset) <- Built box;
    Some box
  in
  let boxes = Array.init (Array.length captures) (fun offset -> Option.get (environment offset)) in
  let written =
    List.map
      (fun offset -> result (Write.box_set context.types boxes.(offset) (fst (held_by offset environment))))
      (Dynarray.to_list incomplete)
  in
  ( Seq.fold_lefti
      (fun values offset box -> Fields.add (Function (context.imported_functions + offset)) (erase box) values)
      entities (Array.to_seq boxes),
    written )
