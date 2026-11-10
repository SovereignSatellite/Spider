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

val commutative : t -> bool
val name : t -> string
