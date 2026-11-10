open IR
open Definitions
open Entities

val lift :
  ([> IR_Memory.t ] as 'a) context -> 'a State.t -> Wasm.Ast.instr' -> 'a link list -> 'a link list * 'a State.t

val accesses : Wasm.Ast.instr' -> (Fields.key * Access.t) list

(** What may throw to a section's clauses, which no trap reaches. *)
val throws : Wasm.Ast.instr' -> bool
