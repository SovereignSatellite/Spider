open IR
open Definitions

type 'x relation = { comparison : 'x link; holds : int }

let relation types link =
  let holds = Node.holds types link.node in
  if holds <> 0 then Some { comparison = link; holds } else None

let operands relation = (relation.comparison.node.inputs.(0), relation.comparison.node.inputs.(1))

let swapped holds =
  holds land (Outcomes.equal lor Outcomes.unordered)
  lor ((holds land Outcomes.less) lsl 2)
  lor ((holds land Outcomes.greater) lsr 2)

let aligned frame other =
  let left, right = operands frame in
  match operands other with
  | other_left, other_right when Link.equal other_left left && Link.equal other_right right -> Some other.holds
  | other_left, other_right when Link.equal other_left right && Link.equal other_right left ->
      Some (swapped other.holds)
  | _ -> None

(** Only comparisons of one node kind are known to share an ordering (signed and unsigned do not); equality is the same
    in all. *)
let kindred types relations =
  relations
  |> List.filter (fun relation -> relation.holds <> Outcomes.equal)
  |> List.map (fun relation -> Node.name types relation.comparison.node)
  |> List.sort_uniq String.compare |> List.length <= 1

let possible types frame =
  let left, right = operands frame in
  let unordered =
    match TypeArena.definition types (Link.type_ types left) with
    | Structural (Integer _) -> 0
    | _ -> Outcomes.unordered
  in
  if Link.equal left right then Outcomes.equal lor unordered
  else Outcomes.less lor Outcomes.equal lor Outcomes.greater lor unordered

let tagged types link = match link.node.kind with I32FromSum -> relation types link.node.inputs.(0) | _ -> None
let compared types link = match tagged types link with Some _ as found -> found | None -> relation types link
let truth_of variant = match variant with 0 -> Some false | 1 -> Some true | _ -> None
let outcomes = [ Outcomes.less; Outcomes.equal; Outcomes.greater; Outcomes.unordered ]

let candidates frame relations =
  List.concat_map
    (fun relation ->
      let holds = Option.get (aligned frame relation) in
      let node = relation.comparison.node in
      [
        (holds, fun () -> relation.comparison);
        (swapped holds, fun () -> Link.output (Copying.node node [| node.inputs.(1); node.inputs.(0) |]) 0);
      ])
    relations

let distinct relations =
  List.fold_left
    (fun found relation ->
      if List.exists (fun other -> Link.equal other.comparison relation.comparison) found then found
      else found @ [ relation ])
    [] relations

let reduce types output_type relations holds =
  let frame = List.hd relations in
  let possible = possible types frame in
  let occurring = List.filter (fun outcome -> possible land outcome <> 0) outcomes in
  let as_output boolean =
    if output_type = Type.i32 then Link.output (Node.create I32FromSum [| boolean |]) 0 else boolean
  in
  let constant truth = Option.get (Branch.variant types output_type (if truth then 1 else 0)) in
  let holding table =
    List.find_opt (fun (candidate, _) -> candidate land possible = table) (candidates frame relations)
    |> Option.map (fun (_, build) -> build ())
  in
  let aligns =
    (output_type = Type.boolean || output_type = Type.i32)
    && List.for_all (fun relation -> Option.is_some (aligned frame relation)) relations
    && kindred types relations
  in
  let truths = if aligns then List.map holds occurring else [] in
  if (not aligns) || not (List.for_all Option.is_some truths) then None
  else
    let wanted =
      List.fold_left2 (fun sum outcome truth -> if Option.get truth then sum + outcome else sum) 0 occurring truths
    in
    match (holding wanted, holding (possible lxor wanted)) with
    | _ when wanted = 0 -> Some (constant false)
    | _ when wanted = possible -> Some (constant true)
    | Some comparison, _ -> Some (as_output comparison)
    | None, Some complement when List.length relations > 1 ->
        Some (Branch.choose_one types complement [||] output_type (fun _ -> constant true) (fun _ -> constant false))
    | _ -> None

let holds_on frame outcome relation = Option.get (aligned frame relation) land outcome <> 0

let of_operation types node =
  let inputs = node.inputs in
  let read = Array.map (tagged types) inputs in
  let relations = distinct (List.filter_map Fun.id (Array.to_list read)) in
  let constants =
    Array.for_all2 (fun input relation -> Option.is_some relation || Constant.is input.node) inputs read
  in
  if Node.operation node && Node.output_count types node = 1 && constants && not (List.is_empty relations) then
    let holds outcome =
      let values =
        Array.map2
          (fun input relation ->
            match relation with
            | Some relation ->
                Link.output
                  (Node.create (I32Constant (if holds_on (List.hd relations) outcome relation then 1 else 0)) [||])
                  0
            | None -> input)
          inputs read
      in
      Option.bind
        (Option.bind
           (ConstantFolding.fold Options.all types (Copying.node node values))
           (fun folded -> Branch.variant_of types folded.(0)))
        truth_of
    in
    reduce types (Node.output_type types node 0) relations holds
  else None

type 'x given = Constant of bool | Compared of 'x relation

let given types matcher variant port =
  let result = (RegionMatch.branch matcher variant).results.inputs.(port) in
  let inside = match result.node.kind with I32FromSum -> result.node.inputs.(0) | _ -> result in
  match Branch.variant_of types result with
  | Some variant -> Option.map (fun truth -> Constant truth) (truth_of variant)
  | None ->
      Option.map
        (fun relation -> Compared relation)
        (Option.bind (Branch.outside matcher variant inside) (compared types))

let condition types matcher =
  let selector = RegionMatch.selector matcher in
  match selector.node.kind with
  | _ when Link.type_ types selector = Type.boolean -> Some selector
  | SumFromI32 _ when Array.length (RegionMatch.branches matcher) = 2 -> (
      let tag = SumFromI32.tag selector.node in
      match tag.node.kind with
      | I32FromSum when Node.operand_type types tag.node = Type.boolean -> Some tag.node.inputs.(0)
      | _ -> None)
  | _ -> None

let of_match types matcher port =
  match Option.bind (condition types matcher) (relation types) with
  | Some frame -> (
      match (given types matcher 0 port, given types matcher 1 port) with
      | Some on_false, Some on_true ->
          let givens = [| on_false; on_true |] in
          let relations =
            distinct
              (frame
              :: List.filter_map
                   (fun given -> match given with Compared relation -> Some relation | Constant _ -> None)
                   (Array.to_list givens))
          in
          let holds outcome =
            match givens.(if holds_on frame outcome frame then 1 else 0) with
            | Constant truth -> Some truth
            | Compared relation -> Some (holds_on frame outcome relation)
          in
          reduce types (Node.output_type types matcher port) relations holds
      | _ -> None)
  | None -> None
