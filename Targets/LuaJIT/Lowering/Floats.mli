(* LuaJIT: the runtime native code computes basic F32 arithmetic. *)
(** F32 arithmetic other than addition, subtraction, multiplication, division and square root, which the runtime's
    native code computes, runs on F64s rounded back to F32. *)

open IR
open Definitions
open LuaJITExtension

(* LuaJIT: NaN tagging reads other NaNs as other values. *)

(** A NaN's bits become those of LuaJIT's NaN of its sign before the F32 is a number, as LuaJIT reads another NaN as
    another value. *)
val widen : x type_arena -> x link -> x link

val narrow : x link -> x link
val unary : x type_arena -> Scalar.number -> NumberUnary.t -> x link -> x link option
val binary : x type_arena -> Scalar.number -> NumberBinary.t -> x link -> x link -> x link option

(* LuaJIT: NaN tagging reads other NaNs as other values. *)
val compare : x type_arena -> Scalar.number -> NumberComparison.t -> x link -> x link -> x link
val convert_signed : Scalar.number -> x link -> x link option
val convert_unsigned : x type_arena -> Scalar.number -> x link -> x link option
val saturate_signed : x type_arena -> Scalar.integer -> x link -> x link
val saturate_unsigned : x type_arena -> Scalar.integer -> x link -> x link
