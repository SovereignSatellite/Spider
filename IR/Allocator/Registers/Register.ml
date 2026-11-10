type t = Fixed of int | Overflow of int

module Map = Map.Make (struct
  type nonrec t = t

  let compare = compare
end)
