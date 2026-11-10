open IR
open Numbers

(* Luau: bit32 takes shift amounts from 0 to 31, and products are exact only below 2^53. *)

(** [bit32] takes a shift or rotation amount from 0 to 31, and an I32's amount is taken modulo 32. *)
let amount value = to_number (extended `LuauBit32Band [| to_number value; constant 31.0 |])

let low value = extended `LuauBit32Band [| to_number value; constant 65535.0 |]

let multiply_i32 left right =
  (* Luau: bit32 gives a word as its unsigned number and wraps any number below 2^63. *)
  let right_lower = to_number (low right) in
  wrap
    (add
       (multiply (to_number left) right_lower)
       (multiply (to_number (low left)) (subtract (to_number right) right_lower)))

(** A quotient of integers below 2^53 never rounds across an integer, so the wrap's truncation is exact. *)
let divide_signed left right = wrap (divide (signed left) (signed right))

let binary (operator : IntegerBinary.t) left right =
  match operator with
  | Add -> wrap (add (to_number left) (to_number right))
  | Subtract -> wrap (subtract (to_number left) (to_number right))
  | Multiply -> multiply_i32 left right
  | And -> extended `LuauBit32Band [| to_number left; to_number right |]
  | Or -> extended `LuauBit32Bor [| to_number left; to_number right |]
  | ExclusiveOr -> extended `LuauBit32Bxor [| to_number left; to_number right |]
  (* Luau: bit32 takes shift amounts from 0 to 31. *)
  | ShiftLeft -> extended `LuauBit32Lshift [| to_number left; amount right |]
  | ShiftRightS -> extended `LuauBit32Arshift [| to_number left; amount right |]
  | ShiftRightU -> extended `LuauBit32Rshift [| to_number left; amount right |]
  | RotateLeft -> extended `LuauBit32Lrotate [| to_number left; amount right |]
  | RotateRight -> extended `LuauBit32Rrotate [| to_number left; amount right |]
  | DivideS -> divide_signed left right
  | DivideU -> to_i32 (extended `LuauFloorDivide [| to_number left; to_number right |])
  | RemainderS -> wrap (extended `LuauMathFmod [| signed left; signed right |])
  | RemainderU -> to_i32 (extended `LuauModulo [| to_number left; to_number right |])

let count_ones value =
  (* Luau: bit32 gives a word as its unsigned number, and Luau has floor division. *)
  let field amount mask =
    to_number
      (extended `LuauBit32Band
         (* Luau: undiagnosed; converge or justify. *)
         [| to_number (extended `LuauBit32Rshift [| to_number value; constant amount |]); constant mask |])
  in
  let threes = subtract (subtract (to_number value) (field 1.0 3681400539.0)) (field 2.0 1227133513.0) in
  let sixes =
    extended `LuauBit32Band
      [| add threes (to_number (extended `LuauBit32Rshift [| threes; constant 3.0 |])); constant 3340530119.0 |]
  in
  to_i32 (extended `LuauModulo [| to_number sixes; constant 63.0 |])

let count (operator : IntegerCount.t) value =
  match operator with
  (* Luau: bit32 counts leading and trailing zeros. *)
  | LeadingZeros -> extended `LuauBit32Countlz [| to_number value |]
  | TrailingZeros -> extended `LuauBit32Countrz [| to_number value |]
  | CountOnes -> count_ones value

let compare (operator : IntegerComparison.t) left right =
  match operator with
  | Equal -> equal (to_number left) (to_number right)
  (* Luau: bit32 gives a word as its unsigned number. *)
  | LessThanS -> less_than (to_number (flip_sign left)) (to_number (flip_sign right))
  | LessThanU -> less_than (to_number left) (to_number right)

let sign_extend bits value =
  let shift = float_of_int (32 - bits) in
  extended `LuauBit32Arshift
    [| to_number (extended `LuauBit32Lshift [| to_number value; constant shift |]); constant shift |]
