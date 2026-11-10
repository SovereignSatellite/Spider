open IR
open Definitions
open Entities

val references : 'x context -> Type.t -> 'x link list -> 'x link
val table : Wasm.Ast.idx -> Fields.key
val is_null : 'x link -> 'x link
val get : 'x context -> 'x State.t -> Wasm.Ast.idx -> 'x link list -> 'x link list * 'x State.t
val set : 'x context -> 'x State.t -> Wasm.Ast.idx -> 'x link list -> 'x link list * 'x State.t
val size : 'x context -> 'x State.t -> Wasm.Ast.idx -> 'x link list * 'x State.t
val grow : 'x context -> 'x State.t -> Wasm.Ast.idx -> 'x link list -> 'x link list * 'x State.t
val drop : 'x context -> 'x State.t -> int -> 'x link list * 'x State.t
val non_null : 'x context -> 'x link -> 'x link
val call_reference : 'x context -> 'x State.t -> Wasm.Ast.idx -> 'x link list -> 'x link list * 'x State.t

(** Calling a function of another type is undefined, whereas WebAssembly traps. *)
val call_indirect :
  'x context -> 'x State.t -> Wasm.Ast.idx -> Wasm.Ast.idx -> 'x link list -> 'x link list * 'x State.t
