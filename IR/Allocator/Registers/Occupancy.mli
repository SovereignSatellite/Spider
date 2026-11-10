open Intervals

type holder = { holder_id : int; interval : interval; register : int; mutable range : int; mutable live : bool }
type t

val create : int -> t
val advance : t -> int -> unit
val count : t -> int
val active : t -> int -> holder option
val is_active : t -> int -> bool
val iter_inactive : (holder -> unit) -> t -> unit
val unheld : t -> int option
val add_register : t -> int
val take : t -> interval -> int -> unit
val remove : t -> holder -> unit
