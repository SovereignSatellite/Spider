open IR
open Definitions

type 'x t =
  | Bits of int64
  | Host of Type.t list * string * string
  | Identifier of string
  | Variant of int * Type.t
  | Literal of Type.t * string
  | Offset of int
  | Body of 'x region
  | Extended of 'x
  | Types of Type.t list

let non_constant_data types node =
  match (node.kind, Node.array_access types node) with
  | Extension extended, _ -> Some (Extended extended)
  | Import { namespace; identifier; _ }, _ -> Some (Host (Node.output_types types node, namespace, identifier))
  | Export identifier, _ -> Some (Identifier identifier)
  | SumNew { variant; sum_type }, _ -> Some (Variant (variant, sum_type))
  | ArrayOf { element_type; bytes; _ }, _ -> Some (Literal (element_type, bytes))
  | _, Some access -> Some (Offset access.offset)
  | RegionReference body, None -> Some (Body body)
  | (RegionArguments _ | RegionResults _), None -> None
  | _, None -> Some (Types (Node.output_types types node))

let of_node types node =
  match Node.bits node.kind with Some bits -> Some (Bits bits) | None -> non_constant_data types node

let equal left right =
  match (left, right) with Body left, Body right -> left == right | Body _, _ -> false | _ -> left = right

let hash data =
  match data with
  | Body body -> body.region_id
  | Literal (element_type, _) -> Hashtbl.hash (element_type :> int)
  | _ -> Hashtbl.hash data

let idempotent types node = not (RegionNodes.any_runs (Node.unrepeatable types) node)
