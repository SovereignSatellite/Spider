open IR
open Definitions

let result kind inputs = Link.output (Node.create kind inputs) 0
let extended operation inputs = result (Extension operation) inputs
let constant value = result (F64Constant value) [||]
let integer value = result (I32Constant value) [||]
let to_number value = extended `LuauToNumber [| value |]
let of_number target number = extended (`LuauFromNumber target) [| number |]

(* Luau: bit32 gives a word as its unsigned number, which converts back as it is. *)
let to_i32 value = of_number `I32 value
let retype target value = of_number target (to_number value)
let wrap value = extended `LuauWrap [| value |]
let flip_sign value = extended `LuauBit32Bxor [| to_number value; constant 2147483648.0 |]
let add left right = extended `LuauAdd [| left; right |]
let subtract left right = extended `LuauSubtract [| left; right |]
let multiply left right = extended `LuauMultiply [| left; right |]
let divide left right = extended `LuauDivide [| left; right |]
let negate value = extended `LuauNegate [| value |]
let truncate value = extended `LuauMathModf [| value |]
let less_than left right = extended `LuauLessThan [| left; right |]
let less_than_equal left right = extended `LuauLessThanEqual [| left; right |]
let equal left right = extended `LuauEqual [| left; right |]

(* Luau: bit32 gives a word as its unsigned number. *)
let signed value = subtract (to_number (flip_sign value)) (constant 2147483648.0)
