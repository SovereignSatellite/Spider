open IR
open Definitions

let number link = match link.node.kind with F64Constant value -> Some value | _ -> None

(* LuaJIT: an I64 lowers to an FFI long. *)
let long link = match link.node.kind with I64Constant value -> Some value | _ -> None

let word_type value_type =
  match Type.scalar value_type with Some (`I8 | `I16 | `I32) -> true | Some (`I64 | `F32 | `F64) | None -> false

(* LuaJIT: bit takes integers below 2^51. *)
let exact = 2251799813685248.0
let ones = 0xFFFF_FFFF

let modular value =
  if Float.trunc value = value && Float.abs value < exact then Some (Int64.to_int (Int64.of_float value) land ones)
  else None

let modular_constant link = Option.bind (number link) modular
let signed value = (value lxor 0x8000_0000) - 0x8000_0000
let rotate_left value amount = (value lsl amount) lor (value lsr ((32 - amount) land 31)) land ones

let rec leading_zeros value =
  if value = 0 then 32 else if value land 0x8000_0000 <> 0 then 0 else 1 + leading_zeros ((value lsl 1) land ones)

let bit_binary node left right =
  (* LuaJIT: bit shifts by the amount modulo 32. *)
  let amount = right land 31 in
  match node.kind with
  | Extension `LuaJITBitBand -> Some (left land right)
  | Extension `LuaJITBitBor -> Some (left lor right)
  | Extension `LuaJITBitBxor -> Some (left lxor right)
  (* LuaJIT: bit shifts by the amount modulo 32. *)
  | Extension `LuaJITBitLshift -> Some ((left lsl amount) land ones)
  | Extension `LuaJITBitRshift -> Some (left lsr amount)
  | Extension `LuaJITBitArshift -> Some ((signed left asr amount) land ones)
  | Extension `LuaJITBitRol -> Some (rotate_left left amount)
  | Extension `LuaJITBitRor -> Some (rotate_left left ((32 - amount) land 31))
  | _ -> None

let number_binary node left right =
  match node.kind with
  | Extension `LuaJITModulo -> Some (left -. (right *. Float.floor (left /. right)))
  | Extension `LuaJITMathFmod -> Some (Float.rem left right)
  | _ -> None
