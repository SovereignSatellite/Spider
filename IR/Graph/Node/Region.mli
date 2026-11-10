open Definitions

type 'x t = 'x region

val parameter : 'x t -> int -> 'x link
val parameters : 'x t -> 'x link array
val parameter_types : 'x t -> Type.t list
val result_types : 'x t -> Type.t list
val result_count : 'x t -> int
val result_type : 'x t -> int -> Type.t
val define : 'x t -> 'x link array -> unit
val create : ?build:('x t -> 'x link array) -> 'x type_arena -> Type.t list -> Type.t list -> 'x t
val redefine : 'x t -> Type.t list -> Type.t list -> ('x t -> 'x link array) -> unit
