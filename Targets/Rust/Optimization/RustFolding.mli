open IR
open Definitions
open Passes
open RustExtension

val casts : Options.optimization
val sums : Options.optimization
val fold : Options.t -> x type_arena -> x node -> x link array option
