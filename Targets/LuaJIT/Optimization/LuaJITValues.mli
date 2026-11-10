open IR
open Definitions
open LuaJITExtension

val number : x link -> float option

(* LuaJIT: an I64 lowers to an FFI long. *)
val long : x link -> int64 option
val word_type : Type.t -> bool

(* LuaJIT: bit takes integers below 2^51. *)

(** [bit] takes an integer modulo 2^32 only below 2^51 in magnitude, and a range below that bound is exact too. *)
val exact : float

val ones : int
val modular : float -> int option
val modular_constant : x link -> int option

(* LuaJIT: bit gives a word as its signed number, which folding reads. *)
val signed : int -> int
val leading_zeros : int -> int
val bit_binary : x node -> int -> int -> int option
val number_binary : x node -> float -> float -> float option
