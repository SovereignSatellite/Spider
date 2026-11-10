open IR
open Definitions
open LuauExtension

val binary : IntegerBinary.t -> x link -> x link -> x link

(* Luau: undiagnosed; converge or justify. *)

(** Counts each 3-bit field in place, gathers them into 6-bit fields, and sums those by taking their value modulo 63. *)
val count_ones : x link -> x link

val count : IntegerCount.t -> x link -> x link
val compare : IntegerComparison.t -> x link -> x link -> x link
val sign_extend : int -> x link -> x link
