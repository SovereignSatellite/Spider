type integer = [ `I32 | `I64 ]
type number = [ `F32 | `F64 ]
type t = [ `I8 | `I16 | integer | number ]

let bits scalar = match scalar with `I8 -> 8 | `I16 -> 16 | `I32 | `F32 -> 32 | `I64 | `F64 -> 64

let name scalar =
  match scalar with `I8 -> "I8" | `I16 -> "I16" | `I32 -> "I32" | `I64 -> "I64" | `F32 -> "F32" | `F64 -> "F64"
