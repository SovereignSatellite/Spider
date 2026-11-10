open Definitions

let create types sum_type tag values =
  let variants = TypeArena.variants types sum_type in
  let value_types = Link.types types values in
  let fitting =
    List.concat (List.mapi (fun variant fields -> if fields = value_types then [ variant ] else []) variants)
  in
  Node.create
    (SumFromI32 { sum_type; variants = fitting; tagged = List.length variants >= 2 })
    (Array.append [| tag |] values)

let tag sum = sum.inputs.(0)
let values sum = Array.sub sum.inputs 1 (Array.length sum.inputs - 1)
