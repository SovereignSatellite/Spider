(* Luau: F32 arithmetic runs on F64s. *)
(** F32 arithmetic runs on F64s rounded back to F32, which rounds addition, subtraction, multiplication, division and
    square root correctly since 53 ≥ 2 × 24 + 2 (Figueroa, 1995). *)

open IR
open Definitions
open LuauExtension

val widen : x link -> x link
val narrow : x link -> x link

(* Luau: math.round rounds halves away from zero. *)

(** Luau's [math.round] rounds halves away from zero, so rounding to nearest even is composed. *)
val unary : x type_arena -> Scalar.number -> NumberUnary.t -> x link -> x link option

val binary : x type_arena -> Scalar.number -> NumberBinary.t -> x link -> x link -> x link option
val compare : Scalar.number -> NumberComparison.t -> x link -> x link -> x link
val convert_signed : Scalar.number -> x link -> x link option
val convert_unsigned : Scalar.number -> x link -> x link option
val saturate_signed : x type_arena -> Scalar.integer -> x link -> x link
val saturate_unsigned : x type_arena -> Scalar.integer -> x link -> x link
