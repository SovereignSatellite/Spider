open IR
open Definitions

let join gathered found = match (gathered, found) with Some earlier, Some later -> Some (earlier @ later) | _ -> None

let rec producers readers region step =
  match step.node.kind with
  | RegionMatch _ when List.length (Readers.of_link readers region step) = 1 ->
      branch_producers readers region step.node step
  | _ when Sums.producing step.node -> Some [ (step.node, region) ]
  | _ -> None

and branch_producers readers region matcher step =
  RegionMatch.branches matcher |> Array.to_list
  |> List.map (fun branch -> producers readers branch branch.results.inputs.(step.port))
  |> List.fold_left join (Some [ (step.node, region) ])

let building variant path = List.filter (fun (producer, _) -> Sums.builds producer variant) path

let extensible path =
  List.for_all
    (fun (producer, _) ->
      match producer.kind with RegionMatch _ -> Option.is_some (Branch.uniform producer) | _ -> true)
    path

let replaced_at port inserted items =
  List.concat (List.mapi (fun index item -> if index = port then inserted else [ item ]) items)

let rec rebuild types readers step_types produce extra region step =
  match step.node.kind with
  | RegionMatch _ -> rebuild_match types readers step_types produce extra region step.node step.port
  | _ -> produce step.node extra

and rebuild_match types readers step_types produce extra region matcher port =
  let added = List.length step_types - 1 in
  let counts = Array.map (fun branch -> Array.length branch.parameter_types) (RegionMatch.branches matcher) in
  Readers.redefine_match readers matcher (Branch.extend types matcher extra)
    (replaced_at port step_types (Node.output_types types matcher))
    (fun variant fields ->
      let branch = RegionMatch.branch matcher variant in
      let results = Rewrite.splice branch (Array.sub fields 0 counts.(variant)) in
      let produced =
        rebuild types readers step_types produce
          (Array.sub fields counts.(variant) (Array.length fields - counts.(variant)))
          branch results.(port)
      in
      Array.of_list (replaced_at port (Array.to_list produced) (Array.to_list results)));
  let moved =
    Array.init (Node.output_count types matcher) (fun index ->
        if index > port then Link.output matcher (index + added) else Link.output matcher index)
  in
  Rewrite.replace region matcher moved;
  Array.init (List.length step_types) (fun index -> Link.output matcher (port + index))

let retype types readers fields extra region step =
  let step_type = Region.result_type region 0 in
  let produce sum extra_inside =
    let variant, links = List.hd (Option.get (Sums.built sum)) in
    let values = Array.append (fields variant links) extra_inside in
    [|
      (match sum.kind with
      | SumNew { variant; _ } -> Link.output (Node.create (SumNew { sum_type = step_type; variant }) values) 0
      | _ -> Link.output (SumFromI32.create types step_type (SumFromI32.tag sum) values) 0);
    |]
  in
  (rebuild types readers [ step_type ] produce extra region step).(0)
