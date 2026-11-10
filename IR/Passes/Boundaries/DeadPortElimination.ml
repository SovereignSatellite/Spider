open IR
open Definitions

let optimization =
  Options.optimization "eliminate-dead-ports" Graph Boundaries 2
    "Remove region outputs and carried values nothing reads."

let kept count keep = Array.of_list (List.filter keep (List.init count Fun.id))
let select kept items = Array.map (Array.get items) kept

let restore kept links all =
  let restored = Array.copy all in
  Array.iteri (fun position index -> restored.(index) <- links.(position)) kept;
  restored

let outputs readers region node =
  let all = Node.outputs region.types node in
  (kept (Array.length all) (fun port -> Readers.is_read readers region all.(port)), all)

let shrink_match types readers region matcher =
  let outputs, all = outputs readers region matcher in
  let unchanged_fields _ inside = inside in
  let selector, fields =
    let current = RegionMatch.selector matcher in
    match current.node.kind with
    | SumFromI32 { variants; _ } -> (
        let sum_values = SumFromI32.values current.node in
        let read index =
          List.exists
            (fun variant ->
              Readers.is_read readers (RegionMatch.branch matcher variant) (RegionMatch.field matcher variant index))
            variants
        in
        match kept (Array.length sum_values) read with
        | values when Array.length values < Array.length sum_values ->
            let fields variant inside =
              if List.mem variant variants then restore values inside sum_values else inside
            in
            (Branch.with_values types matcher variants (select values sum_values), fields)
        | _ -> (current, unchanged_fields))
    | _ -> (current, unchanged_fields)
  in
  if Array.length outputs = Array.length all && Link.equal selector (RegionMatch.selector matcher) then None
  else begin
    Readers.redefine_match readers matcher selector
      (List.map (Node.output_type types matcher) (Array.to_list outputs))
      (fun variant inside ->
        select outputs (Rewrite.splice (RegionMatch.branch matcher variant) (fields variant inside)));
    Some (restore outputs (Array.init (Array.length outputs) (Link.output matcher)) all)
  end

(** A Repeat keeps a carried value only where what decides the step or a kept field reads it, so one only the loop reads
    goes; a [Sum.FromI32] step exits and continues with the same values, so it keeps a field of both or of neither. *)
let needs readers region repeat path =
  let needed = LinkTable.create 64 in
  let fields = [| Hashtbl.create 16; Hashtbl.create 16 |] in
  let sources = NodeTable.create 16 in
  let from_i32 = List.exists (fun (producer, _) -> match producer.kind with SumFromI32 _ -> true | _ -> false) path in
  let rec need link =
    if not (LinkTable.mem needed link) then begin
      LinkTable.replace needed link ();
      match NodeTable.find_opt sources link.node with
      | Some source -> source link.port
      | None -> Array.iter need link.node.inputs
    end
  and need_part (part : Path.t) sum =
    match (sum.node.kind, part) with
    | RegionMatch _, _ -> need_branches sum.node part sum.port
    | SumFromI32 _, Tag -> need (SumFromI32.tag sum.node)
    | _, VariantField (variant, field) -> Option.iter need (Sums.field sum.node variant field)
    | _ -> ()
  and need_branches matcher part port =
    need_selector matcher Tag;
    Array.iteri
      (fun variant branch ->
        NodeTable.replace sources branch.arguments (fun field -> need_selector matcher (VariantField (variant, field)));
        need_part part branch.results.inputs.(port))
      (RegionMatch.branches matcher)
  and need_selector matcher (part : Path.t) =
    let selector = RegionMatch.selector matcher in
    match selector.node.kind with SumFromI32 _ -> need_part part selector | _ -> need selector
  and keep variant index =
    if not (Hashtbl.mem fields.(variant) index) then begin
      Hashtbl.replace fields.(variant) index ();
      need_part (VariantField (variant, index)) (RegionRepeat.step repeat);
      if from_i32 then keep (1 - variant) index
    end
  in
  NodeTable.replace sources (RegionRepeat.body repeat).arguments (keep 1);
  need_part Tag (RegionRepeat.step repeat);
  for index = 0 to Node.output_count region.types repeat - 1 do
    if Readers.is_read readers region (Link.output repeat index) then keep 0 index
  done;
  ( kept (Node.output_count region.types repeat) (Hashtbl.mem fields.(0)),
    kept (Array.length repeat.inputs) (Hashtbl.mem fields.(1)) )

(** A dropped carried value's initial value stands in until the body no longer reads it: the rebuilt step no longer
    reaches what read it, except the selector values that carried it into step Matches, which those Matches then drop,
    innermost first. *)
let rebuild_repeat types readers repeat path carried exits =
  let initial = Node.inputs repeat in
  let body = RegionRepeat.body repeat in
  let fields variant links = select (if variant = 1 then carried else exits) links in
  Readers.redefine_repeat readers repeat (select carried initial)
    (List.map (Node.output_type types repeat) (Array.to_list exits))
    (fun inside ->
      let step = (Rewrite.splice body (restore carried inside initial)).(0) in
      Step.retype types readers fields [||] body step);
  List.iter
    (fun (producer, region) ->
      match producer.kind with
      | RegionMatch _ ->
          Option.iter
            (fun outputs ->
              Rewrite.replace region producer outputs;
              Readers.edited readers region)
            (shrink_match types readers region producer)
      | _ -> ())
    (List.rev path)

let shrink_repeat types readers region repeat path =
  let all = Node.outputs types repeat in
  match needs readers region repeat path with
  | exits, carried when Array.length exits = Array.length all && Array.length carried = Array.length repeat.inputs ->
      None
  | exits, carried ->
      rebuild_repeat types readers repeat path carried exits;
      Some (restore exits (Array.init (Array.length exits) (Link.output repeat)) all)

let eliminate types readers region node =
  match node.kind with
  | RegionMatch _ -> shrink_match types readers region node
  | RegionRepeat body ->
      Option.bind (Step.producers readers body (RegionRepeat.step node)) (shrink_repeat types readers region node)
  | _ -> None

let run (table : 'x Readers.table) = Rewrite.nodes (eliminate table.region.types table table.region) table.nodes
