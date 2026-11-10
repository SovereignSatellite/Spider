open Definitions

type 'x t

val create : int -> 'x t
val mem : 'x t -> 'x region -> bool
val add : 'x t -> 'x region -> bool
val remove : 'x t -> 'x region -> unit
val length : 'x t -> int
val union : 'x t -> 'x t -> unit
val except : 'x t -> 'x t -> unit
