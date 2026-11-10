type t = int

let of_scalar scalar = match scalar with `I8 -> 0 | `I16 -> 1 | `I32 -> 2 | `I64 -> 3 | `F32 -> 4 | `F64 -> 5

let scalar described =
  match described with
  | 0 -> Some `I8
  | 1 -> Some `I16
  | 2 -> Some `I32
  | 3 -> Some `I64
  | 4 -> Some `F32
  | 5 -> Some `F64
  | _ -> None

let i8 = of_scalar `I8
let i16 = of_scalar `I16
let i32 = of_scalar `I32
let i64 = of_scalar `I64
let f32 = of_scalar `F32
let f64 = of_scalar `F64
let unit = 6
let boolean = 7
let bytes = 8
let opaque = 9

let bits described =
  match scalar described with
  | Some scalar -> Scalar.bits scalar
  | None -> invalid_arg "Only integers and numbers have a bit width."

let numbered index = index
