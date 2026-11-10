open IR
open Definitions
open Passes
open Numbers

let widen value = extended `LuauBitsIntoF32 [| value |]
let narrow value = extended `LuauBitsFromF32 [| value |]

let copy_sign types magnitude sign =
  (* Luau: without NaN tagging, any NaN converts as it is. *)
  let high = Link.output (Node.create (Extension `LuauBitsFromI64) [| retype `I64 sign |]) 1 in
  Branch.choose_one types
    (less_than (to_number high) (constant 2147483648.0))
    [| extended `LuauMathAbs [| magnitude |] |]
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
  | Negate -> of_bits (flip_sign value)
  | Absolute -> of_bits (extended `LuauBit32Band [| to_number value; constant 2147483647.0 |])
  (* Luau: an I64 lowers to two I32 words. *)
  | SquareRoot -> narrow (extended `LuauMathSqrt [| widen value |])
  | RoundUp -> narrow (extended `LuauMathCeil [| widen value |])
  | RoundDown -> narrow (extended `LuauMathFloor [| widen value |])
  | Truncate -> narrow (truncate (widen value))
  | Nearest -> narrow (nearest types (widen value))

let unary types width (operator : NumberUnary.t) value =
  match (width, operator) with
  (* Luau: F32 arithmetic runs on F64s, which round it correctly. *)
  | `F32, _ -> Some (unary32 types operator value)
  | `F64, Nearest -> Some (nearest types value)
  | `F64, Negate -> Some (negate value)
  | `F64, Absolute -> Some (extended `LuauMathAbs [| value |])
  | `F64, SquareRoot -> Some (extended `LuauMathSqrt [| value |])
  | `F64, RoundUp -> Some (extended `LuauMathCeil [| value |])
  | `F64, RoundDown -> Some (extended `LuauMathFloor [| value |])
  | `F64, Truncate -> Some (truncate value)

let copy_sign32 left right =
  let magnitude = extended `LuauBit32Band [| to_number left; constant 2147483647.0 |] in
  let sign = extended `LuauBit32Band [| to_number right; constant 2147483648.0 |] in
  of_bits (extended `LuauBit32Bor [| to_number magnitude; to_number sign |])

let binary32 (operator : NumberBinary.t) left right =
  match operator with
  (* Luau: F32 arithmetic runs on F64s, which round it correctly. *)
  | Add -> narrow (add (widen left) (widen right))
  | Subtract -> narrow (subtract (widen left) (widen right))
  | Multiply -> narrow (multiply (widen left) (widen right))
  | Divide -> narrow (divide (widen left) (widen right))
  | (Minimum | Maximum) as operator ->
      narrow (result (Operator (NumberBinary (`F64, operator))) [| widen left; widen right |])
  | CopySign -> copy_sign32 left right

let binary types width (operator : NumberBinary.t) left right =
  match (width, operator) with
  (* Luau: F32 arithmetic runs on F64s, which round it correctly. *)
  | `F32, _ -> Some (binary32 operator left right)
  | `F64, CopySign -> Some (copy_sign types left right)
  | `F64, Add -> Some (add left right)
  | `F64, Subtract -> Some (subtract left right)
  | `F64, Multiply -> Some (multiply left right)
  | `F64, Divide -> Some (divide left right)
  | `F64, (Minimum | Maximum) -> None

let compare width (operator : NumberComparison.t) left right =
  let number value = match width with `F32 -> widen value | `F64 -> value in
  match operator with
  | Equal -> equal (number left) (number right)
  | LessThan -> less_than (number left) (number right)
  | LessThanEqual -> less_than_equal (number left) (number right)

(* Luau: without NaN tagging, any NaN converts as it is. *)
let convert_signed target value = match target with `F64 -> Some (Words.convert signed value) | `F32 -> None
let convert_unsigned target value = match target with `F64 -> Some (Words.convert to_number value) | `F32 -> None
let word_types target = match target with `I32 -> [ Type.i32 ] | `I64 -> Words.word_types

let constant_words target bits =
  let word = Constant.of_bits Type.i32 in
  match target with
  | `I32 -> [| word bits |]
  | `I64 -> [| word (Int64.logand bits 0xFFFF_FFFFL); word (Int64.shift_right_logical bits 32) |]

(** The words join after the Matches, not in each branch, so a reader of one word leaves the other uncomputed. *)
let joined target words = match target with `I32 -> words.(0) | `I64 -> Words.join words.(0) words.(1)

let below_limit types target convert greatest fields =
  Branch.choose types
    (less_than_equal (constant (ConstantFolding.unsigned_to_float greatest +. 1.0)) fields.(0))
    fields (word_types target)
    (fun inner -> convert inner.(0))
    (fun _ -> constant_words target greatest)

let saturate_signed types target value =
  (* Luau: an I64 lowers to two I32 words. *)
  let greatest = Int64.shift_right_logical (Constant.mask (Type.of_scalar target)) 1 in
  let convert = match target with `I32 -> fun number -> [| wrap number |] | `I64 -> Words.truncated_signed in
  let at_or_below_minimum fields =
    Branch.choose types
      (equal fields.(0) fields.(0))
      [||] (word_types target)
      (fun _ -> constant_words target 0L)
      (fun _ -> constant_words target (Int64.succ greatest))
  in
  joined target
    (Branch.choose types
       (less_than (constant (-.(ConstantFolding.unsigned_to_float greatest +. 1.0))) value)
       [| value |] (word_types target) at_or_below_minimum
       (below_limit types target convert greatest))

let saturate_unsigned types target value =
  let convert =
    match target with
    (* Luau: an I64 lowers to two I32 words. *)
    | `I32 -> fun number -> [| to_i32 (extended `LuauMathFloor [| number |]) |]
    | `I64 -> Words.truncated_unsigned
  in
  joined target
    (Branch.choose types
       (less_than (constant 0.0) value)
       [| value |] (word_types target)
       (fun _ -> constant_words target 0L)
       (below_limit types target convert (Constant.mask (Type.of_scalar target))))
