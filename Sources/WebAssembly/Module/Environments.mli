open IR
open Definitions
open Entities

(** A function's own field is left out, since its environment is the one it takes. *)
val captures : Wasm.Ast.module_ -> Fields.key list list

val build : 'x context -> Fields.key list list -> Type.t list -> 'x link Fields.t -> 'x link Fields.t * 'x link list
