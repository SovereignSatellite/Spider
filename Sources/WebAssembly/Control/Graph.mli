type handling = Try of outlined | Catch of int

and statement =
  | Operation of Wasm.Ast.instr' * int list * int list
  | Move of int * int
  | Assign of int * int
  | Exception of handling * int list * int list

(** The selector's value is the index of the successor a block continues to, always in range. *)
and basic_block = {
  statements : statement list;
  selector : int option;
  mutable predecessors : int list;
  mutable successors : int list;
}

and outlined = { graph : basic_block Dynarray.t; outputs : int array; boundaries : int array array }

type t = basic_block Dynarray.t

val block : t -> int -> basic_block
val count : t -> int
val no_operation : t -> int
val assignment : t -> int -> int -> int
val selection : t -> int -> int
val assigns : t -> int -> int -> bool
val add_edge : t -> int -> int -> unit
val index_of : int -> int list -> int
val replace_edge : t -> int -> int -> int -> unit
val add_predecessors : t -> unit
val compact : t -> t
val is_branch_start : t -> int -> bool
val is_branch_end : t -> int -> bool
val is_arm_entry : t -> int -> bool
val is_loop_head : t -> int -> bool
val branch_end : t -> int -> int option
val repeat_start : t -> int -> int option
