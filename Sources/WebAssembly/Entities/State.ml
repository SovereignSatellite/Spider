open IR
open Definitions
open Entities

let position_opt fields field =
  let rec search low high =
    if low > high then None
    else
      let middle = (low + high) / 2 in
      match Field.compare fields.(middle) field with
      | 0 -> Some middle
      | order when order < 0 -> search (middle + 1) high
      | _ -> search low (middle - 1)
  in
  search 0 (Array.length fields - 1)

let position fields field = Option.get (position_opt fields field)

type 'x t = {
  token : 'x link;
  uses : Field.t array;
  held : 'x link array;
  ordered : Field.t array;
  contents : 'x link array;
  pending : 'x link array;
}

let item state field = state.held.(position state.uses field)
let links state = Array.concat [ [| state.token |]; state.held; state.contents; state.pending ]

let of_links state links =
  let uses = Array.length state.uses in
  let ordered = Array.length state.ordered in
  let start = 1 + uses in
  {
    state with
    token = links.(0);
    held = Array.sub links 1 uses;
    contents = Array.sub links start ordered;
    pending = Array.sub links (start + ordered) ordered;
  }

let create context token uses boxes ordered =
  {
    token;
    uses;
    held = boxes;
    ordered;
    contents = Array.map (fun field -> held context field boxes.(position uses field)) ordered;
    pending = Array.make (Array.length ordered) token;
  }

let of_map context token values ordered =
  let bindings = Fields.bindings values in
  create context token (Array.of_list (List.map fst bindings)) (Array.of_list (List.map snd bindings)) ordered

let enter context index fields environment erased token =
  let uses = Array.of_list (Field.Function index :: fields) in
  Array.sort Field.compare uses;
  let erased = fence erased [| token |] in
  let held = make (BoxRetype environment) [| erased |] in
  let read =
    RegionMatch.create context.types
      (result (BoxGet.create context.types held))
      (List.map (Context.field_type context) fields)
      (fun _ entities -> entities)
  in
  let values = Array.make (Array.length uses) erased in
  List.iteri (fun output field -> values.(position uses field) <- Link.output read output) fields;
  let accessed = Hashtbl.find context.effects (Function index) in
  create context token uses values
    (Array.of_list (List.filter (fun field -> Fields.mem field accessed) (Array.to_list uses)))

let contents context state field (access : Access.t) =
  match position_opt state.ordered field with
  | None -> held context field (item state field)
  | Some index when access = Reads -> state.contents.(index)
  | Some index -> fence state.contents.(index) [| state.pending.(index) |]

let box state field =
  match position_opt state.ordered field with
  | None -> item state field
  | Some index -> fence (item state field) [| state.pending.(index) |]

let after state field access result =
  Option.iter
    (fun index ->
      if access >= Access.Writes then state.contents.(index) <- result;
      state.pending.(index) <- fence state.pending.(index) [| result |])
    (position_opt state.ordered field);
  state

let replaced state field box contents =
  state.held.(position state.uses field) <- box;
  Option.iter (fun index -> state.contents.(index) <- contents) (position_opt state.ordered field);
  after state field Reads box

let unwound context state token =
  let after link = fence link [| token |] in
  let boxes = Array.map after state.held in
  { (create context (after state.token) state.uses boxes state.ordered) with pending = Array.map after state.pending }

let settle state = { state with token = fence state.token state.pending }
