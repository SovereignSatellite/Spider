type t =
  | Integer of int
  | Number of int
  | Sum of Type.t list list
  | Function of Type.t list * Type.t list
  | Array of Type.t
  | Opaque

module Table : Hashtbl.S with type key = t
