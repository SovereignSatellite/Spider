open IR
open Definitions
open LuauExtension

val result : x kind -> x link array -> x link
val extended : x -> x link array -> x link
val constant : float -> x link
val integer : int -> x link
val to_number : x link -> x link
val of_number : Scalar.t -> x link -> x link

(* Luau: bit32 gives a word as its unsigned number, which converts back as it is. *)
val to_i32 : x link -> x link
val retype : Scalar.t -> x link -> x link

(* Luau: bit32 wraps any number below 2^63. *)

(** Modulo 2^32 of the number truncated toward zero, as [bit32] takes any number of magnitude below 2^63. *)
val wrap : x link -> x link

val flip_sign : x link -> x link
val add : x link -> x link -> x link
val subtract : x link -> x link -> x link
val multiply : x link -> x link -> x link
val divide : x link -> x link -> x link
val negate : x link -> x link
val truncate : x link -> x link
val less_than : x link -> x link -> x link
val less_than_equal : x link -> x link -> x link
val equal : x link -> x link -> x link

(* Luau: bit32 gives a word as its unsigned number. *)
val signed : x link -> x link
