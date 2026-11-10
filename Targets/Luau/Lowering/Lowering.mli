open IR
open Passes
open LuauExtension

val run : Options.t -> x CompilationUnit.t -> unit
val optimizations : Options.optimization list
