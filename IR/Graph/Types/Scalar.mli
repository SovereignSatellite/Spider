type integer = [ `I32 | `I64 ]
type number = [ `F32 | `F64 ]
type t = [ `I8 | `I16 | integer | number ]

val bits : [< t ] -> int
val name : [< t ] -> string
