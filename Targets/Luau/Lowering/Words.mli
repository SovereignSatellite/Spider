open IR
open Definitions
open LuauExtension

(* Luau: an I64 lowers to two I32 words. *)
val narrow : x link -> x link
val join : x link -> x link -> x link
val word_types : Type.t list
val binary : x type_arena -> IntegerBinary.t -> x link -> x link -> x link option
val count : x type_arena -> IntegerCount.t -> x link -> x link
val compare : x type_arena -> IntegerComparison.t -> x link -> x link -> x link
val widen : x link -> x link
val sign_extend : int -> x link -> x link

(* Luau: an I64 lowers to two I32 words. *)
val convert : (x link -> x link) -> x link -> x link

(** For a magnitude below 2^63: taking away the low word leaves a multiple of 2^32 and the fraction, which the wrap
    drops. *)
val truncated_signed : x link -> x link array

(** For a positive value below 2^64. *)
val truncated_unsigned : x link -> x link array
