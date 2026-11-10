type t = Negate | Absolute | SquareRoot | RoundUp | RoundDown | Truncate | Nearest

let name operator =
  match operator with
  | Negate -> "Negate"
  | Absolute -> "Absolute"
  | SquareRoot -> "SquareRoot"
  | RoundUp -> "RoundUp"
  | RoundDown -> "RoundDown"
  | Truncate -> "Truncate"
  | Nearest -> "Nearest"
