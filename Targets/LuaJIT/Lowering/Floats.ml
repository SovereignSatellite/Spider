open IR
open Definitions
open Passes
open Numbers

(* LuaJIT: NaN tagging reads other NaNs as other values. *)
let widen types value =
  let nan =
    less_than (constant 2139095040.0) (to_number (extended `LuaJITBitBand [| to_number value; constant 2147483647.0 |]))
  in
  let canonical fields =
    let sign = extended `LuaJITBitBand [| to_number fields.(0); constant (-2147483648.0) |] in
    of_number `F32 (to_number (extended `LuaJITBitBor [| to_number sign; constant 2143289344.0 |]))
  in
  extended `LuaJITBitsIntoF32
    [| Branch.choose_one types nan [| value |] Type.f32 (fun fields -> fields.(0)) canonical |]

let narrow value = extended `LuaJITBitsFromF32 [| value |]

let copy_sign types magnitude sign =
  (* LuaJIT: an I64 lowers to an FFI long. *)
  let bits = extended `LuaJITBitsFromF64 [| sign |] in
  Branch.choose_one types
    (extended `LuaJITLessThan [| result (I64Constant (-1L)) [||]; bits |])
    [| extended `LuaJITMathAbs [| magnitude |] |]
    Type.f64
    (fun fields -> negate fields.(0))
    (fun fields -> fields.(0))

(** Below 2^52, adding and subtracting 2^52 rounds a positive number to the nearest integer, halves to even; from 2^52
    every number is an integer. *)
let nearest_positive types value =
  Branch.choose_one types
    (less_than value (constant 4503599627370496.0))
    [| value |] Type.f64
    (fun fields -> fields.(0))
    (fun fields -> subtract (add fields.(0) (constant 4503599627370496.0)) (constant 4503599627370496.0))

(** Doubling a zero keeps its sign and doubling a NaN quiets it. *)
let nearest types value =
  Branch.choose_one types
    (less_than (constant 0.0) value)
    [| value |] Type.f64
    (fun fields ->
      Branch.choose_one types
        (less_than fields.(0) (constant 0.0))
        fields Type.f64
        (fun inner -> add inner.(0) inner.(0))
        (fun inner -> negate (nearest_positive types (negate inner.(0)))))
    (fun fields -> nearest_positive types fields.(0))

let of_bits value = retype `F32 value

let unary32 types (operator : NumberUnary.t) value =
  match operator with
  (* LuaJIT: the runtime native code computes basic F32 arithmetic. *)
  | Negate -> Some (of_bits (flip_sign value))
  | Absolute -> Some (of_bits (extended `LuaJITBitBand [| to_number value; constant 2147483647.0 |]))
  | SquareRoot -> None
  | RoundUp -> Some (narrow (extended `LuaJITMathCeil [| widen types value |]))
  | RoundDown -> Some (narrow (extended `LuaJITMathFloor [| widen types value |]))
  | Truncate -> Some (narrow (truncate (widen types value)))
  | Nearest -> Some (narrow (nearest types (widen types value)))

let unary types width (operator : NumberUnary.t) value =
  match (width, operator) with
  | `F32, _ -> unary32 types operator value
  | `F64, Nearest -> Some (nearest types value)
  | `F64, Negate -> Some (negate value)
  | `F64, Absolute -> Some (extended `LuaJITMathAbs [| value |])
  | `F64, SquareRoot -> Some (extended `LuaJITMathSqrt [| value |])
  | `F64, RoundUp -> Some (extended `LuaJITMathCeil [| value |])
  | `F64, RoundDown -> Some (extended `LuaJITMathFloor [| value |])
  | `F64, Truncate -> Some (truncate value)

let copy_sign32 left right =
  let magnitude = extended `LuaJITBitBand [| to_number left; constant 2147483647.0 |] in
  let sign = extended `LuaJITBitBand [| to_number right; constant 2147483648.0 |] in
  of_bits (extended `LuaJITBitBor [| to_number magnitude; to_number sign |])

(* LuaJIT: the runtime native code computes basic F32 arithmetic. *)
let binary32 types (operator : NumberBinary.t) left right =
  match operator with
  | Add | Subtract | Multiply | Divide -> None
  | (Minimum | Maximum) as operator ->
      Some (narrow (result (Operator (NumberBinary (`F64, operator))) [| widen types left; widen types right |]))
  | CopySign -> Some (copy_sign32 left right)

let binary types width (operator : NumberBinary.t) left right =
  match (width, operator) with
  (* LuaJIT: the runtime native code computes basic F32 arithmetic. *)
  | `F32, _ -> binary32 types operator left right
  | `F64, CopySign -> Some (copy_sign types left right)
  | `F64, Add -> Some (add left right)
  | `F64, Subtract -> Some (subtract left right)
  | `F64, Multiply -> Some (multiply left right)
  | `F64, Divide -> Some (divide left right)
  | `F64, (Minimum | Maximum) -> None

(* LuaJIT: NaN tagging reads other NaNs as other values. *)
let compare types width (operator : NumberComparison.t) left right =
  let number value = match width with `F32 -> widen types value | `F64 -> value in
  match operator with
  | Equal -> equal (number left) (number right)
  | LessThan -> less_than (number left) (number right)
  | LessThanEqual -> less_than_equal (number left) (number right)

(* LuaJIT: an I64 lowers to an FFI long. *)
let convert_signed target value = match target with `F64 -> Some (Longs.convert_signed value) | `F32 -> None

let convert_unsigned types target value =
  match target with `F64 -> Some (Longs.convert_unsigned types value) | `F32 -> None

let below_limit types result_type convert greatest fields =
  Branch.choose_one types
    (less_than_equal (constant (ConstantFolding.unsigned_to_float greatest +. 1.0)) fields.(0))
    fields result_type
    (fun inner -> convert inner.(0))
    (fun _ -> Constant.of_bits result_type greatest)

let saturate_signed types target value =
  (* LuaJIT: an I64 lowers to an FFI long. *)
  let result_type = Type.of_scalar target in
  let greatest = Int64.shift_right_logical (Constant.mask result_type) 1 in
  let convert = match target with `I32 -> fun number -> wrap (truncate number) | `I64 -> Longs.truncated types in
  let at_or_below_minimum fields =
    Branch.choose_one types
      (equal fields.(0) fields.(0))
      [||] result_type
      (fun _ -> Constant.of_bits result_type 0L)
      (fun _ -> Constant.of_bits result_type (Int64.succ greatest))
  in
  Branch.choose_one types
    (less_than (constant (-.(ConstantFolding.unsigned_to_float greatest +. 1.0))) value)
    [| value |] result_type at_or_below_minimum
    (below_limit types result_type convert greatest)

let saturate_unsigned types target value =
  (* LuaJIT: an I64 lowers to an FFI long. *)
  let result_type = Type.of_scalar target in
  let convert =
    match target with
    (* LuaJIT: an I64 lowers to an FFI long. *)
    | `I32 -> fun number -> wrap (extended `LuaJITMathFloor [| number |])
    | `I64 -> Longs.truncated types
  in
  Branch.choose_one types
    (less_than (constant 0.0) value)
    [| value |] result_type
    (fun _ -> Constant.of_bits result_type 0L)
    (below_limit types result_type convert (Constant.mask result_type))
