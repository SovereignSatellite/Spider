open IR
open Definitions

let same_node_data types left_node right_node =
  match (NodeData.of_node types left_node, NodeData.of_node types right_node) with
  | Some left_data, Some right_data -> NodeData.equal left_data right_data
  | _ -> false

let regions types corresponds left right =
  let decided = Hashtbl.create 64 in
  let partners = NodeTable.create 16 in
  let rec same_link left_link right_link =
    match (left_link.node == left.arguments, right_link.node == right.arguments) with
    | true, true -> corresponds left_link.port right_link.port
    | false, false -> left_link.port = right_link.port && same_node left_link.node right_link.node
    | _ -> false
  and same_node left_node right_node =
    let key = (left_node.id, right_node.id) in
    match Hashtbl.find_opt decided key with
    | Some known -> known
    | None ->
        let same = same_structure left_node right_node in
        Hashtbl.replace decided key same;
        same
  and same_structure left_node right_node =
    match NodeTable.find_opt partners left_node with
    | Some partner -> partner == right_node
    | None ->
        Node.same_kind types left_node right_node
        && same_data left_node right_node && same_inputs left_node right_node
        && List.equal same_region (Node.regions left_node) (Node.regions right_node)
  and same_inputs left_node right_node =
    Array.length left_node.inputs = Array.length right_node.inputs
    && Indices.for_all (Array.length left_node.inputs) (fun index ->
        same_link left_node.inputs.(index) right_node.inputs.(index))
  and same_data left_node right_node =
    match (left_node.kind, right_node.kind) with
    | RegionReference left_body, RegionReference right_body when left_body == left -> right_body == right
    | _ -> same_node_data types left_node right_node
  and same_region left_region right_region =
    NodeTable.replace partners left_region.arguments right_region.arguments;
    same_inputs left_region.results right_region.results
  in
  same_inputs left.results right.results
