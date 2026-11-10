open IR
open Definitions

let bits link = Node.bits link.node.kind

let of_bits value_type bits =
  let kind =
    match Type.scalar value_type with
    | Some `I8 -> I8Constant (Int64.to_int bits land 0xFF)
    | Some `I16 -> I16Constant (Int64.to_int bits land 0xFFFF)
    | Some `I32 -> I32Constant (Int64.to_int bits land 0xFFFF_FFFF)
    | Some `I64 -> I64Constant bits
    | Some `F32 -> F32Constant (Int64.to_int32 bits)
    | Some `F64 -> F64Constant (Int64.float_of_bits bits)
    | None -> invalid_arg "Only integers and numbers have constants."
  in
  Link.output (Node.create kind [||]) 0

let mask value_type = Int64.shift_right_logical (-1L) (64 - Type.bits value_type)

let is node =
  match node.kind with
  | SumNew _ -> Array.length node.inputs = 0
  | RegionReference _ -> true
  | kind -> Option.is_some (Node.bits kind)

let copy constant = { constant with node = Copying.node constant.node [||] }

let boolean truth =
  Link.output (Node.create (SumNew { sum_type = Type.boolean; variant = (if truth then 1 else 0) }) [||]) 0
