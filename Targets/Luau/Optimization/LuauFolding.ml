open IR
open Definitions
open Passes
open LuauValues

let both read node =
  match (read node.inputs.(0), read node.inputs.(1)) with Some left, Some right -> Some (left, right) | _ -> None

let held value_type bits =
  match Type.scalar value_type with
  | Some (`I64 | `F64) -> Int64.float_of_bits bits
  (* Luau: bit32 counts zeros, Luau has floor division, and an I64 lowers to two I32 words. *)
  | Some (`I8 | `I16 | `I32 | `F32) | None -> ConstantFolding.unsigned_to_float bits

(** A number the target does not hold leaves [LuauFromNumber] undefined, so any constant of the target stands for it. *)
let holding target value =
  let constant = Constant.of_bits (Type.of_scalar target) in
  match target with
  | `I64 | `F64 -> constant (Int64.bits_of_float value)
  | `I8 | `I16 | `I32 | `F32 -> constant (Int64.of_float value)

let fold_bit32_binary node =
  Option.map Numbers.integer
    (* Luau: bit32 counts zeros, Luau has floor division, and an I64 lowers to two I32 words. *)
    (Option.bind (both modular_constant node) (fun (left, right) -> bit32_binary node left right))

let fold_bit32_unary node =
  Option.map Numbers.integer (Option.bind (modular_constant node.inputs.(0)) (bit32_unary node))

let fold_wrap node = Option.map Numbers.integer (modular_constant node.inputs.(0))

let fold_quotient node =
  Option.map Numbers.constant (Option.bind (both number node) (fun (left, right) -> number_binary node left right))

let fold_arithmetic node =
  let fold operate = Option.map (fun (left, right) -> Numbers.constant (operate left right)) (both number node) in
  let compare test = Option.map (fun (left, right) -> Constant.boolean (test left right)) (both number node) in
  let round operate = Option.map (fun value -> Numbers.constant (operate value)) (number node.inputs.(0)) in
  match node.kind with
  | Extension `LuauAdd -> fold ( +. )
  | Extension `LuauSubtract -> fold ( -. )
  | Extension `LuauMultiply -> fold ( *. )
  | Extension `LuauDivide -> fold ( /. )
  | Extension `LuauNegate -> round Float.neg
  | Extension `LuauMathAbs -> round Float.abs
  | Extension `LuauMathFloor -> round Float.floor
  | Extension `LuauMathCeil -> round Float.ceil
  | Extension `LuauMathSqrt -> round Float.sqrt
  | Extension `LuauMathModf -> round Float.trunc
  | Extension `LuauEqual -> compare ( = )
  | Extension `LuauLessThan -> compare ( < )
  | Extension `LuauLessThanEqual -> compare ( <= )
  | _ -> None

let fold_to_number types node =
  let value = node.inputs.(0) in
  Option.map (fun bits -> Numbers.constant (held (Link.type_ types value) bits)) (Constant.bits value)

let fold_from_number target node = Option.map (holding target) (number node.inputs.(0))

let fold_into_f32 node =
  Option.map (fun bits -> Numbers.constant (Int32.float_of_bits (Int64.to_int32 bits))) (Constant.bits node.inputs.(0))

let fold_from_f32 node =
  Option.map (fun value -> Numbers.result (F32Constant (Int32.bits_of_float value)) [||]) (number node.inputs.(0))

(* Luau: bit32 counts zeros, Luau has floor division, and an I64 lowers to two I32 words. *)
let fold_round_f32 node =
  Option.map (fun value -> Numbers.constant (Int32.float_of_bits (Int32.bits_of_float value))) (number node.inputs.(0))

let fold_into_i64 node =
  Option.map
    (fun (low, high) ->
      Numbers.result (I64Constant (Int64.logor (Int64.of_int low) (Int64.shift_left (Int64.of_int high) 32))) [||])
    (both word node)

let fold_from_i64 node =
  Option.map
    (fun bits ->
      (* Luau: bit32 counts zeros, Luau has floor division, and an I64 lowers to two I32 words. *)
      [|
        Numbers.integer (Int64.to_int bits land ones);
        Numbers.integer (Int64.to_int (Int64.shift_right_logical bits 32));
      |])
    (Constant.bits node.inputs.(0))

(* Luau: bit32 counts zeros, Luau has floor division, and an I64 lowers to two I32 words. *)
let fold_word port node = Option.map (fun words -> words.(port)) (fold_from_i64 node)
let bit_operations = Options.optimization "fold-luau-bit32" Integer Folding 1 "Fold Luau's bit32 operations."

let arithmetic =
  Options.optimization "fold-luau-arithmetic" Number Folding 1
    "Fold Luau's arithmetic, math functions and comparisons of numbers."

(* Luau: bit32 counts zeros, Luau has floor division, and an I64 lowers to two I32 words. *)
let quotients = Options.optimization "fold-luau-quotients" Number Folding 1 "Fold Luau's //, % and math.fmod."

let views =
  Options.optimization "fold-luau-views" Conversion Folding 1 "Fold Luau's views of values as numbers and bits."

let folded options types node =
  let applied = Options.applied options in
  let on operand_type = Node.operand_type types node = operand_type in
  match node.kind with
  | Extension
      ( `LuauBit32Band | `LuauBit32Bor | `LuauBit32Bxor | `LuauBit32Lshift | `LuauBit32Rshift | `LuauBit32Arshift
      | `LuauBit32Lrotate | `LuauBit32Rrotate ) ->
      applied bit_operations (fun () -> fold_bit32_binary node)
  (* Luau: bit32 counts zeros, Luau has floor division, and an I64 lowers to two I32 words. *)
  | Extension (`LuauBit32Countlz | `LuauBit32Countrz) -> applied bit_operations (fun () -> fold_bit32_unary node)
  | Extension `LuauWrap -> applied bit_operations (fun () -> fold_wrap node)
  | Extension
      ( `LuauAdd | `LuauSubtract | `LuauMultiply | `LuauDivide | `LuauNegate | `LuauMathAbs | `LuauMathFloor
      | `LuauMathCeil | `LuauMathSqrt | `LuauMathModf | `LuauEqual | `LuauLessThan | `LuauLessThanEqual )
    when on Type.f64 ->
      applied arithmetic (fun () -> fold_arithmetic node)
  (* Luau: bit32 counts zeros, Luau has floor division, and an I64 lowers to two I32 words. *)
  | Extension (`LuauFloorDivide | `LuauModulo | `LuauMathFmod) -> applied quotients (fun () -> fold_quotient node)
  | Extension `LuauToNumber -> applied views (fun () -> fold_to_number types node)
  | Extension (`LuauFromNumber target) -> applied views (fun () -> fold_from_number target node)
  | Extension `LuauBitsIntoF32 -> applied views (fun () -> fold_into_f32 node)
  | Extension `LuauBitsFromF32 -> applied views (fun () -> fold_from_f32 node)
  (* Luau: bit32 counts zeros, Luau has floor division, and an I64 lowers to two I32 words. *)
  | Extension `LuauBitsRoundF32 -> applied views (fun () -> fold_round_f32 node)
  | Extension `LuauBitsIntoI64 -> applied views (fun () -> fold_into_i64 node)
  | Extension `LuauBitsLowFromI64 -> applied views (fun () -> fold_word 0 node)
  | Extension `LuauBitsHighFromI64 -> applied views (fun () -> fold_word 1 node)
  | _ -> None

let fold options types node =
  (* Luau: bit32 counts zeros, Luau has floor division, and an I64 lowers to two I32 words. *)
  match node.kind with
  | Extension `LuauBitsFromI64 -> Options.applied options views (fun () -> fold_from_i64 node)
  | _ -> Option.map (fun folded -> [| folded |]) (folded options types node)
