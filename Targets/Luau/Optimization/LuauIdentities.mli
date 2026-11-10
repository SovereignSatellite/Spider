open IR
open Definitions
open Passes
open LuauExtension

val bit_operations : Options.optimization
val views : Options.optimization
val ranges : Options.optimization
val words : Options.optimization

(* Luau: reads fold through their readers. *)
val writes : Options.optimization
val reads : Options.optimization
val fold : Options.t -> x type_arena -> x Readers.table -> x node -> x link array option
