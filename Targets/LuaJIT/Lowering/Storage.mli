open IR
open Definitions
open Passes
open LuaJITExtension

val omission : Options.optimization
val length : x type_arena -> x link -> x link
val get : Options.t -> x type_arena -> x node -> x link
val set : Options.t -> x type_arena -> x node -> x link
val fill : Options.t -> x type_arena -> x node -> x link
val copy : Options.t -> x type_arena -> x node -> x link

(* LuaJIT: FFI accesses are unchecked, so loads and stores lower their bounds checks. *)
val load : Options.t -> x type_arena -> x node -> IR_Memory.load -> x link
val store : Options.t -> x type_arena -> x node -> IR_Memory.store -> x link

(** Neither [table.new] nor [ffi.new] fills what it allocates with the initial element. *)
val allocate : x type_arena -> x node -> x link
