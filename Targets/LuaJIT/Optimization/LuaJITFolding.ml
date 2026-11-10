open IR
open Definitions
open Passes
open LuaJITValues

let both read node =
  match (read node.inputs.(0), read node.inputs.(1)) with Some left, Some right -> Some (left, right) | _ -> None

(* LuaJIT: words are signed, an I64 lowers to an FFI long, and operators take longs too. *)

(** An I32 or F32 is the signed value of its bits. *)
let held value_type bits =
  match Type.scalar value_type with
  | Some (`I64 | `F64) -> Int64.float_of_bits bits
  (* LuaJIT: words are signed, an I64 lowers to an FFI long, and operators take longs too. *)
  | Some (`I32 | `F32) -> float_of_int (signed (Int64.to_int bits))
  | Some (`I8 | `I16) | None -> ConstantFolding.unsigned_to_float bits

(** A number the target does not hold leaves [LuaJITFromNumber] undefined, so any constant of the target stands for it.
*)
let holding target value =
  let constant = Constant.of_bits (Type.of_scalar target) in
  match target with
  | `I64 | `F64 -> constant (Int64.bits_of_float value)
  | `I8 | `I16 | `I32 | `F32 -> constant (Int64.of_float value)

let fold_bit_binary node =
  Option.map Numbers.integer
    (Option.bind (both modular_constant node) (fun (left, right) -> bit_binary node left right))

let fold_tobit node = Option.map Numbers.integer (modular_constant node.inputs.(0))

let fold_quotient node =
  Option.map Numbers.constant (Option.bind (both number node) (fun (left, right) -> number_binary node left right))

let fold_arithmetic node =
  let fold operate = Option.map (fun (left, right) -> Numbers.constant (operate left right)) (both number node) in
  let compare test = Option.map (fun (left, right) -> Constant.boolean (test left right)) (both number node) in
  let round operate = Option.map (fun value -> Numbers.constant (operate value)) (number node.inputs.(0)) in
  match node.kind with
  | Extension `LuaJITAdd -> fold ( +. )
  | Extension `LuaJITSubtract -> fold ( -. )
  | Extension `LuaJITMultiply -> fold ( *. )
  | Extension `LuaJITDivide -> fold ( /. )
  | Extension `LuaJITNegate -> round Float.neg
  | Extension `LuaJITMathAbs -> round Float.abs
  | Extension `LuaJITMathFloor -> round Float.floor
  | Extension `LuaJITMathCeil -> round Float.ceil
  | Extension `LuaJITMathSqrt -> round Float.sqrt
  | Extension `LuaJITMathModf -> round Float.trunc
  | Extension `LuaJITEqual -> compare ( = )
  | Extension `LuaJITLessThan -> compare ( < )
  | Extension `LuaJITLessThanEqual -> compare ( <= )
  | _ -> None

(* LuaJIT: words are signed, an I64 lowers to an FFI long, and operators take longs too. *)

(** An I64's arithmetic is [int64_t]'s, modulo 2^64 as OCaml's [Int64] is; a quotient or remainder by zero, or
    overflowing, is left. *)
let fold_long node =
  let fold operate =
    Option.map
      (fun value -> Numbers.result (I64Constant value) [||])
      (Option.bind (both long node) (fun (left, right) -> operate left right))
  in
  let defined left right = right <> 0L && not (left = Int64.min_int && right = -1L) in
  match node.kind with
  | Extension `LuaJITAdd -> fold (fun left right -> Some (Int64.add left right))
  | Extension `LuaJITSubtract -> fold (fun left right -> Some (Int64.sub left right))
  | Extension `LuaJITMultiply -> fold (fun left right -> Some (Int64.mul left right))
  | Extension `LuaJITDivide -> fold (fun left right -> if defined left right then Some (Int64.div left right) else None)
  | Extension `LuaJITModulo -> fold (fun left right -> if defined left right then Some (Int64.rem left right) else None)
  | Extension `LuaJITLessThan -> Option.map (fun (left, right) -> Constant.boolean (left < right)) (both long node)
  | Extension `LuaJITEqual -> Option.map (fun (left, right) -> Constant.boolean (left = right)) (both long node)
  | _ -> None

let fold_to_number types node =
  let value = node.inputs.(0) in
  Option.map (fun bits -> Numbers.constant (held (Link.type_ types value) bits)) (Constant.bits value)

let fold_from_number target node = Option.map (holding target) (number node.inputs.(0))

let fold_into_f32 node =
  Option.map (fun bits -> Numbers.constant (Int32.float_of_bits (Int64.to_int32 bits))) (Constant.bits node.inputs.(0))

let fold_from_f32 node =
  Option.map (fun value -> Numbers.result (F32Constant (Int32.bits_of_float value)) [||]) (number node.inputs.(0))

(* LuaJIT: words are signed, an I64 lowers to an FFI long, and operators take longs too. *)
let fold_into_f64 node =
  let canonical bits =
    if Float.is_nan (Int64.float_of_bits bits) then if bits < 0L then 0xFFF8_0000_0000_0000L else 0x7FF8_0000_0000_0000L
    else bits
  in
  Option.map (fun bits -> Numbers.constant (Int64.float_of_bits (canonical bits))) (Constant.bits node.inputs.(0))

let fold_from_f64 node =
  Option.map (fun value -> Numbers.result (I64Constant (Int64.bits_of_float value)) [||]) (number node.inputs.(0))

let bit_operations = Options.optimization "fold-luajit-bit" Integer Folding 1 "Fold LuaJIT's bit operations."

let arithmetic =
  Options.optimization "fold-luajit-arithmetic" Number Folding 1
    "Fold LuaJIT's arithmetic, math functions and comparisons of numbers."

(* LuaJIT: words are signed, an I64 lowers to an FFI long, and operators take longs too. *)
let longs =
  Options.optimization "fold-luajit-longs" Integer Folding 1 "Fold LuaJIT's arithmetic and comparisons of int64_t."

let quotients = Options.optimization "fold-luajit-quotients" Number Folding 1 "Fold LuaJIT's % and math.fmod."

let views =
  Options.optimization "fold-luajit-views" Conversion Folding 1 "Fold LuaJIT's views of values as numbers and bits."

let folded options types node =
  let applied = Options.applied options in
  let on operand_type = Node.operand_type types node = operand_type in
  match node.kind with
  | Extension
      ( `LuaJITBitBand | `LuaJITBitBor | `LuaJITBitBxor | `LuaJITBitLshift | `LuaJITBitRshift | `LuaJITBitArshift
      | `LuaJITBitRol | `LuaJITBitRor )
  (* LuaJIT: words are signed, an I64 lowers to an FFI long, and operators take longs too. *)
    when on Type.f64 ->
      applied bit_operations (fun () -> fold_bit_binary node)
  | Extension `LuaJITBitTobit when on Type.f64 -> applied bit_operations (fun () -> fold_tobit node)
  | Extension
      (`LuaJITAdd | `LuaJITSubtract | `LuaJITMultiply | `LuaJITDivide | `LuaJITModulo | `LuaJITEqual | `LuaJITLessThan)
    when on Type.i64 ->
      applied longs (fun () -> fold_long node)
  | Extension
      ( `LuaJITAdd | `LuaJITSubtract | `LuaJITMultiply | `LuaJITDivide | `LuaJITNegate | `LuaJITMathAbs
      | `LuaJITMathFloor | `LuaJITMathCeil | `LuaJITMathSqrt | `LuaJITMathModf | `LuaJITEqual | `LuaJITLessThan
      | `LuaJITLessThanEqual )
    when on Type.f64 ->
      applied arithmetic (fun () -> fold_arithmetic node)
  (* LuaJIT: words are signed, an I64 lowers to an FFI long, and operators take longs too. *)
  | Extension (`LuaJITModulo | `LuaJITMathFmod) when on Type.f64 -> applied quotients (fun () -> fold_quotient node)
  | Extension `LuaJITToNumber -> applied views (fun () -> fold_to_number types node)
  | Extension (`LuaJITFromNumber target) -> applied views (fun () -> fold_from_number target node)
  | Extension `LuaJITBitsIntoF32 -> applied views (fun () -> fold_into_f32 node)
  | Extension `LuaJITBitsFromF32 -> applied views (fun () -> fold_from_f32 node)
  (* LuaJIT: words are signed, an I64 lowers to an FFI long, and operators take longs too. *)
  | Extension `LuaJITBitsIntoF64 -> applied views (fun () -> fold_into_f64 node)
  | Extension `LuaJITBitsFromF64 -> applied views (fun () -> fold_from_f64 node)
  | _ -> None

let fold options types node = Option.map (fun folded -> [| folded |]) (folded options types node)
