open Definitions

val create : 'x type_arena -> 'x link -> 'x link -> 'x node
val array_type : 'x node -> Type.t
val or_throw : 'x type_arena -> 'x link array -> 'x node -> 'x link
