open Definitions

type ('x, 'value) t

val create : int -> ('x, 'value) t
val find_opt : ('x, 'value) t -> 'x link -> 'value option
val find : ('x, 'value) t -> 'x link -> 'value
val mem : ('x, 'value) t -> 'x link -> bool
val replace : ('x, 'value) t -> 'x link -> 'value -> unit
val length : ('x, 'value) t -> int
