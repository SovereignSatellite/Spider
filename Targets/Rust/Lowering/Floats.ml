open IR
open Numbers

let convert_signed types target value = value |> cast (signed_of types `I64) |> cast (Type.of_scalar target)
let saturate_signed types target value = value |> cast (signed_of types target) |> cast (Type.of_scalar target)

let binary (operator : NumberBinary.t) left right =
  let both operation = Some (extended operation [| left; right |]) in
  match operator with
  | Add -> both `RustAdd
  | Subtract -> both `RustSubtract
  | Multiply -> both `RustMultiply
  | Divide -> both `RustDivide
  | CopySign -> both `RustCopysign
  | Minimum | Maximum -> None

let unary (operator : NumberUnary.t) value =
  extended
    (match operator with
    | Negate -> `RustNegate
    | Absolute -> `RustAbs
    | SquareRoot -> `RustSqrt
    | RoundUp -> `RustCeil
    | RoundDown -> `RustFloor
    | Truncate -> `RustTrunc
    | Nearest -> `RustRoundTiesEven)
    [| value |]

let compare (operator : NumberComparison.t) left right =
  extended
    (match operator with Equal -> `RustEqual | LessThan -> `RustLessThan | LessThanEqual -> `RustLessThanEqual)
    [| left; right |]

let retype target value =
  let target_type = Type.of_scalar (target :> Scalar.t) in
  match target with
  | #Scalar.number -> extended (`RustFromBits target_type) [| value |]
  | #Scalar.integer -> extended (`RustToBits target_type) [| value |]
