open IR
open Definitions
open Allocator
open Emission
open Tree
open LuauExtension

type reader = (x, Expression.t) Computation.reader
type computation = (Expression.t, Statement.t) Computation.t

let call path arguments : Expression.t = Call (Name path, arguments)

let helper (context : Context.t) name : Expression.t =
  RuntimeLibrary.NeededHelpers.need context.helpers name;
  Name (Helper name)

let call_helper context name arguments : Expression.t = Call (helper context name, arguments)

(* Luau: Luau has types, if-expressions, floor division and buffers, and keeps NaN and subnormal bits through words. *)
let each_leaf (value : int -> Expression.t) = Computation.each_leaf value
let single (value : Expression.t) = Computation.single value
let computed_where_read _ = true
let leaf_values (context : Context.t) reader link = Computation.leaf_values context.layouts reader link
let value (reader : reader) link = reader.value link 0
let types (context : Context.t) = Layouts.types context.layouts

module Spelling = struct
  type expression = Expression.t

  let truth truth : expression = Name (Text (if truth then "true" else "false"))
  let equal value variant : expression = Binary (Equal, value, Integer (Int64.of_int variant))
  let both first second : expression = Binary (And, first, second)
  let either first second : expression = Binary (Or, first, second)

  (* Luau: Luau has types, if-expressions, floor division and buffers, and keeps NaN and subnormal bits through words. *)
  let if_else condition value otherwise : expression = IfElse (condition, value, otherwise)
  let not_ condition : expression = Unary (Not, condition)
end

module Conditions = Conditions.Make (Spelling)

let at_least tag lower upper variant : Expression.t =
  match (lower, upper) with
  | [ ([ only ], _) ], [ _ ] -> Binary ((if variant = 1 then NotEqual else Equal), tag, Integer (Int64.of_int only))
  | _ ->
      let bound = List.hd (fst (List.hd upper)) in
      Binary ((if variant = 1 then GreaterThanEqual else LessThan), tag, Integer (Int64.of_int bound))

let rec halved tag split runs =
  match runs with
  | [ (_, only) ] -> only
  | _ ->
      let half = List.length runs / 2 in
      let lower = List.take half runs in
      let upper = List.drop half runs in
      split (at_least tag lower upper) (halved tag split lower) (halved tag split upper)

let element (reader : reader) array leaf key : Expression.t =
  let operand = reader.value array leaf in
  Index (operand, value reader key)

let slot reference leaf : Expression.t = Index (reference, Integer (Int64.of_int (leaf + 1)))
let each_array (context : Context.t) array statement = Computation.each_array context.layouts array statement

let element_value (context : Context.t) reader element leaf =
  Computation.element_value context.layouts Expression.Nil reader element leaf

(* Luau: Luau has types, if-expressions, floor division and buffers, and keeps NaN and subnormal bits through words. *)

(** Luau has no NaN literal, and a subnormal literal reads as zero where subnormals flush to zero (Roblox), so either is
    joined from its two words, which keeps its exact bits. *)
let number_of_bits context value : Expression.t =
  let number = Int64.float_of_bits value in
  match Float.classify_float number with
  | FP_normal | FP_zero -> Number value
  | FP_infinite ->
      let huge = helper context "MathHuge" in
      if number > 0.0 then huge else Unary (Negate, huge)
  (* Luau: Luau has types, if-expressions, floor division and buffers, and keeps NaN and subnormal bits through words. *)
  | FP_subnormal | FP_nan ->
      call_helper context "LuauBitsIntoI64"
        [ Integer (Int64.logand value 0xFFFF_FFFFL); Integer (Int64.shift_right_logical value 32) ]

let literal context value_type bits = if Type.bits value_type = 64 then number_of_bits context bits else Integer bits
let create_constant context constant bits = literal context (Node.output_type (types context) constant 0) bits

(* Luau: Luau has types, if-expressions, floor division and buffers, and keeps NaN and subnormal bits through words. *)

(** Luau's [math.min] and [math.max] ignore a NaN after the first operand and do not order −0 below +0. *)
let operator node operand =
  let binary operator =
    let left = operand 0 in
    Some (Expression.Binary (operator, left, operand 1))
  in
  match node.kind with
  | Extension `LuauAdd -> binary Add
  | Extension `LuauSubtract -> binary Subtract
  | Extension `LuauMultiply -> binary Multiply
  | Extension `LuauDivide -> binary Divide
  | Extension `LuauNegate -> Some (Unary (Negate, operand 0))
  (* Luau: Luau has types, if-expressions, floor division and buffers, and keeps NaN and subnormal bits through words. *)
  | Extension `LuauFloorDivide -> binary FloorDivide
  | Extension `LuauModulo -> binary Modulo
  | _ -> None

(** [~=] negates [==] exactly, but an ordering takes [not], since [not (a < b)] and [a >= b] differ where either is NaN.
*)
let compare (operator : LuauOperator.binary) reader node variant : Expression.t =
  let left = value reader node.inputs.(0) in
  let right = value reader node.inputs.(1) in
  match (operator, variant) with
  | Equal, 0 -> Binary (NotEqual, left, right)
  | _ -> Conditions.holds (Binary (operator, left, right)) variant

let create_sum (context : Context.t) sum_type variant =
  Computation.create_sum context.layouts (fun variant -> Expression.Integer (Int64.of_int variant)) sum_type variant

let applied (context : Context.t) reader node : Expression.t =
  let callee = value reader node.inputs.(0) in
  Call (callee, Computation.input_values context.layouts reader node 1)

(* Luau: Luau has types, if-expressions, floor division and buffers, and keeps NaN and subnormal bits through words. *)
let choose (context : Context.t) reader matcher =
  let many tag runs = halved tag (fun test below above -> Expression.IfElse (test 0, below, above)) runs in
  single (Conditions.choose_value context.layouts many reader matcher)

let name text : Expression.t = Bytes text

let throw (context : Context.t) reader node : computation =
  let payload = Computation.input_values context.layouts reader node 0 in
  All (call_helper context "Throw" [ Expression.grouped payload ])

let caught (context : Context.t) payload_types thrown =
  (* Luau: Luau has types, if-expressions, floor division and buffers, and keeps NaN and subnormal bits through words. *)
  let leaves = Array.of_list (Layouts.leaves context.layouts payload_types) in
  let leaf = if Array.length leaves = 1 then fun _ -> thrown else slot thrown in
  each_leaf (fun index ->
      let caught = leaf index in
      Cast (caught, LuauTypes.leaf_type context.luau leaves.(index)))

let literal_element context element_type bytes index =
  literal context element_type
    (match Type.bits element_type with
    | 16 -> Int64.of_int (String.get_uint16_le bytes (2 * index))
    (* Luau: Luau has types, if-expressions, floor division and buffers, and keeps NaN and subnormal bits through words. *)
    | 32 -> Int64.logand (Int64.of_int32 (String.get_int32_le bytes (4 * index))) 0xFFFF_FFFFL
    | _ -> String.get_int64_le bytes (8 * index))

let create_array context element_type bytes : Expression.t =
  let length = String.length bytes / (Type.bits element_type / 8) in
  let elements = List.init length (literal_element context element_type bytes) in
  Table (elements, [ (Text "length", Integer (Int64.of_int length)) ])

let new_box context reader node : Expression.t = Table (leaf_values context reader node.inputs.(0), [])

(* Luau: Luau has types, if-expressions, floor division and buffers, and keeps NaN and subnormal bits through words. *)
let get_box_contents (context : Context.t) reader node =
  let reference = value reader node.inputs.(0) in
  let leaf_types = Layouts.leaves_of context.layouts (Node.output_type (types context) node 0) in
  match LuauType.union (List.map (LuauTypes.leaf_type context.luau) (Array.to_list leaf_types)) with
  | Union _ ->
      each_leaf (fun leaf ->
          let contents = slot reference leaf in
          Cast (contents, LuauTypes.leaf_type context.luau leaf_types.(leaf)))
  | _ -> each_leaf (slot reference)

(** One assignment per slot: a single assignment of every slot would hold each target and value in a register of its
    own, more than the 255 a Luau function has for a large box. *)
let set_box_contents context reader node : computation =
  let reference = value reader node.inputs.(0) in
  Statements
    (List.mapi
       (fun leaf content -> Statement.Assign ([ slot reference leaf ], [ content ]))
       (* Luau: Luau has types, if-expressions, floor division and buffers, and keeps NaN and subnormal bits through words. *)
       (leaf_values context reader node.inputs.(1)))

let retype (context : Context.t) reader node =
  let erased =
    match value reader node.inputs.(0) with
    | Cast (operand, _) | operand -> Expression.Cast (operand, Named (Text "any"))
  in
  single (Cast (erased, LuauTypes.leaf_type context.luau (Node.output_type (types context) node 0)))

let get_element reader node = each_leaf (fun leaf -> element reader node.inputs.(0) leaf node.inputs.(1))

let set_element context reader node =
  let array = node.inputs.(0) in
  each_array context
    (Link.type_ (types context) array)
    (fun leaf ->
      let target = element reader array leaf node.inputs.(1) in
      Statement.Assign ([ target ], [ element_value context reader node.inputs.(2) leaf ]))

let move_table context reader node =
  let key index = value reader node.inputs.(index) in
  each_array context
    (Link.type_ (types context) node.inputs.(0))
    (fun leaf ->
      let table index = reader.value node.inputs.(index) leaf in
      Statement.Call (call (Text "table.move") [ table 2; key 3; key 4; key 1; table 0 ]))

(* Luau: Luau has types, if-expressions, floor division and buffers, and keeps NaN and subnormal bits through words. *)

(** Each leaf of the array is a table of its own, cast from the optional table its allocation gives. *)
let new_array (context : Context.t) reader node =
  let leaf_types = Layouts.leaves_of context.layouts (Node.output_type (types context) node 0) in
  let length = value reader node.inputs.(0) in
  each_leaf (fun leaf ->
      let initial = element_value context reader node.inputs.(1) leaf in
      Cast (call_helper context "LuauArrayNew" [ length; initial ], LuauTypes.leaf_type context.luau leaf_types.(leaf)))

let create_buffer context reader node =
  single
    (Cast
       ( call_helper context "LuauBufferCreate" [ value reader node.inputs.(0) ],
         LuauTypes.leaf_type context.luau Type.bytes ))

let allocated context reader node variant =
  match leaf_values context reader node.inputs.(0) with
  | first :: rest -> Conditions.holds (List.fold_left Spelling.both first rest) variant
  | [] -> invalid_arg "An array has a leaf."

let test context reader node variant =
  match node.kind with
  | Extension `LuauEqual -> Some (compare Equal reader node variant)
  | Extension `LuauLessThan -> Some (compare LessThan reader node variant)
  | Extension `LuauLessThanEqual -> Some (compare LessThanEqual reader node variant)
  | Extension (`LuauAllocated _) -> Some (allocated context reader node variant)
  | _ -> None

let select tag runs = halved tag (fun test -> Conditions.choose (fun variant -> Holds (test variant))) runs

let called context reader node =
  match test context reader node 1 with
  (* Luau: Luau has types, if-expressions, floor division and buffers, and keeps NaN and subnormal bits through words. *)
  | Some condition -> single (IfElse (condition, Integer 1L, Integer 0L))
  | None ->
      let call = call_helper context (RuntimeLibrary.helper_name (types context) node) in
      Computation.calls context.layouts Expression.Nil name call (fun call -> Statement.Call call) reader node

let compute context reader node =
  let operand index = value reader node.inputs.(index) in
  match (node.kind, Node.bits node.kind) with
  | _, Some bits -> single (create_constant context node bits)
  | SumNew { sum_type; variant }, _ -> create_sum context sum_type variant
  (* Luau: Luau has types, if-expressions, floor division and buffers, and keeps NaN and subnormal bits through words. *)
  | RegionMatch _, _ -> choose context reader node
  | RegionReference body, _ -> single (Context.function_of context body)
  | FunctionApply, _ -> All (applied context reader node)
  | Throw _, _ -> throw context reader node
  | Extension (`LuauArrayOf (_, element_type, bytes)), _ -> single (create_array context element_type bytes)
  | Extension (`LuauBufferFromString bytes), _ -> single (call_helper context "LuauBufferFromString" [ Bytes bytes ])
  | Extension (`LuauLength _), _ -> single (Field (operand 0, Text "length"))
  | Extension (`LuauBoxNew _), _ -> single (new_box context reader node)
  (* Luau: Luau has types, if-expressions, floor division and buffers, and keeps NaN and subnormal bits through words. *)
  | Extension (`LuauBoxGet _), _ -> get_box_contents context reader node
  | Extension (`LuauBoxSet _), _ -> set_box_contents context reader node
  | Extension (`LuauRetype _), _ -> retype context reader node
  | Extension (`LuauTableGet _), _ -> get_element reader node
  | Extension (`LuauTableSet _), _ -> set_element context reader node
  | Extension (`LuauTableMove _), _ -> move_table context reader node
  | Extension (`LuauArrayNew _), _ -> new_array context reader node
  (* Luau: Luau has types, if-expressions, floor division and buffers, and keeps NaN and subnormal bits through words. *)
  | Extension `LuauBufferCreate, _ -> create_buffer context reader node
  | Extension `LuauWrap, _ -> single (call_helper context "LuauBit32Bor" [ operand 0; Integer 0L ])
  | Extension (`LuauCaught payload_types), _ -> caught context payload_types (operand 0)
  | _ -> ( match operator node operand with Some expression -> single expression | None -> called context reader node)
