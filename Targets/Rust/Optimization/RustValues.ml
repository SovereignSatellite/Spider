open IR
open Definitions

let integer_type types value_type =
  match TypeArena.definition types value_type with
  | Structural (Integer bits) -> Some (bits, false)
  | Extension (IR_Rust.Signed bits) -> Some (bits, true)
  | _ -> None

let held (bits, signed) value =
  if bits = 64 then value
  else
    let modulus = Int64.shift_left 1L bits in
    let low = Int64.logand value (Int64.pred modulus) in
    if signed && Int64.logand low (Int64.shift_right_logical modulus 1) <> 0L then Int64.sub low modulus else low

let keeps types source target =
  match (integer_type types source, integer_type types target) with
  | Some (source_bits, source_signed), Some (target_bits, target_signed) ->
      (target_signed || not source_signed)
      && (source_bits < target_bits || (source_bits = target_bits && source_signed = target_signed))
  | _ -> false

let rec integer types link =
  match (link.node.kind, Node.bits link.node.kind, integer_type types (Link.type_ types link)) with
  | _, Some bits, Some _ -> Some bits
  | Extension (`RustCast _), _, Some target -> Option.map (held target) (integer types link.node.inputs.(0))
  | _ -> None
