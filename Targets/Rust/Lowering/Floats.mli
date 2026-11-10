open IR
open Definitions
open RustExtension

val convert_signed : x type_arena -> Scalar.number -> x link -> x link
val saturate_signed : x type_arena -> Scalar.integer -> x link -> x link
val binary : NumberBinary.t -> x link -> x link -> x link option
val unary : NumberUnary.t -> x link -> x link
val compare : NumberComparison.t -> x link -> x link -> x link
val retype : [ `F32 | `F64 | `I32 | `I64 ] -> x link -> x link
