open IR
open Definitions

let optimization =
  Options.optimization "place-nodes" Graph Boundaries 2
    "Move work a loop does not change before it, and work some branches read into them."

(** A read's reference may change through writes but names one object throughout, so where no write in the loop changes
    what it reads, it reads the initial one outside. The step's producer stays in the body, where [Step.retype] rebuilds
    it. *)
let movable types invariant repeat node =
  NodeData.idempotent types node && node != (RegionRepeat.step repeat).node
  && Node.exists (fun input -> not (Constant.is input.node)) node
  &&
  match Accesses.read types node with
  | Some access when not (invariant access.reference) ->
      Array.for_all invariant (Array.sub node.inputs 1 (Array.length node.inputs - 1))
      && Option.is_some (KnownValue.carried_outside types (Accesses.unwritten types) repeat access.reference)
      && not (Accesses.changes types access repeat)
  | _ -> Node.for_all invariant node

let leaving types readers repeat =
  let leaves = NodeSet.create 16 in
  let invariant link =
    NodeSet.mem leaves link.node || Option.is_some (KnownValue.carried_outside types Fun.id repeat link)
  in
  let body = RegionRepeat.body repeat in
  match Step.producers readers body (RegionRepeat.step repeat) with
  | Some path when Step.extensible path ->
      List.filter
        (fun node -> movable types invariant repeat node && NodeSet.add leaves node)
        (Array.to_list (RegionNodes.ordered body))
      |> Array.of_list
  | _ -> [||]

(** The region of the step's one exit, with work in it, when that exit is its only result: a region on the step's path
    has no other result anything reads, so all its work serves the exit. *)
let exit_region readers repeat =
  match Option.map (Step.building 0) (Step.producers readers (RegionRepeat.body repeat) (RegionRepeat.step repeat)) with
  | Some [ (({ kind = SumNew _; _ } as exit), region) ]
    when Array.length region.results.inputs = 1
         && Node.exists (fun field -> match field.node.kind with RegionArguments _ -> false | _ -> true) exit ->
      Some region
  | _ -> None

let move_across types readers repeat =
  let leaving = leaving types readers repeat in
  let leaves = NodeSet.of_list (Array.to_list leaving) in
  let body = RegionRepeat.body repeat in
  let outside link =
    if NodeSet.mem leaves link.node then link
    else Option.get (KnownValue.carried_outside types (Accesses.unwritten types) repeat link)
  in
  let inputs = Array.map (fun node -> Array.map outside node.inputs) leaving in
  let carried = RegionNodes.crossing (NodeSet.mem leaves) (RegionNodes.ordered body) in
  let initial = repeat.inputs in
  let hoist exit_types exits =
    Readers.redefine_repeat readers repeat (Array.append initial carried)
      (exit_types @ Link.types types carried)
      (fun inside ->
        let step = (Rewrite.splice body (Array.sub inside 0 (Array.length initial))).(0) in
        let within = Array.sub inside (Array.length initial) (Array.length inside - Array.length initial) in
        Rewrite.substitute body (List.combine (Array.to_list carried) (Array.to_list within));
        let fields variant links = if variant = 0 then exits links else links in
        Step.retype types readers fields within body step);
    Array.iteri (fun index node -> ignore (Node.reading inputs.(index) node)) leaving
  in
  match exit_region readers repeat with
  | Some region ->
      let fields = Region.parameters region in
      let exit = region.results.inputs.(0).node in
      hoist (Region.parameter_types region) (fun _ -> fields);
      let after link = if link.node == region.arguments then Some (Link.output repeat link.port) else None in
      Some (Copying.links after exit.inputs)
  | None when Array.length leaving > 0 ->
      hoist (Node.output_types types repeat) Fun.id;
      None
  | None -> None

let stays types readers =
  match Readers.owner readers with
  | Some ({ kind = RegionRepeat _; _ } as repeat) ->
      let leaves = NodeSet.of_list (Array.to_list (leaving types readers repeat)) in
      let step = RegionRepeat.step repeat in
      let exits matcher variant =
        step.node == matcher
        &&
        let produced = (RegionMatch.branch matcher variant).results.inputs.(step.port).node in
        Sums.builds produced 0 && not (Sums.builds produced 1)
      in
      fun node matcher variant -> NodeSet.mem leaves node && not (exits matcher variant)
  | _ -> fun _ _ _ -> false

let join left right =
  match (left, right) with
  | Some (matcher, reading), Some (other, more) when matcher == other -> Some (matcher, Array.map2 ( || ) reading more)
  | _ -> None

(** The Match each node moves into, with the branches reading it, readers first: every reader is a value of a uniform
    selector only its Match reads, or a node moving into that Match; some branch reads none of them. *)
let homes (readers : 'x Readers.table) stays =
  let region = readers.region in
  let homes = NodeTable.create 16 in
  let found = Dynarray.create () in
  let selecting selector index =
    match Readers.of_link readers region (Link.output selector 0) with
    | [ (({ kind = RegionMatch branches; _ } as matcher), 0) ] when index > 0 && Option.is_some (Branch.uniform matcher)
      ->
        let reads variant =
          Readers.is_read readers (RegionMatch.branch matcher variant) (RegionMatch.field matcher variant (index - 1))
        in
        Some (matcher, Array.init (Array.length branches) reads)
    | _ -> None
  in
  let home (reader, index) =
    match (NodeTable.find_opt homes reader, reader.kind) with
    | Some home, _ -> Some home
    | None, SumFromI32 _ -> selecting reader index
    | None, _ -> None
  in
  let stays_in node (matcher, reading) =
    Array.for_all Fun.id reading
    || Indices.exists (Array.length reading) (fun variant -> reading.(variant) && stays node matcher variant)
  in
  let count = Node.output_count region.types in
  for position = Array.length readers.nodes - 1 downto 0 do
    let node = readers.nodes.(position) in
    let is_arguments = match node.kind with RegionArguments _ -> true | _ -> false in
    if count node > 0 && not is_arguments then
      match
        List.init (count node) (fun port -> Readers.of_link readers region (Link.output node port))
        |> List.concat |> List.map home
      with
      | first :: rest -> (
          match List.fold_left join first rest with
          | Some home when not (stays_in node home) ->
              NodeTable.replace homes node home;
              Dynarray.add_last found (node, home)
          | _ -> ())
      | [] -> invalid_arg "A node every output of which no one reads has no home."
  done;
  Dynarray.to_list found

let pull_in types readers matcher moving =
  let moving_set = NodeSet.of_list moving in
  let tag, values = Option.get (Branch.uniform matcher) in
  let outside =
    Array.to_list values @ List.concat_map (fun node -> Array.to_list node.inputs) moving
    |> List.filter (fun link -> not (NodeSet.mem moving_set link.node))
    |> Link.distinct
  in
  let positions = LinkTable.create 16 in
  Array.iteri (fun index link -> LinkTable.replace positions link index) outside;
  Readers.redefine_match readers matcher
    (Branch.tagged types (Array.length (RegionMatch.branches matcher)) tag outside)
    (Node.output_types types matcher)
    (fun variant fields ->
      let field link = Option.map (Array.get fields) (LinkTable.find_opt positions link) in
      Rewrite.splice (RegionMatch.branch matcher variant) (Copying.links field values))

let run (table : 'x Readers.table) =
  let types = table.region.types in
  let homes = homes table (stays types table) in
  let housed = NodeSet.mem (NodeSet.of_list (List.map fst homes)) in
  let moved =
    Rewrite.nodes
      (fun node ->
        match node.kind with RegionRepeat _ when not (housed node) -> move_across types table node | _ -> None)
      table.nodes
  in
  List.iter
    (fun (matcher, moving) -> if not (housed matcher) then pull_in types table matcher (List.map fst moving))
    (Branch.grouped (fun (_, (matcher, _)) -> matcher) homes);
  moved
