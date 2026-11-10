(** A widening names its source and a narrowing its target, the other end being [wider]. Integers convert to numbers
    only from an I64, and saturate to integers only from an F64. *)
type t =
  | Widen of [ `I8 | `I16 | `I32 | `F32 ]
  | Narrow of [ `I8 | `I16 | `I32 | `F32 ]
  | SignExtend of Scalar.integer * [ `I8 | `I16 | `I32 ]
  | ConvertSigned of Scalar.number
  | ConvertUnsigned of Scalar.number
  | SaturateSigned of Scalar.integer
  | SaturateUnsigned of Scalar.integer
  | Retype of [ Scalar.integer | Scalar.number ]

val source : t -> Scalar.t
val target : t -> Scalar.t
val name : t -> string
