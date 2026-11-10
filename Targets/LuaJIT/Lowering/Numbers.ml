open IR
open Definitions

let result kind inputs = Link.output (Node.create kind inputs) 0
let extended operation inputs = result (Extension operation) inputs
let constant value = result (F64Constant value) [||]
let integer value = result (I32Constant value) [||]
let to_number value = extended `LuaJITToNumber [| value |]
let of_number target number = extended (`LuaJITFromNumber target) [| number |]
let retype target value = of_number target (to_number value)
let wrap value = extended `LuaJITBitTobit [| value |]
let flip_sign value = extended `LuaJITBitBxor [| to_number value; constant 2147483648.0 |]
let add left right = extended `LuaJITAdd [| left; right |]
let subtract left right = extended `LuaJITSubtract [| left; right |]
let multiply left right = extended `LuaJITMultiply [| left; right |]
let divide left right = extended `LuaJITDivide [| left; right |]
let negate value = extended `LuaJITNegate [| value |]
let truncate value = extended `LuaJITMathModf [| value |]
let less_than left right = extended `LuaJITLessThan [| left; right |]
let less_than_equal left right = extended `LuaJITLessThanEqual [| left; right |]
let equal left right = extended `LuaJITEqual [| left; right |]

(* LuaJIT: bit gives a word as its signed number, and the FFI casts numbers to longs. *)
let unsigned value = extended `LuaJITModulo [| to_number value; constant 4294967296.0 |]
let cast target value = extended (`LuaJITCast target) [| value |]
let to_long value = cast Type.i64 (to_number value)
