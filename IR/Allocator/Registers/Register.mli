type t = Fixed of int | Overflow of int

module Map : Map.S with type key = t
