(** A Boolean or tag computed only from constants and comparisons of the same two values is a table over the outcomes
    less, equal, greater and unordered. It becomes a constant, one of those comparisons with its operands as they are or
    swapped, or that comparison's complement where that leaves fewer comparisons. *)

open IR
open Definitions

val of_operation : 'x type_arena -> 'x node -> 'x link option
val of_match : 'x type_arena -> 'x node -> int -> 'x link option
