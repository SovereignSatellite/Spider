open IR

let literal types (node : 'x Node.t) =
  Array.length node.inputs = 0
  && (Node.operation node || Node.forwarding types node)
  && not (Node.unrepeatable types node)

let rec tag_of (link : 'x Link.t) =
  match link.node.kind with I32FromSum | SumFromI32 { tagged = true; _ } -> tag_of link.node.inputs.(0) | _ -> link

let deciding schedule flows region tested =
  let tag = tag_of tested in
  match tag.node.kind with
  | RegionMatch _ ->
      let matcher = Schedule.number schedule tag.node in
      let results = Schedule.end_ schedule region - 1 in
      let silent number =
        number = results || Flows.computes_nothing flows number
        || literal (Flows.types flows) (Schedule.node schedule number)
      in
      if
        Schedule.start schedule region < matcher
        && List.for_all silent (List.init (results - matcher) (( + ) (matcher + 1)))
      then Some (tag.node, tag.port)
      else None
  | _ -> None

let constant (link : 'x Link.t) =
  match (link.node.kind, Node.bits link.node.kind) with
  | SumNew { variant; _ }, _ when Array.length link.node.inputs = 0 -> Some variant
  | _, Some bits -> Some (Int64.to_int bits)
  | _ -> None

(** The constants [field] holds wherever the loop leaves through [tested], a result of [region], if it holds one at
    each. *)
let rec exit_values schedule flows region tested (field : 'x Link.t) =
  match deciding schedule flows region tested with
  | Some (matcher, port) ->
      List.fold_left
        (fun found (branch : 'x Region.t) ->
          let held = if field.node == matcher then branch.results.inputs.(field.port) else field in
          Option.bind found (fun values ->
              Option.map (( @ ) values) (exit_values schedule flows branch branch.results.inputs.(port) held)))
        (Some [])
        (Array.to_list (RegionMatch.branches matcher))
  | None -> (
      match constant (tag_of tested) with
      | Some variant when variant <> 0 -> Some []
      | _ -> Option.map (fun value -> [ value ]) (constant field))

(** The Match evaluated next after the loop, selecting on one of its exit values, with that value's field of the step,
    where every exit gives the field a constant of its own, so that each exit runs the branch it selects. *)
let following schedule flows repeat =
  let body = RegionRepeat.body repeat in
  let step = RegionRepeat.step repeat in
  let rec next number =
    let node = Schedule.node schedule number in
    match node.kind with
    | RegionResults _ -> None
    | _ when Flows.computes_nothing flows number || literal (Flows.types flows) node -> next (number + 1)
    | _ -> Some node
  in
  match (next (Schedule.number schedule repeat + 1), step.node.kind) with
  | Some ({ kind = RegionMatch branches; _ } as matcher), SumFromI32 { tagged = true; _ } -> (
      let selected = tag_of (RegionMatch.selector matcher) in
      if selected.node != repeat then None
      else
        let field = (SumFromI32.values step.node).(selected.port) in
        match exit_values schedule flows body step field with
        | Some values
          when List.length (List.sort_uniq compare values) = List.length values
               && List.for_all (fun value -> 0 <= value && value < Array.length branches) values ->
            Some (matcher, field)
        | _ -> None)
  | _ -> None

let rec decide schedule flows deciders tested_at_end region tested =
  match deciding schedule flows region tested with
  | Some (matcher, port) ->
      ignore (NodeSet.add deciders matcher);
      Array.iter
        (fun (branch : 'x Region.t) ->
          let result = branch.results.inputs.(port) in
          if not (decide schedule flows deciders tested_at_end branch result) then tested_at_end result)
        (RegionMatch.branches matcher);
      true
  | None -> false

let decided schedule flows replaced tested_at_end followed =
  let deciders = NodeSet.create 64 in
  let repeats =
    List.filter
      (fun (node : 'x Node.t) -> match node.kind with RegionRepeat _ -> true | _ -> false)
      (List.init (Schedule.count schedule) (Schedule.node schedule))
  in
  List.iter
    (fun repeat ->
      let step = RegionRepeat.step repeat in
      if decide schedule flows deciders tested_at_end (RegionRepeat.body repeat) step then replaced step)
    repeats;
  List.iter
    (fun repeat ->
      match following schedule flows repeat with
      | Some ((matcher, _) as next) when not (NodeSet.mem deciders matcher) ->
          replaced (RegionMatch.selector matcher);
          followed repeat next
      | _ -> ())
    repeats
