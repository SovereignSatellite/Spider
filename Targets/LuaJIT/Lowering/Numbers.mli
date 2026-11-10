open IR
open Definitions
open LuaJITExtension

val result : x kind -> x link array -> x link
val extended : x -> x link array -> x link
val constant : float -> x link
val integer : int -> x link
val to_number : x link -> x link
val of_number : Scalar.t -> x link -> x link
val retype : Scalar.t -> x link -> x link

(* LuaJIT: bit.tobit wraps only integers below 2^51. *)

(** Modulo 2^32 of an integer below 2^51 in magnitude, which [bit.tobit] takes. *)
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

(* LuaJIT: bit gives a word as its signed number, and the FFI casts numbers to longs. *)
val unsigned : x link -> x link
val cast : Type.t -> x link -> x link
val to_long : x link -> x link
