open IR
open Definitions
open RustExtension

(** [usize] is left out because its width depends on the machine the generated Rust is compiled for. *)
val integer_type : x type_arena -> Type.t -> (int * bool) option

val keeps : x type_arena -> Type.t -> Type.t -> bool
val integer : x type_arena -> x link -> int64 option
