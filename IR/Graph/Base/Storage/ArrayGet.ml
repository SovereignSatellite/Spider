open Definitions

let create types array index =
  Node.create (ArrayGet (TypeArena.element types (Link.type_ types array))) [| array; index |]
