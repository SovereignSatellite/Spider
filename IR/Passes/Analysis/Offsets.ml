open IR
open Definitions

let operation_offset node width (operator : IntegerBinary.t) =
  match (operator, Constant.bits node.inputs.(1)) with
  | Add, Some value -> Some value
  | Subtract, Some value -> Some (Int64.logand (Int64.neg value) (Constant.mask (Type.of_scalar width)))
  | _ -> None

let offset_of node =
  match node.kind with
  | Operator (IntegerBinary (width, operator)) ->
      Option.map (fun offset -> (width, offset)) (operation_offset node width operator)
  | _ -> None

let split link =
  match (Constant.bits link, offset_of link.node) with
  | Some value, _ -> (None, value)
  | None, Some (_, value) -> (Some link.node.inputs.(0), value)
  | None, None -> (Some link, 0L)

let offset width origin value =
  let origin_type = Type.of_scalar width in
  let full_mask = Constant.mask origin_type in
  let built operator amount =
    Link.output
      (Node.create (Operator (IntegerBinary (width, operator))) [| origin; Constant.of_bits origin_type amount |])
      0
  in
  if value = 0L then origin
  else if Int64.unsigned_compare value (Int64.shift_right_logical full_mask 1) <= 0 then built Add value
  else built Subtract (Int64.logand (Int64.neg value) full_mask)
