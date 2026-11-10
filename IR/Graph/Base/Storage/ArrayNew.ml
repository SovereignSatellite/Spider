open Definitions

let create types length initial =
  let array_type = TypeArena.intern types (Array (Link.type_ types initial)) in
  let allocated = TypeArena.intern types (Sum [ []; [ array_type ] ]) in
  Node.create (ArrayNew { array_type; allocated }) [| length; initial |]

let array_type allocation =
  match allocation.kind with ArrayNew { array_type; _ } -> array_type | _ -> invalid_arg "Expected an Array.New."

let or_throw types payload allocation =
  let array_type = array_type allocation in
  Link.output
    (RegionMatch.create types (Link.output allocation 0) [ array_type ] (fun variant fields ->
         [| (if variant = 1 then fields.(0) else Link.output (Throw.create payload [ array_type ]) 0) |]))
    0
