open IR
open Definitions

type 'x t

val create : 'x type_arena -> 'x t
val besides : 'x t -> 'x link list -> 'x node -> 'x node list
