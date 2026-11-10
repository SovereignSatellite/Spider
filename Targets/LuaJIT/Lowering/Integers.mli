open IR
open Definitions
open LuaJITExtension

val binary : IntegerBinary.t -> x link -> x link -> x link
val count_ones : x link -> x link

(* LuaJIT: bit has no countlz or countrz, so those stay prefab calls. *)
val count : IntegerCount.t -> x link -> x link option
val compare : IntegerComparison.t -> x link -> x link -> x link
val sign_extend : int -> x link -> x link
