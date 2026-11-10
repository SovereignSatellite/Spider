open IR
open Definitions

let ordered_with visited order region =
  NodeSet.clear visited;
  Dynarray.clear order;
  let rec visit node =
    if NodeSet.add visited node then begin
      Array.iter (fun input -> visit input.node) node.inputs;
      Dynarray.add_last order node
    end
  in
  visit region.results;
  Dynarray.to_array order

let ordered region = ordered_with (NodeSet.create 64) (Dynarray.create ()) region

let crossing moves nodes =
  nodes |> Array.to_list
  |> List.filter (fun node -> not (moves node))
  |> List.concat_map (fun node -> Array.to_list node.inputs)
  |> List.filter (fun input -> moves input.node)
  |> Link.distinct

let rec any_runs predicate node =
  predicate node || List.exists (fun region -> Array.exists (any_runs predicate) (ordered region)) (Node.regions node)
