open IR
open Definitions
open LuaJITExtension

(* LuaJIT: bit takes integers below 2^51. *)

(** The F64 is an integer from [least] to [greatest], of magnitude below 2^51, and never −0. *)
type range = { least : float; greatest : float }

(** Results stay valid through a pass run, as a rewrite replaces a value only with one equal wherever the replaced value
    is defined. *)
type t

val create : x type_arena -> t
val types : t -> x type_arena
val ones : t -> x link -> int
val word_ones : t -> x link -> int option
val operand_ones : t -> x link -> int
val range : t -> x link -> range option
