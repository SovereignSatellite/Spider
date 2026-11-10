open LiveRanges
open Intervals

let interfere small large =
  let rec overlaps range index =
    index < large.count && (get large index).from < range.to_
    && ((get large index).value <> range.value || overlaps range (index + 1))
  in
  let rec test index =
    index < small.count
    && (overlaps (get small index) (LiveRanges.first_ending_after large (get small index).from) || test (index + 1))
  in
  test 0

(** Ranked by structure: a node's only region runs at least as often as the node, and one of several regions at most as
    often. *)
let most_frequent_first schedule flows =
  let ranked = Dynarray.create () in
  let rec visit repeated branched region =
    for number = Schedule.start schedule region to Schedule.end_ schedule region - 1 do
      (match List.filter (fun (edge : Flows.edge) -> edge.copied) (Array.to_list (Flows.edges flows number)) with
      | [] -> ()
      | copies -> Dynarray.add_last ranked ((-repeated, branched), copies));
      visit_regions repeated branched (IR.Node.regions (Schedule.node schedule number))
    done
  and visit_regions repeated branched regions =
    match regions with
    | [ region ] -> visit (repeated + 1) branched region
    | regions -> List.iter (visit repeated (branched + 1)) regions
  in
  visit 0 0 (Schedule.body schedule);
  let ordered = Dynarray.to_list ranked |> List.stable_sort (fun (rank, _) (other, _) -> compare rank other) in
  List.concat_map snd ordered

(** The merged ranges go to the source's interval, since the join keeps the source's root, by which intervals are found.
*)
let try_join flows intervals source destination =
  let small, large =
    if source.ranges.count <= destination.ranges.count then (source.ranges, destination.ranges)
    else (destination.ranges, source.ranges)
  in
  if not (interfere small large) then begin
    Flows.join flows source.congruence destination.congruence;
    source.ranges <- LiveRanges.merge large small;
    intervals.(destination.congruence) <- None
  end

let coalesce schedule flows intervals =
  List.iter
    (fun (edge : Flows.edge) ->
      match (intervals.(Flows.congruence flows edge.source), intervals.(Flows.congruence flows edge.destination)) with
      | Some source, Some destination when source != destination -> try_join flows intervals source destination
      | _ -> ())
    (most_frequent_first schedule flows)
