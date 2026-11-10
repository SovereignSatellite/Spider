open IR
open Definitions

let number link = match link.node.kind with F64Constant value -> Some value | _ -> None

(* Luau: an I64 lowers to two I32 words. *)
let word link = match link.node.kind with I32Constant value -> Some value | _ -> None

let word_type value_type =
  match Type.scalar value_type with Some (`I8 | `I16 | `I32) -> true | Some (`I64 | `F32 | `F64) | None -> false

(* Luau: bit32 takes integers below 2^53. *)
let exact = 9007199254740992.0
let ones = 0xFFFF_FFFF

let modular value =
  if Float.trunc value = value && Float.abs value < exact then Some (Int64.to_int (Int64.of_float value) land ones)
  else None

let modular_constant link = Option.bind (number link) modular
let signed value = (value lxor 0x8000_0000) - 0x8000_0000
let rotate_left value amount = (value lsl amount) lor (value lsr ((32 - amount) land 31)) land ones

let rec leading_zeros value =
  if value = 0 then 32 else if value land 0x8000_0000 <> 0 then 0 else 1 + leading_zeros ((value lsl 1) land ones)

(* Luau: bit32 counts leading and trailing zeros. *)
let rec trailing_zeros value =
  if value = 0 then 32 else if value land 1 <> 0 then 0 else 1 + trailing_zeros (value lsr 1)

let bit32_binary node left right =
  match node.kind with
  | Extension `LuauBit32Band -> Some (left land right)
  | Extension `LuauBit32Bor -> Some (left lor right)
  | Extension `LuauBit32Bxor -> Some (left lxor right)
  (* Luau: bit32 shifts by the whole amount. *)
  | Extension `LuauBit32Lshift -> Some ((left lsl right) land ones)
  | Extension `LuauBit32Rshift -> Some (left lsr right)
  | Extension `LuauBit32Arshift -> Some ((signed left asr min right 31) land ones)
  | Extension `LuauBit32Lrotate -> Some (rotate_left left right)
  | Extension `LuauBit32Rrotate -> Some (rotate_left left ((32 - right) land 31))
  | _ -> None

let bit32_unary node value =
  match node.kind with
  | Extension `LuauBit32Countlz -> Some (leading_zeros value)
  | Extension `LuauBit32Countrz -> Some (trailing_zeros value)
  | _ -> None

let number_binary node left right =
  match node.kind with
  (* Luau: Luau has floor division. *)
  | Extension `LuauFloorDivide -> Some (Float.floor (left /. right))
  | Extension `LuauModulo -> Some (left -. (right *. Float.floor (left /. right)))
  | Extension `LuauMathFmod -> Some (Float.rem left right)
  | _ -> None
