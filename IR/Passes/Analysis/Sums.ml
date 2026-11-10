open IR
open Definitions

let shape node =
  match node.kind with
  | SumNew { variant; _ } -> Some ([ variant ], 0)
  | SumFromI32 { variants; _ } -> Some (variants, 1)
  | _ -> None

let fields_from node first = Array.sub node.inputs first (Array.length node.inputs - first)

let built node =
  Option.map
    (fun (variants, first) ->
      let fields = fields_from node first in
      List.map (fun variant -> (variant, fields)) variants)
    (shape node)

let start node variant =
  match shape node with Some (variants, first) when List.mem variant variants -> Some first | _ -> None

let builds node variant = Option.is_some (start node variant)
let producing node = match shape node with Some (variants, _) -> variants <> [] | None -> false
let fields node variant = Option.map (fields_from node) (start node variant)
let field node variant index = Option.map (fun first -> node.inputs.(first + index)) (start node variant)
