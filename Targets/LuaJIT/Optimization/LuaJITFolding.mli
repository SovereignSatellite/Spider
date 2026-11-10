open IR
open Definitions
open Passes
open LuaJITExtension

val bit_operations : Options.optimization

(* LuaJIT: an I64 lowers to an FFI long. *)
val arithmetic : Options.optimization
val longs : Options.optimization
val quotients : Options.optimization
val views : Options.optimization
val fold : Options.t -> x type_arena -> x node -> x link array option
