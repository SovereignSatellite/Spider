open Definitions

let create types box = Node.create (BoxGet (TypeArena.contents types (Link.type_ types box))) [| box |]
