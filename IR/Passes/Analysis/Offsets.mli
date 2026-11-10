open IR
open Definitions

val offset_of : 'x node -> (Scalar.integer * int64) option
val split : 'x link -> 'x link option * int64
val offset : Scalar.integer -> 'x link -> int64 -> 'x link
