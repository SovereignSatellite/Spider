open Definitions

let create types array = Node.create (ArrayFinalize (Link.type_ types array)) [| array |]
