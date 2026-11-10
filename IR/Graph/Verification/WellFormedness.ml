open Definitions

let walk_regions types root =
  let owners = NodeTable.create 1024 in
  let claimed = Dynarray.create () in
  let regions = Dynarray.create () in
  let errors = Dynarray.create () in
  let reading = NodeSet.create 64 in
  let claim region node =
    match NodeTable.find_opt owners node with
    | Some owner ->
        if owner != region.arguments then
          Dynarray.add_last errors (Node.name types node ^ ": used from more than one region");
        false
    | None ->
        NodeTable.replace owners node region.arguments;
        Dynarray.add_last claimed node;
        true
  in
  let rec walk_region region =
    if claim region region.arguments then begin
      Dynarray.add_last regions region;
      walk_node region region.results
    end
  and walk_node region node =
    if claim region node then begin
      ignore (NodeSet.add reading node);
      Array.iteri (walk_input region node) node.inputs;
      NodeSet.remove reading node;
      match node.kind with RegionReference body -> walk_region body | _ -> List.iter walk_region (Node.regions node)
    end
  and walk_input region node input_index input =
    if input.port >= Node.output_count types input.node then
      Dynarray.add_last errors
        (Printf.sprintf "%s: input %d names port %d of %s, which has %d outputs" (Node.name types node) input_index
           input.port (Node.name types input.node) (Node.output_count types input.node));
    if NodeSet.mem reading input.node then
      Dynarray.add_last errors (Printf.sprintf "%s: input %d closes a cycle" (Node.name types node) input_index);
    walk_node region input.node
  in
  walk_region root;
  (Dynarray.to_list claimed, Dynarray.to_list regions, Dynarray.to_list errors)

let root_errors root =
  (if Region.parameter_types root <> [ Type.unit ] then [ "the root takes other than the start" ] else [])
  @
  if Region.result_count root <> 1 then [ Printf.sprintf "the root has %d results, not one" (Region.result_count root) ]
  else []

let typing_errors types nodes regions =
  List.concat_map (Typing.region types) regions
  @ List.concat_map
      (fun node -> List.map (fun message -> Node.name types node ^ ": " ^ message) (Typing.check types node))
      nodes

let check (compilation_unit : 'x CompilationUnit.t) =
  let types = compilation_unit.types in
  match (root_errors compilation_unit.root, walk_regions types compilation_unit.root) with
  | [], (nodes, regions, []) -> typing_errors types nodes regions
  | [], (_, _, structural_errors) -> structural_errors
  | errors, _ -> errors
