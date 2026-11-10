open IR
open Definitions
open Passes
open LuaJITExtension

val bit_operations : Options.optimization
val views : Options.optimization
val ranges : Options.optimization
val longs : Options.optimization

(* LuaJIT: the bits of an FFI long can hold an F64. *)
val f64_bits : Options.optimization
val writes : Options.optimization
val fold : Options.t -> x type_arena -> x node -> x link array option
