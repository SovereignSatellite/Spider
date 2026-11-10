open IR
open Definitions

val signed : int -> int64 -> int64

(** Computing in binary64 and rounding to binary32 gives the correctly rounded binary32 result of every operation folded
    through this, since 53 ≥ 2 × 24 + 2 bits (Figueroa 1995, "When is double rounding innocuous?"). *)
val of_float : Scalar.number -> float -> int64

val leading_zeros : int64 -> int
val integer_binary : IntegerBinary.t -> Scalar.integer -> int64 -> int64 -> int64
val unsigned_to_float : int64 -> float
val integers : Options.optimization
val integer_comparisons : Options.optimization
val numbers : Options.optimization
val number_comparisons : Options.optimization
val conversions : Options.optimization
val sums : Options.optimization
val fold : Options.t -> 'x type_arena -> 'x node -> 'x link array option
