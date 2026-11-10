open Definitions

val create : 'x type_arena -> 'x link -> Type.t list -> (int -> 'x link array -> 'x link array) -> 'x node
val branches : 'x node -> 'x region array
val selector : 'x node -> 'x link
val branch : 'x node -> int -> 'x region
val field : 'x node -> int -> int -> 'x link
val fields : 'x node -> int -> 'x link array
val redefine : 'x type_arena -> 'x node -> 'x link -> Type.t list -> (int -> 'x link array -> 'x link array) -> unit
