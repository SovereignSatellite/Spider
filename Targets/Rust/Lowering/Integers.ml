open IR
open Numbers

(** [wrapping_shl], [wrapping_shr] and the rotations take a [u32] amount modulo the width, so an I64 amount cut to 32
    bits shifts the same. *)
let amount width value = match width with `I32 -> value | `I64 -> cast Type.i32 value

let signed_operands types width left right =
  let signed = signed_of types width in
  (cast signed left, cast signed right)

let through_signed types width operate left right =
  let left_signed, right_signed = signed_operands types width left right in
  cast (Type.of_scalar width) (extended operate [| left_signed; right_signed |])

let shift_right_signed types width value shift =
  let signed = signed_of types width in
  cast (Type.of_scalar width) (extended (`RustSignedWrappingShr signed) [| cast signed value; amount width shift |])

let binary types width (operator : IntegerBinary.t) left right =
  let value_type = Type.of_scalar width in
  let shifted shift = extended shift [| left; amount width right |] in
  let both operation = extended operation [| left; right |] in
  match operator with
  | Add -> both `RustWrappingAdd
  | Subtract -> both `RustWrappingSub
  | Multiply -> both `RustWrappingMul
  | And -> both `RustBitAnd
  | Or -> both `RustBitOr
  | ExclusiveOr -> both `RustBitXor
  | ShiftLeft -> shifted (`RustWrappingShl value_type)
  | ShiftRightS -> shift_right_signed types width left right
  | ShiftRightU -> shifted (`RustWrappingShr value_type)
  | RotateLeft -> shifted (`RustRotateLeft value_type)
  | RotateRight -> shifted (`RustRotateRight value_type)
  | DivideU -> both `RustDivide
  | RemainderU -> both `RustRemainder
  | DivideS -> through_signed types width `RustDivide left right
  | RemainderS -> through_signed types width `RustWrappingRem left right

let counted (operator : IntegerCount.t) value =
  match operator with
  | LeadingZeros -> extended `RustLeadingZeros [| value |]
  | TrailingZeros -> extended `RustTrailingZeros [| value |]
  | CountOnes -> extended `RustCountOnes [| value |]

let count width operator value =
  match width with `I32 -> counted operator value | `I64 -> cast Type.i64 (counted operator value)

let compare types width (operator : IntegerComparison.t) left right =
  match operator with
  | Equal -> extended `RustEqual [| left; right |]
  | LessThanU -> extended `RustLessThan [| left; right |]
  | LessThanS ->
      let left_signed, right_signed = signed_operands types width left right in
      extended `RustLessThan [| left_signed; right_signed |]

let sign_extend types width from value =
  value
  |> cast (Type.of_scalar from)
  |> cast (signed_of types from)
  |> cast (signed_of types width)
  |> cast (Type.of_scalar width)
