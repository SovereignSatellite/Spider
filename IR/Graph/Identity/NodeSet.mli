open Definitions

type 'x t

val create : int -> 'x t
val mem : 'x t -> 'x node -> bool
val add : 'x t -> 'x node -> bool
val remove : 'x t -> 'x node -> unit
val clear : 'x t -> unit
val of_list : 'x node list -> 'x t
