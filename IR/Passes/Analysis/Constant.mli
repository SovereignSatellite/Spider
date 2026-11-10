open IR
open Definitions

val bits : 'x link -> int64 option
val of_bits : Type.t -> int64 -> 'x link
val mask : Type.t -> int64
val is : 'x node -> bool
val copy : 'x link -> 'x link
val boolean : bool -> 'x link
