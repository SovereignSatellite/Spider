open Definitions

let array_set types array index value = Node.create (ArraySet (Link.type_ types array)) [| array; index; value |]

let array_fill types array start count value =
  Node.create (ArrayFill (Link.type_ types array)) [| array; start; count; value |]

let array_copy types destination destination_start source source_start count =
  Node.create
    (ArrayCopy (Link.type_ types destination))
    [| destination; destination_start; source; source_start; count |]

let box_set types box contents = Node.create (BoxSet (Link.type_ types box)) [| box; contents |]
