(** Rust reads and writes memory in the host's byte order, so this assumes a little-endian host, as memory is. *)

open IR
open Definitions
open Passes
open RustExtension

val omission : Options.optimization
val get : Options.t -> x type_arena -> x node -> x link
val set : Options.t -> x type_arena -> x node -> x link
val size : x type_arena -> x node -> x link
val fill : Options.t -> x type_arena -> x node -> x link
val copy : Options.t -> x type_arena -> x node -> x link
val load : Options.t -> x type_arena -> x node -> IR_Memory.load -> x link
val store : Options.t -> x type_arena -> x node -> IR_Memory.store -> x link
val allocate : x type_arena -> x node -> x link
