open Definitions

let create types element_type bytes =
  Node.create (ArrayOf { element_type; bytes; array_type = TypeArena.intern types (Array element_type) }) [||]
