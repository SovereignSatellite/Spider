open IR
open Definitions

let optimization =
  Options.optimization "eliminate-common-nodes" Graph Sharing 1
    "Compute equal nodes once, a value every branch computes before the Match."

(** Region nodes are never merged: their data does not include their regions. *)
let mergeable types node = List.is_empty (Node.regions node) && NodeData.idempotent types node

type 'x key = { node : 'x node; data : 'x NodeData.t; outside : 'x link -> 'x link }

let input key index = key.outside key.node.inputs.(index)

let hash types key =
  let node = key.node in
  let commutative = Node.commutative types node in
  let hash = ref ((Hashtbl.hash (Node.name types node) * 31) + NodeData.hash key.data) in
  for index = 0 to Array.length node.inputs - 1 do
    let link = input key index in
    let input = (link.node.id * 65599) + link.port in
    hash := if commutative then !hash + input else (!hash * 31) + input
  done;
  !hash

let equal types key other =
  let left = key.node in
  let right = other.node in
  let inputs swapped =
    Indices.for_all (Array.length left.inputs) (fun index ->
        Link.equal (input key index) (input other (if swapped then 1 - index else index)))
  in
  Node.same_kind types left right
  && Array.length left.inputs = Array.length right.inputs
  && NodeData.equal key.data other.data
  && (inputs false || (Node.commutative types left && inputs true))

type ('key, 'value) keyed = {
  hash_of : 'key -> int;
  equal_to : 'key -> 'key -> bool;
  buckets : (int, 'key * 'value ref) Hashtbl.t;
}

let keyed hash_of equal_to = { hash_of; equal_to; buckets = Hashtbl.create 64 }

let cell table key =
  List.find_map
    (fun (other, cell) -> if table.equal_to key other then Some cell else None)
    (Hashtbl.find_all table.buckets (table.hash_of key))

let find_opt table key = Option.map ( ! ) (cell table key)

let try_add table key value =
  Option.is_none (cell table key)
  &&
  (Hashtbl.add table.buckets (table.hash_of key) (key, ref value);
   true)

let value types node outside = Option.map (fun data -> { node; data; outside }) (NodeData.of_node types node)
let key types node outside = if mergeable types node then value types node outside else None

let eliminate types representatives node =
  Option.bind (key types node Fun.id) (fun key ->
      match find_opt representatives key with
      | Some representative -> Some (Node.outputs types representative)
      | None ->
          ignore (try_add representatives key node);
          None)

(** The outer node a branch of a Match on the settled region's nodes gets for a value every branch before it computes
    too, made from the first branch's node; a value every branch computes runs once wherever the Match does. *)
let anticipated made (matcher, variant) key copy =
  match cell made (matcher, key) with
  | Some ({ contents = outer, count } as found) when count >= variant ->
      found := (outer, variant + 1);
      Some outer
  | None when variant = 0 ->
      let outer = copy () in
      ignore (try_add made (matcher, key) (outer, 1));
      Some outer
  | _ -> None

(** The link outside for a constant a node there reads: an outer node computing it, or else one copy shared by all. *)
let outer_constant types representatives constants (link : 'x link) =
  let key = Option.get (key types link.node Fun.id) in
  match (find_opt representatives key, find_opt constants key) with
  | Some representative, _ -> Link.output representative 0
  | None, Some copy -> copy
  | None, None ->
      let copy = Constant.copy link in
      ignore (try_add constants key copy);
      copy

(** Maps each node of a nested region computing from outer values and constants what an outer node computes, or what
    [anticipated] gives it, to that node, which it then reads through fields; work that may not repeat counts once per
    branch. Only a node that may repeat reads an outer node already there, a node reading nothing is made where it is
    read, and one computing nothing stays. *)
let taken types representatives made constants (nested : 'x Nested.t) =
  let outer = nested.outer in
  let taken = NodeTable.create 16 in
  let seen = keyed (hash types) (equal types) in
  let outside (link : 'x link) =
    if Constant.is link.node then outer_constant types representatives constants link else LinkTable.find outer link
  in
  let known (link : 'x link) = Constant.is link.node || LinkTable.mem outer link in
  let copy node () = Copying.node node (Array.map outside node.inputs) in
  let take node outer_node =
    NodeTable.replace taken node outer_node;
    for port = 0 to Node.output_count types node - 1 do
      LinkTable.replace outer (Link.output node port) (Link.output outer_node port)
    done
  in
  Array.iter
    (fun node ->
      if Array.length node.inputs > 0 && List.is_empty (Node.regions node) && Node.for_all known node then
        match value types node outside with
        | Some key when mergeable types node || try_add seen key () ->
            (match (find_opt representatives key, nested.path) with
              | Some representative, _ when mergeable types node -> Some representative
              | _, [ branch ] -> anticipated made branch key (copy node)
              | _ -> None)
            |> Option.iter (take node)
        | _ -> ())
    nested.nodes;
  taken

(** The outer nodes that exist once rewriting is done: a value made for a Match stays only if every branch took it. *)
let present representatives made =
  let present = NodeSet.create 64 in
  Hashtbl.iter (fun _ (_, representative) -> ignore (NodeSet.add present !representative)) representatives.buckets;
  Hashtbl.iter
    (fun _ ((matcher, _), { contents = outer, count }) ->
      if count = Array.length (RegionMatch.branches matcher) then ignore (NodeSet.add present outer))
    made.buckets;
  present

let add_routes routes present ((nested : 'x Nested.t), taken) =
  let routed node =
    (not (Node.structural node)) && NodeTable.mem taken node && NodeSet.mem present (NodeTable.find taken node)
  in
  Array.iter
    (fun link ->
      Dynarray.add_last routes
        {
          Branch.path = nested.path;
          link;
          outer = Link.output (NodeTable.find taken link.node) link.port;
          within = Fun.id;
        })
    (RegionNodes.crossing routed nested.nodes)

let reuse types representatives routes nodes =
  let made =
    keyed
      (fun (matcher, key) -> (matcher.id * 31) + hash types key)
      (fun (matcher, key) (other, other_key) -> matcher == other && equal types key other_key)
  in
  let constants = keyed (hash types) (equal types) in
  let mapped = Dynarray.create () in
  Nested.visit nodes (fun nested ->
      Dynarray.add_last mapped (nested, taken types representatives made constants nested));
  Dynarray.iter (add_routes routes (present representatives made)) mapped

let run (table : 'x Readers.table) =
  let types = table.region.types in
  let representatives = keyed (hash types) (equal types) in
  let merged = Rewrite.nodes (eliminate types representatives) table.nodes in
  let routes = Dynarray.create () in
  reuse types representatives routes table.nodes;
  Branch.route types table (Dynarray.to_list routes);
  merged
