open Definitions

val create : 'x type_arena -> Type.t -> 'x link -> 'x link array -> 'x node
val tag : 'x node -> 'x link
val values : 'x node -> 'x link array
