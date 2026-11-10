open Definitions

type 'x t = 'x link

val output : 'x node -> int -> 'x t
val type_ : 'x type_arena -> 'x t -> Type.t
val types : 'x type_arena -> 'x t array -> Type.t list
val equal : 'x t -> 'x t -> bool
val distinct : 'x t list -> 'x t array
val index_of : 'x t array -> 'x t -> int
