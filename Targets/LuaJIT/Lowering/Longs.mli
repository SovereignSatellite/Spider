open IR
open Definitions
open LuaJITExtension

(* LuaJIT: an I64 lowers to an FFI long. *)

(** [int64_t] has LuaJIT's arithmetic and [bit]'s bitwise operations. *)
val binary : x type_arena -> IntegerBinary.t -> x link -> x link -> x link

(* LuaJIT: an I64 lowers to an FFI long. *)
val narrow : x link -> x link
val count : IntegerCount.t -> x link -> x link option
val compare : x type_arena -> IntegerComparison.t -> x link -> x link -> x link
val widen : x link -> x link
val sign_extend : int -> x link -> x link

(* LuaJIT: an I64 lowers to an FFI long. *)
val convert_signed : x link -> x link
val convert_unsigned : x type_arena -> x link -> x link

(** For values from −2^63 below 2^64, as a number cast to [uint64_t] below zero gives its two's complement. *)
val truncated : x type_arena -> x link -> x link

(** Bits read from memory are read as an F64 unless their high word's exponent is all ones; then they are an infinity,
    read as one, or by their fraction a NaN, LuaJIT's of the high word's sign. *)
val into_f64 : x type_arena -> x link -> x link
