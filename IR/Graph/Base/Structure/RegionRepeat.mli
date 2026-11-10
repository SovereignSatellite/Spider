open Definitions

val step_type : 'x type_arena -> Type.t list -> Type.t list -> Type.t
val create : 'x type_arena -> 'x link array -> Type.t list -> ('x link array -> 'x link) -> 'x node
val body : 'x node -> 'x region
val step : 'x node -> 'x link
val redefine : 'x type_arena -> 'x node -> 'x link array -> Type.t list -> ('x link array -> 'x link) -> unit
