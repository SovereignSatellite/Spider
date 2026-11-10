open IR

(** After a restructuring the region steps again before its edited regions settle, since another restructuring may edit
    them again. Each time the regions inside settle, the region gets a fresh step, which may cache what it reads of
    them. *)
let settle_body start settled added body =
  let unsettled = RegionSet.create 16 in
  let rec settle owner region =
    settle_children region;
    let rec steps step restructured =
      match step () with
      | Revisit.Subtree edited ->
          List.iter
            (fun inner ->
              RegionSet.remove added inner;
              ignore (RegionSet.add unsettled inner))
            edited;
          steps (start owner region) true
      | _ when restructured ->
          settle_children region;
          steps (start owner region) false
      | Revisit.Region -> steps step false
      | Revisit.Nothing -> ()
    in
    steps (start owner region) false;
    ignore (RegionSet.add added region);
    RegionSet.remove unsettled region
  and settle_children region =
    Array.iter
      (fun node ->
        List.iter
          (fun child ->
            if RegionSet.mem unsettled child || not (RegionSet.mem settled child || RegionSet.mem added child) then
              settle (Some node) child)
          (Node.regions node))
      (RegionNodes.ordered region)
  in
  settle None body

let run_fixpoint start settled functions =
  functions
  |> List.filter (fun body -> not (RegionSet.mem settled body))
  |> Bodies.in_parallel (settle_body start settled)
  |> RegionSet.union settled
