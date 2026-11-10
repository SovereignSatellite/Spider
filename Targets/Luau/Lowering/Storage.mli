open IR
open Definitions
open Passes
open LuauExtension

val omission : Options.optimization
val length : x type_arena -> x link -> x link
val get : Options.t -> x type_arena -> x node -> x link
val set : Options.t -> x type_arena -> x node -> x link
val fill : Options.t -> x type_arena -> x node -> x link
val copy : Options.t -> x type_arena -> x node -> x link
val load : x node -> IR_Memory.load -> x link
val store : x node -> IR_Memory.store -> x link

(* Luau: table.create fills, and buffer.create zeroes. *)

(** [table.create] fills what it allocates with the initial element, and [buffer.create] with zeros. *)
val allocate : x type_arena -> x node -> x link
