open IR
open Numbers

(* LuaJIT: the FFI multiplies longs. *)
let multiply_i32 left right = extended `LuaJITBitTobit [| extended `LuaJITMultiply [| to_long left; to_long right |] |]

(** A quotient of integers below 2^53 never rounds across an integer, so truncating it is exact. *)
let divide_signed left right = wrap (truncate (divide (to_number left) (to_number right)))

let binary (operator : IntegerBinary.t) left right =
  match operator with
  | Add -> wrap (add (to_number left) (to_number right))
  | Subtract -> wrap (subtract (to_number left) (to_number right))
  | Multiply -> multiply_i32 left right
  | And -> extended `LuaJITBitBand [| to_number left; to_number right |]
  | Or -> extended `LuaJITBitBor [| to_number left; to_number right |]
  | ExclusiveOr -> extended `LuaJITBitBxor [| to_number left; to_number right |]
  (* LuaJIT: bit takes shift amounts modulo 32. *)
  | ShiftLeft -> extended `LuaJITBitLshift [| to_number left; to_number right |]
  | ShiftRightS -> extended `LuaJITBitArshift [| to_number left; to_number right |]
  | ShiftRightU -> extended `LuaJITBitRshift [| to_number left; to_number right |]
  | RotateLeft -> extended `LuaJITBitRol [| to_number left; to_number right |]
  | RotateRight -> extended `LuaJITBitRor [| to_number left; to_number right |]
  | DivideS -> divide_signed left right
  | DivideU -> wrap (extended `LuaJITMathFloor [| divide (unsigned left) (unsigned right) |])
  | RemainderS -> wrap (extended `LuaJITMathFmod [| to_number left; to_number right |])
  | RemainderU -> wrap (extended `LuaJITModulo [| unsigned left; unsigned right |])

let count_ones value =
  (* LuaJIT: undiagnosed; converge or justify. *)
  let pairs =
    subtract (to_number value)
      (to_number
         (extended `LuaJITBitBand
            [| to_number (extended `LuaJITBitRshift [| to_number value; constant 1.0 |]); constant 1431655765.0 |]))
  in
  let nibbles =
    add
      (to_number (extended `LuaJITBitBand [| pairs; constant 858993459.0 |]))
      (to_number
         (extended `LuaJITBitBand
            [| to_number (extended `LuaJITBitRshift [| pairs; constant 2.0 |]); constant 858993459.0 |]))
  in
  let bytes =
    extended `LuaJITBitBand
      [| add nibbles (to_number (extended `LuaJITBitRshift [| nibbles; constant 4.0 |])); constant 252645135.0 |]
  in
  let halves = add (to_number bytes) (to_number (extended `LuaJITBitRshift [| to_number bytes; constant 8.0 |])) in
  extended `LuaJITBitBand
    [| add halves (to_number (extended `LuaJITBitRshift [| halves; constant 16.0 |])); constant 63.0 |]

let count (operator : IntegerCount.t) value =
  match operator with
  (* LuaJIT: bit has no countlz or countrz, so those stay prefab calls. *)
  | LeadingZeros | TrailingZeros -> None
  | CountOnes -> Some (count_ones value)

let compare (operator : IntegerComparison.t) left right =
  match operator with
  | Equal -> equal (to_number left) (to_number right)
  (* LuaJIT: bit gives a word as its signed number. *)
  | LessThanS -> less_than (to_number left) (to_number right)
  | LessThanU -> less_than (to_number (flip_sign left)) (to_number (flip_sign right))

let sign_extend bits value =
  let shift = float_of_int (32 - bits) in
  extended `LuaJITBitArshift
    [| to_number (extended `LuaJITBitLshift [| to_number value; constant shift |]); constant shift |]
