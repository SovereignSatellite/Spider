open Definitions

let named body =
  let visited = NodeSet.create 64 in
  let rec visit node =
    if NodeSet.add visited node then
      List.concat_map (fun input -> visit input.node) (Array.to_list node.inputs)
      @
      match node.kind with
      | RegionReference body -> [ body ]
      | _ -> List.concat_map (fun region -> visit region.results) (Node.regions node)
    else []
  in
  visit body.results

let functions root =
  let visited = RegionSet.create 64 in
  let ordered = Dynarray.create () in
  let rec visit_body body =
    if RegionSet.add visited body then begin
      List.iter visit_body (named body);
      Dynarray.add_last ordered body
    end
  in
  visit_body root;
  Dynarray.to_list ordered
