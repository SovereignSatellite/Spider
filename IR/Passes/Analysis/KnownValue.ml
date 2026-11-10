open IR
open Definitions

type 'x knowledge = SameAs of 'x link | Unproduced | Unknown

let same types left right =
  Link.equal left right
  || Constant.is left.node
     && Node.same_kind types left.node right.node
     && Option.equal NodeData.equal (NodeData.of_node types left.node) (NodeData.of_node types right.node)

let join types left right =
  match (left, right) with
  | Unproduced, knowledge | knowledge, Unproduced -> knowledge
  | SameAs first, SameAs second when same types first second -> left
  | _ -> Unknown

let outside matcher variant knowledge =
  match knowledge with
  | SameAs link when Constant.is link.node -> knowledge
  | SameAs link -> ( match Branch.outside matcher variant link with Some outer -> SameAs outer | None -> Unknown)
  | Unproduced | Unknown -> knowledge

let across_branches types matcher port knowledge_of =
  Seq.fold_lefti
    (fun known variant branch -> join types known (outside matcher variant (knowledge_of branch.results.inputs.(port))))
    Unproduced
    (Array.to_seq (RegionMatch.branches matcher))

let rec field_across types see variant field matcher port =
  match across_branches types matcher port (field_through types see variant field) with
  | SameAs outer -> SameAs (see outer)
  | knowledge -> knowledge

and field_through types see variant field link =
  match link.node.kind with
  | RegionMatch _ -> field_across types see variant field link.node link.port
  | SumNew _ | SumFromI32 _ -> (
      match Sums.field link.node variant field with Some found -> SameAs (see found) | None -> Unproduced)
  | _ -> Unknown

(** Carried values that start from the same link and continue, as [see] sees them, from one another in a cycle hold that
    link throughout. *)
let is_invariant types see repeat index =
  let body = RegionRepeat.body repeat in
  let rec follow visited port =
    List.mem port visited
    || Link.equal repeat.inputs.(port) repeat.inputs.(index)
       &&
       match field_through types see 1 port (RegionRepeat.step repeat) with
       | SameAs link when link.node == body.arguments -> follow (port :: visited) link.port
       | _ -> false
  in
  follow [] index

let carried_outside types see repeat link =
  if Constant.is link.node then Some (Constant.copy link)
  else if link.node == (RegionRepeat.body repeat).arguments && is_invariant types see repeat link.port then
    Some repeat.inputs.(link.port)
  else None

let carried_known types repeat index =
  let initial = repeat.inputs.(index) in
  let holds_initial port = Link.equal repeat.inputs.(port) initial && is_invariant types Fun.id repeat port in
  if not (is_invariant types Fun.id repeat index) then None
  else if Constant.is initial.node then Some (Constant.copy initial)
  else Option.map (Region.parameter (RegionRepeat.body repeat)) (List.find_opt holds_initial (List.init index Fun.id))
