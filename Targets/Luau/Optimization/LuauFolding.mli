open IR
open Definitions
open Passes
open LuauExtension

val bit_operations : Options.optimization
val arithmetic : Options.optimization
val quotients : Options.optimization
val views : Options.optimization
val fold : Options.t -> x type_arena -> x node -> x link array option
