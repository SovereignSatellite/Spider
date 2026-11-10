open Definitions

type ('x, 'value) t

val create : int -> ('x, 'value) t
val find_opt : ('x, 'value) t -> 'x region -> 'value option
val find : ('x, 'value) t -> 'x region -> 'value
val mem : ('x, 'value) t -> 'x region -> bool
val replace : ('x, 'value) t -> 'x region -> 'value -> unit
val remove : ('x, 'value) t -> 'x region -> unit
val length : ('x, 'value) t -> int
