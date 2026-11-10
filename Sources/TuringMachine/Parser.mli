open Instruction

type syntax_error = UnmatchedOpen of int | UnmatchedClose of int

val parse : string -> (t list, syntax_error) result
