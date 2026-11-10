open IR
open Definitions
open RustExtension

val binary : x type_arena -> Scalar.integer -> IntegerBinary.t -> x link -> x link -> x link
val count : Scalar.integer -> IntegerCount.t -> x link -> x link
val compare : x type_arena -> Scalar.integer -> IntegerComparison.t -> x link -> x link -> x link
val sign_extend : x type_arena -> Scalar.integer -> [ `I16 | `I32 | `I8 ] -> x link -> x link
