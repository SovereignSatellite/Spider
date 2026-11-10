open IR
open Definitions

type 'x rule = 'x node -> 'x link array option

val rules : Options.t -> 'x type_arena -> 'x Readers.table -> 'x rule list
val first : 'x rule list -> 'x node -> 'x link array option
val run : ('x Readers.table -> 'x rule list) -> 'x Readers.table -> bool
