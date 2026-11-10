type t =
  | Add
  | Subtract
  | Multiply
  | And
  | Or
  | ExclusiveOr
  | ShiftLeft
  | ShiftRightS
  | ShiftRightU
  | RotateLeft
  | RotateRight
  | DivideS
  | DivideU
  | RemainderS
  | RemainderU

let commutative operator =
  match operator with
  | Add | Multiply | And | Or | ExclusiveOr -> true
  | Subtract | ShiftLeft | ShiftRightS | ShiftRightU | RotateLeft | RotateRight | DivideS | DivideU | RemainderS
  | RemainderU ->
      false

let name operator =
  match operator with
  | Add -> "Add"
  | Subtract -> "Subtract"
  | Multiply -> "Multiply"
  | And -> "And"
  | Or -> "Or"
  | ExclusiveOr -> "ExclusiveOr"
  | ShiftLeft -> "ShiftLeft"
  | ShiftRightS -> "ShiftRightS"
  | ShiftRightU -> "ShiftRightU"
  | RotateLeft -> "RotateLeft"
  | RotateRight -> "RotateRight"
  | DivideS -> "DivideS"
  | DivideU -> "DivideU"
  | RemainderS -> "RemainderS"
  | RemainderU -> "RemainderU"
