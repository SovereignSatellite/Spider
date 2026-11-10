open IR
open Definitions
open Passes
open LuauExtension
open LuauValues

type range = { least : float; greatest : float }

let within least greatest = if -.exact < least && greatest < exact then Some { least; greatest } else None
let covering value = ones lsr leading_zeros value
let any_bits = ones
let integral value = Option.is_some (modular value) && Int64.bits_of_float value <> Int64.min_int

let multiply left right =
  let corners =
    [
      left.least *. right.least;
      left.least *. right.greatest;
      left.greatest *. right.least;
      left.greatest *. right.greatest;
    ]
  in
  within (List.fold_left Float.min Float.infinity corners) (List.fold_left Float.max Float.neg_infinity corners)

let combine (operator : NumberBinary.t) left right =
  match operator with
  | Add -> within (left.least +. right.least) (left.greatest +. right.greatest)
  | Subtract -> within (left.least -. right.greatest) (left.greatest -. right.least)
  | Multiply
    when (left.least <= 0.0 && 0.0 <= left.greatest && right.least < 0.0)
         || (right.least <= 0.0 && 0.0 <= right.greatest && left.least < 0.0) ->
      None
  | Multiply -> multiply left right
  | Minimum -> within (Float.min left.least right.least) (Float.min left.greatest right.greatest)
  | Maximum -> within (Float.max left.least right.least) (Float.max left.greatest right.greatest)
  | Divide | CopySign -> None

(** The floored quotient of integers below 2^53 is exact, and Luau computes [a % b] as [a - floor(a / b) * b], exact
    while [|a| + b] is at most 2^53. *)
let divide node left right =
  match node.kind with
  | _ when right.least < 1.0 -> None
  | Extension `LuauModulo when Float.max (-.left.least) left.greatest +. right.greatest <= exact ->
      within 0.0 (right.greatest -. 1.0)
  (* Luau: Luau has floor division. *)
  | Extension `LuauFloorDivide when left.least >= 0.0 ->
      within (Float.floor (left.least /. right.greatest)) (Float.floor (left.greatest /. right.least))
  | Extension `LuauMathFmod when left.least >= 0.0 -> within 0.0 (Float.min left.greatest (right.greatest -. 1.0))
  | _ -> None

let both_operands range apply = match (range 0, range 1) with Some left, Some right -> apply left right | _ -> None

let union left right =
  match (left, right) with
  | Some left, Some right -> within (Float.min left.least right.least) (Float.max left.greatest right.greatest)
  | _ -> None

type t = {
  types : x type_arena;
  known_ones : (x, int) LinkTable.t;
  ranges : (x, range option) LinkTable.t;
  branches : (x, x node * int) NodeTable.t;
}

let create types =
  { types; known_ones = LinkTable.create 64; ranges = LinkTable.create 64; branches = NodeTable.create 16 }

let types bounds = bounds.types

let results bounds matcher port =
  let branches = RegionMatch.branches matcher in
  Array.iteri (fun variant branch -> NodeTable.replace bounds.branches branch.arguments (matcher, variant)) branches;
  Array.map (fun branch -> branch.results.inputs.(port)) branches

let outside bounds link =
  match NodeTable.find_opt bounds.branches link.node with
  | Some (matcher, variant) -> Branch.outside matcher variant link
  | None -> None

let rec ones bounds link =
  match LinkTable.find_opt bounds.known_ones link with
  | Some known -> known
  | None ->
      let found = find_ones bounds link in
      LinkTable.replace bounds.known_ones link found;
      found

and word_ones bounds link =
  let input () = link.node.inputs.(0) in
  match (link.node.kind, modular_constant link, range bounds link) with
  | _, Some value, _ when number link = Some (float_of_int value) -> Some value
  | Extension `LuauToNumber, _, _ when word_type (Link.type_ bounds.types (input ())) -> Some (ones bounds (input ()))
  | _, _, Some range when range.least >= 0.0 && range.greatest < 4294967296.0 ->
      Some (covering (int_of_float range.greatest))
  | _ -> None

and operand_ones bounds link =
  match modular_constant link with
  | Some value -> value
  | None -> Option.value (word_ones bounds link) ~default:any_bits

and range bounds link =
  match LinkTable.find_opt bounds.ranges link with
  | Some known -> known
  | None ->
      let found = find_range bounds link in
      LinkTable.replace bounds.ranges link found;
      found

and shifted bounds node =
  let value = operand_ones bounds node.inputs.(0) in
  match (modular_constant node.inputs.(1), node.kind) with
  | Some amount, _ -> Option.value (bit32_binary node value amount) ~default:any_bits
  | None, Extension `LuauBit32Rshift -> covering value
  | None, Extension `LuauBit32Arshift when value < 0x8000_0000 -> covering value
  | None, _ when value = 0 -> 0
  | None, _ -> any_bits

and find_ones bounds link =
  let operand index = operand_ones bounds link.node.inputs.(index) in
  match link.node.kind with
  | _ when not (word_type (Link.type_ bounds.types link)) -> any_bits
  | I32Constant value -> value
  | Extension `LuauBit32Band ->
      let left = operand 0 in
      left land operand 1
  | Extension (`LuauBit32Bor | `LuauBit32Bxor) ->
      let left = operand 0 in
      left lor operand 1
  | Extension (`LuauBit32Lshift | `LuauBit32Rshift | `LuauBit32Arshift | `LuauBit32Lrotate | `LuauBit32Rrotate) ->
      shifted bounds link.node
  (* Luau: bit32 counts leading and trailing zeros. *)
  | Extension (`LuauBit32Countlz | `LuauBit32Countrz) -> 63
  | Extension (`LuauFromNumber _ | `LuauWrap) -> operand 0
  | I32FromSum when Node.operand_type bounds.types link.node = Type.boolean -> 1
  | RegionMatch _ ->
      Array.fold_left (fun found result -> found lor ones bounds result) 0 (results bounds link.node link.port)
  | RegionArguments _ -> Option.fold ~none:any_bits ~some:(ones bounds) (outside bounds link)
  | _ -> any_bits

(* Luau: words shift through multiplication by powers of two. *)

(** A non-negative multiple stays non-negative, and rounding keeps a product's order. *)
and floored bounds value =
  match value.node with
  (* Luau: words shift through multiplication by powers of two. *)
  | { kind = Extension `LuauMultiply; inputs = [| multiplied; { node = { kind = F64Constant scale; _ }; _ } |]; _ }
    when scale > 0.0 -> (
      match range bounds multiplied with
      | Some range when range.least >= 0.0 ->
          within (Float.floor (range.least *. scale)) (Float.floor (range.greatest *. scale))
      | _ -> None)
  | _ -> None

and find_range bounds link =
  let range_of index = range bounds link.node.inputs.(index) in
  match link.node.kind with
  | _ when Link.type_ bounds.types link <> Type.f64 -> None
  | F64Constant value when integral value -> within value value
  | Extension `LuauToNumber when word_type (Node.operand_type bounds.types link.node) ->
      (* Luau: bit32 gives a word as its unsigned number. *)
      within 0.0 (float_of_int (ones bounds link.node.inputs.(0)))
  | Extension `LuauAdd -> both_operands range_of (combine Add)
  | Extension `LuauSubtract -> both_operands range_of (combine Subtract)
  | Extension `LuauMultiply -> both_operands range_of (combine Multiply)
  | Operator (NumberBinary (`F64, ((Minimum | Maximum) as operator))) -> both_operands range_of (combine operator)
  | Extension `LuauBufferReadU8 -> within 0.0 255.0
  (* Luau: buffer reads I8, I16 and U32, and Luau has floor division. *)
  | Extension `LuauBufferReadI8 -> within (-128.0) 127.0
  | Extension `LuauBufferReadU16 -> within 0.0 65535.0
  | Extension `LuauBufferReadI16 -> within (-32768.0) 32767.0
  | Extension `LuauBufferReadU32 -> within 0.0 4294967295.0
  | Extension (`LuauFloorDivide | `LuauModulo | `LuauMathFmod) -> both_operands range_of (divide link.node)
  | Extension `LuauMathFloor -> floored bounds link.node.inputs.(0)
  | RegionMatch _ ->
      let ranges = Array.map (range bounds) (results bounds link.node link.port) in
      Array.fold_left union ranges.(0) ranges
  | RegionArguments _ -> Option.bind (outside bounds link) (range bounds)
  | _ -> None
