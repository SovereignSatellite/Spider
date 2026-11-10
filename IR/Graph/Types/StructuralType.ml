type t =
  | Integer of int
  | Number of int
  | Sum of Type.t list list
  | Function of Type.t list * Type.t list
  | Array of Type.t
  | Opaque

module Table = Hashtbl.Make (struct
  type nonrec t = t

  let equal = ( = )
  let hash = Hashtbl.hash_param 256 256
end)
