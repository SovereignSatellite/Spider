open IR
open Definitions

let optimization =
  Options.optimization "commute-matches" Control Restructuring 2
    "Move a node reading a Match's results into the branches where its work goes."

let choice_of types input =
  match ((Branch.root input).node.kind, input.node.kind) with
  | RegionMatch _, _ -> Some (Branch.root input).node
  | _, I32FromSum -> Some (RegionMatch.create types input.node.inputs.(0) [] (fun _ _ -> [||]))
  | _ -> None

(** A new Match decides nothing yet: it has no outputs and its branches compute and read nothing, so nothing depends on
    it. *)
let decides types matcher = Node.output_count types matcher > 0

let resolved types matcher variant input =
  if input.node == matcher then Some (RegionMatch.branch matcher variant).results.inputs.(input.port)
  else Branch.known types (RegionMatch.selector matcher) variant input

let carries matcher input = (Branch.root input).node == matcher

let rec within types fold matcher variant receive input =
  match resolved types matcher variant input with
  | Some link -> link
  | None when carries matcher input -> (
      let copied = Copying.node input.node (Array.map (within types fold matcher variant receive) input.node.inputs) in
      match fold copied with Some links -> links.(input.port) | None -> Link.output copied input.port)
  | None -> receive input

let rec received types matcher input =
  if
    Indices.for_all
      (Array.length (RegionMatch.branches matcher))
      (fun variant -> Option.is_some (resolved types matcher variant input))
  then []
  else if carries matcher input then List.concat_map (received types matcher) (Array.to_list input.node.inputs)
  else [ input ]

type piece = Nothing | Taken of int | Whole

(** The piece is built in a region whose parameters stand for what the branch receives, so no rule sees through them. *)
let piece types fold table matcher node outside variant =
  let region = Region.create types (Link.types types outside) (Node.output_types types node) in
  let inputs =
    Array.map
      (within types fold matcher variant (fun input -> Region.parameter region (Link.index_of outside input)))
      node.inputs
  in
  let computed, values =
    match node.kind with
    | RegionMatch inner_branches -> (
        match Sums.built inputs.(0).node with
        | Some [ (named, fields) ] -> (Taken named, Copying.results inner_branches.(named) fields)
        | _ -> (Whole, Node.outputs types node))
    | _ -> (Whole, match fold (Copying.node node inputs) with Some links -> links | None -> Node.outputs types node)
  in
  let present link =
    Constant.is link.node || link.node == region.arguments
    || (decides types matcher && Readers.is_read table (RegionMatch.branch matcher variant) link)
  in
  let kept = if computed = Whole && Array.for_all present values then Nothing else computed in
  Region.define region (if kept = Whole then Node.outputs types node else values);
  (kept, region, inputs)

let depends_on node value =
  let visited = NodeSet.create 64 in
  let rec reaches current =
    current == node || (NodeSet.add visited current && Node.exists (fun input -> reaches input.node) current)
  in
  reaches value.node

let commute types table node matcher outside found =
  let count = Node.output_count types matcher in
  (* A new Match's branches read no fields, so it selects on its sum's tag where it receives values. *)
  let selector =
    match Branch.uniform matcher with
    | None when Array.length outside > 0 ->
        Branch.tagged types
          (Array.length (RegionMatch.branches matcher))
          (Link.output (Node.create I32FromSum [| RegionMatch.selector matcher |]) 0)
          outside
    | _ -> Branch.extend types matcher outside
  in
  Array.iter (fun (kept, _, inputs) -> if kept = Whole then ignore (Node.reading inputs node)) found;
  Readers.redefine_match table matcher selector
    (Node.output_types types matcher @ Node.output_types types node)
    (fun variant fields ->
      let _, region, _ = found.(variant) in
      let spliced =
        Rewrite.splice region (Array.sub fields (Array.length fields - Array.length outside) (Array.length outside))
      in
      Array.append (RegionMatch.branch matcher variant).results.inputs spliced);
  Array.init (Node.output_count types node) (fun port -> Link.output matcher (count + port))

let accepted types matcher outside found =
  let kept =
    List.filter (fun piece -> piece <> Nothing) (List.map (fun (piece, _, _) -> piece) (Array.to_list found))
  in
  List.length (List.sort_uniq compare kept) = List.length kept
  && ((not (List.mem Whole kept)) || (List.length kept = 1 && decides types matcher))
  && not (decides types matcher && Array.exists (depends_on matcher) outside)

let attempt types fold table node matcher =
  let outside = Link.distinct (List.concat_map (received types matcher) (Array.to_list node.inputs)) in
  if Array.length outside = 0 || Option.is_some (Branch.uniform matcher) || not (decides types matcher) then
    let found =
      Array.init (Array.length (RegionMatch.branches matcher)) (piece types fold table matcher node outside)
    in
    if accepted types matcher outside found then Some (commute types table node matcher outside found) else None
  else None

let commuted types fold table node =
  let commutes =
    match node.kind with RegionMatch _ -> true | RegionResults _ -> false | _ -> not (Node.forwarding types node)
  in
  if not commutes then None
  else
    Array.find_map
      (fun input ->
        match choice_of types input with
        | Some matcher when Array.length (RegionMatch.branches matcher) > 1 -> attempt types fold table node matcher
        | _ -> None)
      (Node.inputs node)

let run types fold table = Rewrite.nodes (commuted types fold table) table.nodes
