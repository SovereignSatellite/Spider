open IR
open Definitions
open LuauExtension

val number : x link -> float option

(* Luau: an I64 lowers to two I32 words. *)
val word : x link -> int option
val word_type : Type.t -> bool

(* Luau: bit32 takes integers below 2^53. *)

(** [bit32] takes an integer modulo 2^32 below 2^53 in magnitude, where every integer is exact. *)
val exact : float

val ones : int
val modular : float -> int option
val modular_constant : x link -> int option
val leading_zeros : int -> int
val bit32_binary : x node -> int -> int -> int option

(* Luau: bit32 counts leading and trailing zeros. *)
val bit32_unary : x node -> int -> int option
val number_binary : x node -> float -> float -> float option
