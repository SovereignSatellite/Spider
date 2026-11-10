open Definitions

type ('x, 'value) t

val create : int -> ('x, 'value) t
val find_opt : ('x, 'value) t -> 'x node -> 'value option
val find : ('x, 'value) t -> 'x node -> 'value
val mem : ('x, 'value) t -> 'x node -> bool
val replace : ('x, 'value) t -> 'x node -> 'value -> unit
val length : ('x, 'value) t -> int
