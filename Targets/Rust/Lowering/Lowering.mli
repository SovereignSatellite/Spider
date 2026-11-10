open IR
open Passes
open RustExtension

val run : Options.t -> x CompilationUnit.t -> unit
val optimizations : Options.optimization list
