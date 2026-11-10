open Definitions

let create types box = Node.create (BoxFinalize (Link.type_ types box)) [| box |]
