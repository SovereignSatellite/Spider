type t =
  | IntegerBinary of Scalar.integer * IntegerBinary.t
  | IntegerCount of Scalar.integer * IntegerCount.t
  | IntegerComparison of Scalar.integer * IntegerComparison.t
  | NumberBinary of Scalar.number * NumberBinary.t
  | NumberUnary of Scalar.number * NumberUnary.t
  | NumberComparison of Scalar.number * NumberComparison.t
  | Conversion of Conversion.t

val name : t -> string
val operand : t -> Scalar.t
val result_type : t -> Type.t
val binary : t -> bool
val commutative : t -> bool

(** The [Outcomes] flags of the left operand against the right on which a comparison is true, or 0 for any other
    operator. *)
val holds : t -> int
