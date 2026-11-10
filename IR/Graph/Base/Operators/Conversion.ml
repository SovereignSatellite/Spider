type t =
  | Widen of [ `I8 | `I16 | `I32 | `F32 ]
  | Narrow of [ `I8 | `I16 | `I32 | `F32 ]
  | SignExtend of Scalar.integer * [ `I8 | `I16 | `I32 ]
  | ConvertSigned of Scalar.number
  | ConvertUnsigned of Scalar.number
  | SaturateSigned of Scalar.integer
  | SaturateUnsigned of Scalar.integer
  | Retype of [ Scalar.integer | Scalar.number ]

let wider scalar = match scalar with `I8 | `I16 -> `I32 | `I32 -> `I64 | `F32 -> `F64

let source conversion =
  match conversion with
  | Widen source -> (source :> Scalar.t)
  | Narrow target -> wider target
  | SignExtend (width, _) -> (width :> Scalar.t)
  | ConvertSigned _ | ConvertUnsigned _ -> `I64
  | SaturateSigned _ | SaturateUnsigned _ -> `F64
  | Retype `I32 -> `F32
  | Retype `I64 -> `F64
  | Retype `F32 -> `I32
  | Retype `F64 -> `I64

let target conversion =
  match conversion with
  | Widen source -> wider source
  | Narrow target -> (target :> Scalar.t)
  | SignExtend (width, _) | SaturateSigned width | SaturateUnsigned width -> (width :> Scalar.t)
  | ConvertSigned number | ConvertUnsigned number -> (number :> Scalar.t)
  | Retype target -> (target :> Scalar.t)

let name conversion =
  let target = Scalar.name (target conversion) in
  match conversion with
  | Widen _ -> "WidenTo" ^ target
  | Narrow _ -> "NarrowTo" ^ target
  | SignExtend (_, from) -> "SignExtend" ^ string_of_int (Scalar.bits from)
  | ConvertSigned _ -> "ConvertSTo" ^ target
  | ConvertUnsigned _ -> "ConvertUTo" ^ target
  | SaturateSigned width -> "SaturateToS" ^ string_of_int (Scalar.bits width)
  | SaturateUnsigned width -> "SaturateToU" ^ string_of_int (Scalar.bits width)
  | Retype _ -> "RetypeTo" ^ target
