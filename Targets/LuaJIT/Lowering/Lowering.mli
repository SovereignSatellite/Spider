open IR
open Passes
open LuaJITExtension

val run : Options.t -> x CompilationUnit.t -> unit
val optimizations : Options.optimization list
