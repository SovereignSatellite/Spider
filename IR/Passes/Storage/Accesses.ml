open IR
open Definitions

type 'x access = { reference : 'x link; index : 'x link; offset : int64; width : int64; value_type : Type.t }

(** A box holds one element, at a constant index never linked into a graph. *)
let box_index () = Link.output (Node.create (I32Constant 0) [||]) 0

let access node index offset width value_type = { reference = node.inputs.(0); index; offset; width; value_type }

let read types node =
  match (Node.array_access types node, node.kind) with
  | Some array, _ when not (Node.write types node) ->
      Some
        (access node node.inputs.(1) (Int64.of_int array.offset) (Int64.of_int array.width)
           (Node.output_type types node 0))
  | _, BoxGet _ -> Some (access node (box_index ()) 0L 1L (Node.output_type types node 0))
  | _ -> None

let write types node =
  match (Node.array_access types node, node.kind) with
  | Some array, _ when Node.write types node ->
      Some
        ( access node node.inputs.(1) (Int64.of_int array.offset) (Int64.of_int array.width)
            (Link.type_ types node.inputs.(2)),
          node.inputs.(2) )
  | _, BoxSet _ -> Some (access node (box_index ()) 0L 1L (Link.type_ types node.inputs.(1)), node.inputs.(1))
  | _ -> None

let bulk_write node =
  match node.kind with
  | ArrayFill _ -> Some (node.inputs.(1), node.inputs.(2))
  | ArrayCopy _ -> Some (node.inputs.(1), node.inputs.(4))
  | _ -> None

let rec root types reference =
  match reference.node.kind with
  | Fence -> root types reference.node.inputs.(0)
  | _ when Node.write types reference.node -> root types reference.node.inputs.(0)
  | _ -> reference

let rec unwritten types reference =
  if Node.write types reference.node then unwritten types reference.node.inputs.(0) else reference

let matched_creation matcher reference =
  let gives variant branch =
    let result = branch.results.inputs.(reference.port) in
    match variant with
    | 1 -> Link.equal result (RegionMatch.field matcher 1 0)
    | _ -> ( match result.node.kind with Throw _ -> true | _ -> false)
  in
  let branches = RegionMatch.branches matcher in
  if Indices.for_all (Array.length branches) (fun variant -> gives variant branches.(variant)) then
    Some (RegionMatch.selector matcher).node
  else None

let creation reference =
  match reference.node.kind with
  | BoxNew _ | ArrayOf _ -> Some reference.node
  | RegionMatch _ -> (
      match (RegionMatch.selector reference.node).node.kind with
      | ArrayNew _ -> matched_creation reference.node reference
      | _ -> None)
  | _ -> None

let low_word = 0xFFFF_FFFFL

(** In-bounds accesses lie below 2^32, so their distance modulo 2^32 decides whether they overlap. *)
let distance from_index from_offset to_index to_offset =
  let from_origin, from_constant = Offsets.split from_index in
  let to_origin, to_constant = Offsets.split to_index in
  if Option.equal Link.equal from_origin to_origin then
    Some (Int64.logand (Int64.sub (Int64.sub (Int64.add to_constant to_offset) from_constant) from_offset) low_word)
  else None

let at_least left right = Int64.unsigned_compare left right >= 0

let disjoint left_width right_width apart =
  at_least apart left_width && at_least (Int64.sub 0x1_0000_0000L apart) right_width

let same_index left right =
  match distance left.index left.offset right.index right.offset with
  | Some 0L when left.width = right.width -> Some true
  | Some apart when disjoint left.width right.width apart -> Some false
  | _ -> None

let distinct_objects types left right =
  Link.type_ types left <> Link.type_ types right
  ||
  match (creation (root types left), creation (root types right)) with
  | Some left_creation, Some right_creation -> left_creation != right_creation
  | _ -> false

type overlap = Inside of int64 | Outside | Unknown

let overlap_of_length access start length =
  match distance start 0L access.index access.offset with
  | Some apart when at_least length (Int64.add apart access.width) -> Inside apart
  | Some apart when disjoint length access.width apart -> Outside
  | _ -> Unknown

let overlap access start count =
  match Constant.bits count with Some length -> overlap_of_length access start length | None -> Unknown

let may_write types access node =
  match (write types node, bulk_write node) with
  | Some (written, _), _ ->
      (not (distinct_objects types written.reference access.reference)) && same_index written access <> Some false
  | _, Some (start, count) ->
      (not (distinct_objects types node.inputs.(0) access.reference)) && overlap access start count <> Outside
  | _ -> Node.write types node

let changes types access node = RegionNodes.any_runs (may_write types access) node
let writes_anything types node = RegionNodes.any_runs (Node.write types) node
