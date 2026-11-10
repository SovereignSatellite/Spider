open IR
open Definitions
open Passes
open LuaJITExtension
open LuaJITValues

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

(** The floored quotient of integers below 2^53 is exact, and LuaJIT computes [a % b] as [a - floor(a / b) * b], exact
    while [|a| + b] is at most 2^53. *)
let divide node left right =
  match node.kind with
  | _ when right.least < 1.0 -> None
  | Extension `LuaJITModulo when Float.max (-.left.least) left.greatest +. right.greatest <= exact ->
      within 0.0 (right.greatest -. 1.0)
  (* LuaJIT: Lua has no floor division, so it floors a quotient. *)
  | Extension `LuaJITDivide when left.least >= 0.0 ->
      within (Float.floor (left.least /. right.greatest)) (Float.floor (left.greatest /. right.least))
  | Extension `LuaJITMathFmod when left.least >= 0.0 -> within 0.0 (Float.min left.greatest (right.greatest -. 1.0))
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
  | Extension `LuaJITToNumber, _, _ when word_type (Link.type_ bounds.types (input ())) -> Some (ones bounds (input ()))
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
  | Some amount, _ -> Option.value (bit_binary node value amount) ~default:any_bits
  | None, Extension `LuaJITBitRshift -> covering value
  | None, Extension `LuaJITBitArshift when value < 0x8000_0000 -> covering value
  | None, _ when value = 0 -> 0
  | None, _ -> any_bits

and find_ones bounds link =
  let operand index = operand_ones bounds link.node.inputs.(index) in
  match link.node.kind with
  | _ when not (word_type (Link.type_ bounds.types link)) -> any_bits
  | I32Constant value -> value
  | Extension `LuaJITBitBand ->
      let left = operand 0 in
      left land operand 1
  | Extension (`LuaJITBitBor | `LuaJITBitBxor) ->
      let left = operand 0 in
      left lor operand 1
  | Extension (`LuaJITBitLshift | `LuaJITBitRshift | `LuaJITBitArshift | `LuaJITBitRol | `LuaJITBitRor) ->
      shifted bounds link.node
  (* LuaJIT: bit has no countlz or countrz, so those stay core operators. *)
  | Operator (IntegerCount (`I32, (LeadingZeros | TrailingZeros))) -> 63
  | Extension (`LuaJITFromNumber _ | `LuaJITBitTobit) -> operand 0
  | I32FromSum when Node.operand_type bounds.types link.node = Type.boolean -> 1
  | RegionMatch _ ->
      Array.fold_left (fun found result -> found lor ones bounds result) 0 (results bounds link.node link.port)
  | RegionArguments _ -> Option.fold ~none:any_bits ~some:(ones bounds) (outside bounds link)
  | _ -> any_bits

and floored bounds value =
  (* LuaJIT: Lua has no floor division, so it floors a quotient. *)
  match value.node.kind with
  | Extension `LuaJITDivide -> both_operands (fun index -> range bounds value.node.inputs.(index)) (divide value.node)
  | _ -> None

and find_range bounds link =
  let range_of index = range bounds link.node.inputs.(index) in
  match link.node.kind with
  | _ when Link.type_ bounds.types link <> Type.f64 -> None
  | F64Constant value when integral value -> within value value
  | Extension `LuaJITToNumber when word_type (Node.operand_type bounds.types link.node) ->
      (* LuaJIT: bit gives a word as its signed number. *)
      let ones = ones bounds link.node.inputs.(0) in
      if ones < 0x8000_0000 then within 0.0 (float_of_int ones) else within (-2147483648.0) 2147483647.0
  | Extension `LuaJITAdd -> both_operands range_of (combine Add)
  | Extension `LuaJITSubtract -> both_operands range_of (combine Subtract)
  | Extension `LuaJITMultiply -> both_operands range_of (combine Multiply)
  | Operator (NumberBinary (`F64, ((Minimum | Maximum) as operator))) -> both_operands range_of (combine operator)
  | Extension `LuaJITReadU8 -> within 0.0 255.0
  | Extension `LuaJITReadU16 -> within 0.0 65535.0
  (* LuaJIT: the FFI reads I32s, and Lua has no floor division. *)
  | Extension `LuaJITReadI32 -> within (-2147483648.0) 2147483647.0
  | Extension (`LuaJITModulo | `LuaJITMathFmod) -> both_operands range_of (divide link.node)
  | Extension `LuaJITMathFloor -> floored bounds link.node.inputs.(0)
  | RegionMatch _ ->
      let ranges = Array.map (range bounds) (results bounds link.node link.port) in
      Array.fold_left union ranges.(0) ranges
  | RegionArguments _ -> Option.bind (outside bounds link) (range bounds)
  | _ -> None
