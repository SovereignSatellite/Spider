open IR
open Definitions
open RustExtension

val result : x kind -> x link array -> x link
val extended : x -> x link array -> x link
val cast : Type.t -> x link -> x link
val signed_of : x type_arena -> [< Scalar.t ] -> Type.t
val usize : x type_arena -> x link -> x link
